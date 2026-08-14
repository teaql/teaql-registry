use anyhow::Result;
use bytes::Bytes;
use nexus_repository_service_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::FileBlobStore;
use crate::format::pypi::{
    generate_pypi_simple_package_html, generate_pypi_simple_root_html, PyPiFileEntry,
};
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct PyPiEngine;

impl PyPiEngine {
    pub async fn upload_file(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &FileBlobStore,
        project_name: &str,
        version: &str,
        filename: &str,
        data: &[u8],
    ) -> Result<()> {
        let content_repo = RepositoryService::ensure_content_repository(ctx, repo.id(), "pypi").await?;
        let blob_info = blobstore.create_blob(data).await?;

        let ct = if filename.ends_with(".whl") {
            "application/x-wheel+zip"
        } else {
            "application/gzip"
        };

        let asset_blob = AssetService::create_asset_blob(
            ctx,
            repo.blob_store_id(),
            &blob_info.blob_ref,
            blob_info.size,
            ct,
            &blob_info.checksums.sha1,
            &blob_info.checksums.sha256,
            &blob_info.checksums.md5,
        )
        .await?;

        let comp = ComponentService::find_or_create(
            ctx,
            content_repo.id(),
            "",
            project_name,
            version,
            if filename.ends_with(".whl") { "whl" } else { "tar.gz" },
        )
        .await?;

        let path = format!("/packages/{}", filename);
        AssetService::upsert_asset(
            ctx,
            content_repo.id(),
            Some(comp.id()),
            asset_blob.id(),
            &path,
            "distribution",
        )
        .await?;

        Ok(())
    }

    pub async fn get_simple_root(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
    ) -> Result<String> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(generate_pypi_simple_root_html(&[])),
        };

        let comps = ComponentService::list_by_content_repository(ctx, content_repo.id(), 1000, 0).await?;
        let mut names: Vec<String> = comps.into_iter().map(|c| c.name().to_string()).collect();
        names.sort();
        names.dedup();

        Ok(generate_pypi_simple_root_html(&names))
    }

    pub async fn get_simple_package(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        project_name: &str,
    ) -> Result<Option<String>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let comps = ComponentService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
        let matching: Vec<_> = comps.into_iter().filter(|c| c.name().eq_ignore_ascii_case(project_name)).collect();

        if matching.is_empty() {
            return Ok(None);
        }

        let assets = AssetService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
        let mut files = Vec::new();

        for a in assets {
            let comp_id = a.component_id();
            if comp_id > 0 && matching.iter().any(|m| m.id() == comp_id as u64) {
                let filename = a.path().trim_start_matches("/packages/").to_string();
                if let Ok(Some(blob)) = AssetService::get_asset_blob(ctx, a.asset_blob_id()).await {
                    files.push(PyPiFileEntry {
                        filename: filename.clone(),
                        url: format!("../../packages/{}", filename),
                        sha256: blob.sha256_checksum().to_string(),
                    });
                }
            }
        }

        Ok(Some(generate_pypi_simple_package_html(project_name, &files)))
    }

    pub async fn get_package_file(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &FileBlobStore,
        filename: &str,
    ) -> Result<Option<(Bytes, String)>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let path = format!("/packages/{}", filename);
        let asset = match AssetService::find_by_path(ctx, content_repo.id(), &path).await? {
            Some(a) => a,
            None => return Ok(None),
        };

        let asset_blob = match AssetService::get_asset_blob(ctx, asset.asset_blob_id()).await? {
            Some(b) => b,
            None => return Ok(None),
        };

        match blobstore.read_blob(&asset_blob.blob_ref()).await {
            Ok(data) => Ok(Some((data, asset_blob.content_type().to_string()))),
            Err(_) => Ok(None),
        }
    }
}
