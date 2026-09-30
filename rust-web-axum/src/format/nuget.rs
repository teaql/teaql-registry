use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};

pub async fn extract_nupkg(
    content_type: Option<&str>,
    body: bytes::Bytes,
) -> anyhow::Result<bytes::Bytes> {
    let Some(content_type) = content_type else {
        return Ok(body);
    };
    if !content_type
        .to_ascii_lowercase()
        .starts_with("multipart/form-data")
    {
        return Ok(body);
    }

    let boundary = multer::parse_boundary(content_type)?;
    let stream =
        futures_util::stream::once(
            async move { Ok::<bytes::Bytes, std::convert::Infallible>(body) },
        );
    let mut multipart = multer::Multipart::new(stream, boundary);
    while let Some(field) = multipart.next_field().await? {
        let is_package = field.name() == Some("package")
            || field
                .file_name()
                .is_some_and(|name| name.to_ascii_lowercase().ends_with(".nupkg"));
        if is_package {
            return Ok(field.bytes().await?);
        }
    }
    anyhow::bail!("NuGet multipart upload does not contain a package part")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuGetResource {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuGetServiceIndex {
    pub version: String,
    pub resources: Vec<NuGetResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuGetPackageVersions {
    pub versions: Vec<String>,
}

pub fn read_nupkg_identity(data: &[u8]) -> anyhow::Result<(String, String)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(data))?;
    let mut nuspec = String::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        if entry.name().to_ascii_lowercase().ends_with(".nuspec") {
            entry.read_to_string(&mut nuspec)?;
            break;
        }
    }
    if nuspec.is_empty() {
        anyhow::bail!("NuGet package does not contain a .nuspec manifest");
    }

    let mut reader = quick_xml::Reader::from_str(&nuspec);
    reader.config_mut().trim_text(true);
    let mut id = None;
    let mut version = None;
    loop {
        match reader.read_event()? {
            quick_xml::events::Event::Start(element) => match element.local_name().as_ref() {
                b"id" if id.is_none() => {
                    id = Some(reader.read_text(element.name())?.into_owned());
                }
                b"version" if version.is_none() => {
                    version = Some(reader.read_text(element.name())?.into_owned());
                }
                _ => {}
            },
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
    }

    let id = id
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("NuGet .nuspec manifest does not contain a package id"))?;
    let version = version
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("NuGet .nuspec manifest does not contain a version"))?;
    Ok((id, version))
}

pub fn create_nuget_service_index(base_url: &str) -> NuGetServiceIndex {
    NuGetServiceIndex {
        version: "3.0.0".to_string(),
        resources: vec![
            NuGetResource {
                id: format!("{}/v3/package", base_url.trim_end_matches('/')),
                resource_type: "PackagePublish/2.0.0".to_string(),
                comment: Some("Initial push endpoint".to_string()),
            },
            NuGetResource {
                id: format!("{}/v3/flatcontainer/", base_url.trim_end_matches('/')),
                resource_type: "PackageBaseAddress/3.0.0".to_string(),
                comment: Some(
                    "Base URL of Azure storage where NuGet packages are stored".to_string(),
                ),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::read_nupkg_identity;
    use std::io::{Cursor, Write};
    use zip::{write::SimpleFileOptions, ZipWriter};

    #[test]
    fn reads_identity_from_nuspec_manifest() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("TeaQL.Native.Probe.nuspec", SimpleFileOptions::default())
            .unwrap();
        writer
            .write_all(
                br#"<?xml version="1.0"?>
<package xmlns="http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd">
  <metadata><id>TeaQL.Native.Probe</id><version>2.3.4</version></metadata>
</package>"#,
            )
            .unwrap();
        let bytes = writer.finish().unwrap().into_inner();

        assert_eq!(
            read_nupkg_identity(&bytes).unwrap(),
            ("TeaQL.Native.Probe".to_string(), "2.3.4".to_string())
        );
    }
}
