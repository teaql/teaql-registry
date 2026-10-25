//! Read-only protocol adapters, executed after the existing authentication layer.
use crate::{
    api::AppState,
    engine::remote::{RemoteAsset, RemoteCache},
    security::RequestContext,
    services::RepositoryService,
};
use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use serde_json::Value;
use std::sync::Arc;
use teaql_registry_core::RepositoryConfiguration;

pub async fn proxy_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    let (docker, rest) = if let Some(rest) = path.strip_prefix("/repository/") {
        (false, rest)
    } else if let Some(rest) = path.strip_prefix("/v2/") {
        (true, rest)
    } else {
        return next.run(request).await;
    };
    let Some((name, tail)) = rest.split_once('/') else {
        return next.run(request).await;
    };
    let Some(context) = request.extensions().get::<Arc<RequestContext>>().cloned() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Missing authenticated request context",
        )
            .into_response();
    };
    let repo = match RepositoryService::find_by_name(&context.runtime, name).await {
        Ok(Some(repo)) => repo,
        Ok(None) => return next.run(request).await,
        Err(error) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
        }
    };
    let recipe = repo.recipe_name();
    let Some(format) = recipe.strip_suffix("-proxy") else {
        return next.run(request).await;
    };
    if matches!(format, "npm" | "maven2" | "maven" | "raw") {
        return next.run(request).await;
    }
    if !matches!(
        format,
        "cargo"
            | "pypi"
            | "gomod"
            | "nuget"
            | "docker"
            | "swift"
            | "dart"
            | "rubygems"
            | "composer"
            | "conan"
            | "hex"
    ) {
        return next.run(request).await;
    }
    if !repo.online() {
        return (StatusCode::SERVICE_UNAVAILABLE, "Repository is offline").into_response();
    }
    if !matches!(*request.method(), Method::GET | Method::HEAD) {
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            "Proxy repositories only support downloads",
        )
            .into_response();
    }
    if docker != (format == "docker") {
        return (StatusCode::BAD_REQUEST, "Wrong protocol endpoint").into_response();
    }
    let head = request.method() == Method::HEAD;
    let local_repo = format!(
        "{}/repository/{name}",
        super::public_base_url(request.headers())
    );
    let result = dispatch(
        &state,
        &context,
        &repo,
        format,
        tail,
        request.uri().query(),
        request.headers(),
        &local_repo,
    )
    .await;
    match result {
        Ok(Some(asset)) => {
            let size = asset.body.len();
            let mut response = (
                asset.headers,
                if head {
                    Body::empty()
                } else {
                    Body::from(asset.body)
                },
            )
                .into_response();
            response
                .headers_mut()
                .insert(header::CONTENT_LENGTH, size.into());
            response
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Upstream artifact not found").into_response(),
        Err(error) => (StatusCode::BAD_GATEWAY, error.to_string()).into_response(),
    }
}

