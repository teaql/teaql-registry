use anyhow::Result;
use bytes::Bytes;
use nexus_repository_service_core::{RepositoryConfiguration, ServiceRuntime};

use crate::blobstore::BlobStore;
use crate::format::nuget::{create_nuget_service_index, NuGetPackageVersions, NuGetServiceIndex};
use crate::services::{AssetService, ComponentService, RepositoryService};

pub struct NuGetEngine;

impl NuGetEngine {
    pub async fn get_service_index(base_url: &str) -> NuGetServiceIndex {
        create_nuget_service_index(base_url)
    }

    pub async fn upload_package(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        id: &str,
        version: &str,
        nupkg_data: &[u8],
    ) -> Result<()> {
        let content_repo = RepositoryService::ensure_content_repository(ctx, repo.id(), "nuget").await?;
        let blob_info = blobstore.create_blob(nupkg_data).await?;

        let id_lower = id.to_lowercase();
        let ver_lower = version.to_lowercase();

        let asset_blob = AssetService::create_asset_blob(
            ctx,
            repo.blob_store_id(),
            &blob_info.blob_ref,
            blob_info.size,
            "application/zip",
            &blob_info.checksums.sha1,
            &blob_info.checksums.sha256,
            &blob_info.checksums.md5,
        )
        .await?;

        let comp = ComponentService::find_or_create(
            ctx,
            content_repo.id(),
            "",
            &id_lower,
            &ver_lower,
            "nupkg",
        )
        .await?;

        let path = format!("/v3/flatcontainer/{}/{}/{}.{}.nupkg", id_lower, ver_lower, id_lower, ver_lower);
        AssetService::upsert_asset(
            ctx,
            content_repo.id(),
            Some(comp.id()),
            asset_blob.id(),
            &path,
            "nupkg",
        )
        .await?;

        Ok(())
    }

    pub async fn get_package_versions(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        id: &str,
    ) -> Result<Option<NuGetPackageVersions>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let id_lower = id.to_lowercase();
        let comps = ComponentService::list_by_content_repository(ctx, content_repo.id(), 100, 0).await?;
        let matching: Vec<_> = comps.into_iter().filter(|c| c.name().to_lowercase() == id_lower).collect();

        if matching.is_empty() {
            return Ok(None);
        }

        let versions: Vec<String> = matching.into_iter().map(|c| c.version_name().to_string()).collect();
        Ok(Some(NuGetPackageVersions { versions }))
    }

    pub async fn get_package_file(
        ctx: &ServiceRuntime,
        repo: &RepositoryConfiguration,
        blobstore: &dyn BlobStore,
        id: &str,
        version: &str,
    ) -> Result<Option<Bytes>> {
        let content_repo = match RepositoryService::get_content_repository(ctx, repo.id()).await? {
            Some(cr) => cr,
            None => return Ok(None),
        };

        let id_lower = id.to_lowercase();
        let ver_lower = version.to_lowercase();
        let path = format!("/v3/flatcontainer/{}/{}/{}.{}.nupkg", id_lower, ver_lower, id_lower, ver_lower);

        let asset = match AssetService::find_by_path(ctx, content_repo.id(), &path).await? {
            Some(a) => a,
            None => return Ok(None),
        };

        let asset_blob = match AssetService::get_asset_blob(ctx, asset.asset_blob_id()).await? {
            Some(b) => b,
            None => return Ok(None),
        };

        match blobstore.read_blob(&asset_blob.blob_ref()).await {
            Ok(data) => Ok(Some(data)),
            Err(_) => Ok(None),
        }
    }
}