#[allow(clippy::too_many_arguments)]
async fn dispatch(
    state: &AppState,
    context: &RequestContext,
    repo: &RepositoryConfiguration,
    format: &str,
    tail: &str,
    query: Option<&str>,
    incoming: &HeaderMap,
    local: &str,
) -> Result<Option<RemoteAsset>> {
    let remote = repo.remote_url();
    anyhow::ensure!(!remote.is_empty(), "Proxy repository has no upstream URL");
    let mut accept = incoming
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("*/*")
        .to_string();
    if format == "docker" {
        accept = "application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json".into();
    }
    if format == "pypi" && !tail.starts_with("_proxy/") {
        accept = "application/vnd.pypi.simple.v1+json, application/vnd.pypi.simple.v1+html;q=0.9, text/html;q=0.8".into();
    }
    if format == "gomod" {
        if let Some(checksum_path) = tail.strip_prefix("gomod/sumdb/sum.golang.org/") {
            if checksum_path == "supported" {
                return Ok(Some(RemoteAsset {
                    body: Bytes::new(),
                    headers: HeaderMap::new(),
                }));
            }
            let checksum_base = std::env::var("TEAQL_PROXY_SUMDB_BASE_URL")
                .unwrap_or_else(|_| "https://sum.golang.org".into());
            let checksum_url = join(&checksum_base, checksum_path, query)?;
            return RemoteCache::fetch(
                &context.runtime,
                repo,
                &state.blobstore,
                &checksum_url,
                &accept,
                false,
            )
            .await;
        }
    }
    let url = if let Some(link) = tail.strip_prefix("_proxy/link/") {
        let Some((id, suffix)) = link.split_once('/') else {
            return Ok(None);
        };
        let Some(url) =
            RemoteCache::resolve_link(&context.runtime, repo, &state.blobstore, id, suffix, query)
                .await?
        else {
            return Ok(None);
        };
        url
    } else if format == "cargo" {
        if tail.ends_with("config.json") {
            let config = serde_json::json!({"dl":format!("{local}/api/v1/crates/{{crate}}/{{version}}/download"), "api":local,
                "auth-required": state.enforces_authentication() && !state.allows_anonymous_read()});
            // Warm the original configuration; archive resolution needs it offline.
            let url = join(&remote, "config.json", None)?;
            if RemoteCache::fetch(
                &context.runtime,
                repo,
                &state.blobstore,
                &url,
                "application/json",
                false,
            )
            .await?
            .is_none()
            {
                return Ok(None);
            }
            return Ok(Some(json_asset(&config)?));
        }
        if let Some(path) = tail.strip_prefix("cargo/index/") {
            join(&remote, path, query)?
        } else if let Some(path) = tail.strip_prefix("api/v1/crates/") {
            let pieces = path.split('/').collect::<Vec<_>>();
            anyhow::ensure!(
                pieces.len() == 3 && pieces[2] == "download",
                "Invalid Cargo archive path"
            );
            let config_url = join(&remote, "config.json", None)?;
            let config = RemoteCache::fetch(
                &context.runtime,
                repo,
                &state.blobstore,
                &config_url,
                "application/json",
                false,
            )
            .await?
            .ok_or_else(|| anyhow!("Missing upstream Cargo configuration"))?;
            let config: Value = serde_json::from_slice(&config.body)?;
            let template = config
                .get("dl")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("Cargo upstream has no dl URL"))?;
            let name = pieces[0];
            let version = pieces[1];
            let prefix = cargo_prefix(name);
            let mut url = if template.contains('{') {
                template.to_string()
            } else {
                format!(
                    "{}/{name}/{version}/download",
                    template.trim_end_matches('/')
                )
            };
            for (key, value) in [
                ("{crate}", name),
                ("{version}", version),
                ("{prefix}", prefix.as_str()),
                ("{lowerprefix}", prefix.to_lowercase().as_str()),
            ] {
                url = url.replace(key, value);
            }
            if url.contains("{sha256-checksum}") {
                let index_url = join(
                    &remote,
                    &format!("{}/{name}", cargo_prefix(&name.to_lowercase())),
                    None,
                )?;
                let index = RemoteCache::fetch(
                    &context.runtime,
                    repo,
                    &state.blobstore,
                    &index_url,
                    "*/*",
                    false,
                )
                .await?
                .ok_or_else(|| anyhow!("Missing Cargo index"))?;
                let checksum = std::str::from_utf8(&index.body)?
                    .lines()
                    .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                    .find(|v| v["vers"] == version)
                    .and_then(|v| v["cksum"].as_str().map(str::to_string))
                    .ok_or_else(|| anyhow!("Missing Cargo checksum"))?;
                url = url.replace("{sha256-checksum}", &checksum);
            }
            url
        } else {
            return Ok(None);
        }
    } else if format == "nuget" && tail == "v3/index.json" {
        if remote.ends_with(".json") {
            remote.clone()
        } else {
            join(&remote, "v3/index.json", query)?
        }
    } else {
        let prefix = match format {
            "gomod" => "gomod/",
            "swift" => "swift/",
            "dart" => "dart/",
            "rubygems" => "rubygems/",
            "composer" => "composer/",
            "conan" => "conan/",
            "hex" => "hex/repo/",
            _ => "",
        };
        let path = tail.strip_prefix(prefix).unwrap_or(tail);
        if format == "docker" {
            join(&remote, &format!("v2/{path}"), query)?
        } else if format == "pypi" && remote.trim_end_matches('/').ends_with("/simple") {
            join(&remote, path.strip_prefix("simple/").unwrap_or(path), query)?
        } else {
            join(&remote, path, query)?
        }
    };
    let Some(mut asset) = RemoteCache::fetch(
        &context.runtime,
        repo,
        &state.blobstore,
        &url,
        &accept,
        format == "docker",
    )
    .await?
    else {
        return Ok(None);
    };
    let content_type = asset
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let is_json = content_type.contains("json")
        || (matches!(format, "nuget" | "dart" | "composer" | "swift" | "conan")
            && serde_json::from_slice::<Value>(&asset.body).is_ok());
    let mut transformed = false;
    if is_json
        && matches!(
            format,
            "pypi" | "nuget" | "dart" | "composer" | "swift" | "conan"
        )
    {
        let mut json: Value = serde_json::from_slice(&asset.body)?;
        if format == "composer" {
            expand_composer(&mut json);
        }
        let mut links = Vec::new();
        collect_links(&json, "", "", format, &mut links);
        for (pointer, value) in links {
            let source = absolute(&url, &value)?;
            let rewritten = if source.contains("%package%") || source.contains("%hash%") {
                let marker = source.find('%').unwrap_or(source.len());
                let split = source[..marker].rfind('/').unwrap_or(marker) + 1;
                let directory = &source[..split];
                let prefix = RemoteCache::bind_link(
                    &context.runtime,
                    repo,
                    &state.blobstore,
                    directory,
                    local,
                    true,
                )
                .await?;
                format!("{prefix}{}", &source[split..])
            } else {
                let directory = source.split('?').next().unwrap_or(&source).ends_with('/');
                RemoteCache::bind_link(
                    &context.runtime,
                    repo,
                    &state.blobstore,
                    &source,
                    local,
                    directory,
                )
                .await?
            };
            if let Some(slot) = json.pointer_mut(&pointer) {
                *slot = Value::String(rewritten);
            }
        }
        if format == "composer" {
            // These optional reporting endpoints would send client installation
            // telemetry to the public upstream; a download proxy does not need them.
            if let Some(object) = json.as_object_mut() {
                object.remove("notify");
                object.remove("notify-batch");
            }
        }
        asset.body = Bytes::from(serde_json::to_vec(&json)?);
        transformed = true;
    } else if format == "pypi" && content_type.contains("html") {
        let html = std::str::from_utf8(&asset.body)?.to_string();
        let re = regex::Regex::new(r#"(?i)(href\s*=\s*["'])([^"']+)(["'])"#)?;
        let mut rewritten = String::new();
        let mut offset = 0;
        for captures in re.captures_iter(&html) {
            let found = captures.get(0).expect("HTML match");
            rewritten.push_str(&html[offset..found.start()]);
            let source = absolute(&url, &captures[2].replace("&amp;", "&"))?;
            let link = RemoteCache::bind_link(
                &context.runtime,
                repo,
                &state.blobstore,
                &source,
                local,
                source.split('#').next().unwrap_or(&source).ends_with('/'),
            )
            .await?;
            rewritten.push_str(&format!(
                "{}{}{}",
                &captures[1],
                link.replace('&', "&amp;"),
                &captures[3]
            ));
            offset = found.end();
        }
        rewritten.push_str(&html[offset..]);
        asset.body = Bytes::from(rewritten);
        transformed = true;
    }
    if format == "swift" {
        if let Some(link_header) = asset
            .headers
            .get(header::LINK)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
        {
            let re = regex::Regex::new(r"<([^>]+)>")?;
            let mut links = String::new();
            let mut offset = 0;
            for capture in re.captures_iter(&link_header) {
                let found = capture.get(0).expect("Link match");
                links.push_str(&link_header[offset..found.start()]);
                let remote = absolute(&url, &capture[1])?;
                let bound = RemoteCache::bind_link(
                    &context.runtime,
                    repo,
                    &state.blobstore,
                    &remote,
                    local,
                    false,
                )
                .await?;
                links.push_str(&format!("<{bound}>"));
                offset = found.end();
            }
            links.push_str(&link_header[offset..]);
            asset.headers.insert(header::LINK, links.parse()?);
        }
    }
    if transformed {
        asset.headers.remove(header::ETAG);
        asset.headers.remove("digest");
        asset.headers.remove(header::CONTENT_ENCODING);
    }
    if format == "docker" {
        asset
            .headers
            .insert("docker-distribution-api-version", "registry/2.0".parse()?);
    }
    Ok(Some(asset))
}
fn join(base: &str, path: &str, query: Option<&str>) -> Result<String> {
    let mut url = reqwest::Url::parse(&format!("{}/", base.trim_end_matches('/')))?
        .join(path.trim_start_matches('/'))?;
    if let Some(query) = query {
        url.set_query(Some(query));
    }
    Ok(url.into())
}
fn absolute(base: &str, value: &str) -> Result<String> {
    Ok(reqwest::Url::parse(base)?.join(value)?.into())
}
fn cargo_prefix(name: &str) -> String {
    match name.len() {
        0 => String::new(),
        1 => "1".into(),
        2 => "2".into(),
        3 => format!("3/{}", &name[..1]),
        _ => format!("{}/{}", &name[..2], &name[2..4]),
    }
}
fn json_asset(value: &Value) -> Result<RemoteAsset> {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse()?);
    Ok(RemoteAsset {
        body: Bytes::from(serde_json::to_vec(value)?),
        headers,
    })
}
fn collect_links(
    value: &Value,
    pointer: &str,
    key: &str,
    format: &str,
    out: &mut Vec<(String, String)>,
) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                collect_links(
                    value,
                    &format!("{}/{}", pointer, key.replace('~', "~0").replace('/', "~1")),
                    key,
                    format,
                    out,
                );
            }
        }
        Value::Array(array) => {
            for (index, value) in array.iter().enumerate() {
                collect_links(value, &format!("{pointer}/{index}"), key, format, out);
            }
        }
        Value::String(url) => {
            let selected = match format {
                "pypi" => key == "url" || (pointer.starts_with("/projects/") && key == "url"),
                "nuget" => matches!(
                    key,
                    "@id"
                        | "packageContent"
                        | "contentUrl"
                        | "registration"
                        | "catalogEntry"
                        | "url"
                        | "base"
                        | "index"
                ),
                "dart" => key == "archive_url",
                "composer" => {
                    key == "metadata-url"
                        || key == "providers-url"
                        || (key == "url" && pointer.contains("/dist/"))
                }
                "swift" => key == "url",
                "conan" => true,
                _ => false,
            };
            if selected
                && (url.starts_with("http://")
                    || url.starts_with("https://")
                    || url.starts_with('/')
                    || (format == "pypi" && key == "url"))
            {
                out.push((pointer.into(), url.clone()));
            }
        }
        _ => {}
    }
}
fn expand_composer(value: &mut Value) {
    if value.get("minified").and_then(Value::as_str) != Some("composer/2.0") {
        return;
    }
    if let Some(packages) = value.get_mut("packages").and_then(Value::as_object_mut) {
        for versions in packages.values_mut().filter_map(Value::as_array_mut) {
            let mut previous = serde_json::Map::new();
            for version in versions {
                if let Some(fields) = version.as_object() {
                    for (key, value) in fields {
                        if value.as_str() == Some("__unset") {
                            previous.remove(key);
                        } else {
                            previous.insert(key.clone(), value.clone());
                        }
                    }
                    *version = Value::Object(previous.clone());
                }
            }
        }
    }
    if let Some(object) = value.as_object_mut() {
        object.remove("minified");
    }
}
