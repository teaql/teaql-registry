#![recursion_limit = "256"]

mod common;

use std::panic::{catch_unwind, AssertUnwindSafe};

use teaql_registry::{
    context::NexusContextExt,
    services::{
        AssetService, BlobStoreService, ComponentService, RepositoryService, TenantService,
    },
};
use teaql_registry_core::{service_runtime, E};

#[test]
fn e_expression_distinguishes_selected_values_from_missing_relations() {
    common::run_with_large_stack(expression_projection_contract_body);
}

async fn expression_projection_contract_body() {
    let mut context = service_runtime(common::runtime_config())
        .await
        .expect("connect test runtime");
    context.ensure_schema().await.expect("ensure test schema");

    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let tenant_name = format!("E Expression Tenant {suffix}");
    let tenant =
        TenantService::create_tenant(&context, &tenant_name, &format!("e-expression-{suffix}"))
            .await
            .expect("create tenant fixture");
    context.set_tenant(tenant.id(), &tenant_name);
    let blob_store = BlobStoreService::create(
        &context,
        &format!("e-expression-store-{suffix}"),
        "/tmp/e-expression-blobs",
        true,
    )
    .await
    .expect("create blob store fixture");
    let repository = RepositoryService::create(
        &context,
        &format!("e-expression-repo-{suffix}"),
        "raw-hosted",
        "HOSTED",
        "RAW",
        "ALLOW_WRITE",
        blob_store.id(),
        true,
        "",
    )
    .await
    .expect("create repository fixture");
    let content_repository =
        RepositoryService::ensure_content_repository(&context, repository.id(), "RAW")
            .await
            .expect("create content repository fixture");
    let blob = AssetService::create_asset_blob(
        &context,
        blob_store.id(),
        &format!("default@{suffix}"),
        17,
        "application/octet-stream",
        "sha1",
        "sha256",
        "md5",
    )
    .await
    .expect("create asset blob fixture");
    let path = format!("/expression/{suffix}.bin");
    AssetService::create(
        &context,
        content_repository.id(),
        0,
        blob.id(),
        &path,
        "binary",
    )
    .await
    .expect("create asset fixture");

    // A separate context prevents a just-created related entity from being
    // resolved through the first context's identity map.
    let mut read_context = service_runtime(common::runtime_config())
        .await
        .expect("connect isolated read runtime");
    read_context
        .ensure_schema()
        .await
        .expect("ensure isolated read schema");
    read_context.set_tenant(tenant.id(), &tenant_name);

    let scalar_only = AssetService::find_by_path(&read_context, content_repository.id(), &path)
        .await
        .expect("query scalar-only asset")
        .expect("asset fixture exists");
    assert_eq!(E::asset(&scalar_only).get_path().unwrap(), path);

    let missing_relation = catch_unwind(AssertUnwindSafe(|| {
        E::asset(&scalar_only).get_asset_blob().eval()
    }));
    assert!(
        missing_relation.is_err(),
        "E must fail fast when asset_blob was not selected"
    );

    let loaded =
        AssetService::find_by_path_with_blob(&read_context, content_repository.id(), &path)
            .await
            .expect("query relation-aware asset")
            .expect("asset fixture exists");
    let loaded_blob = loaded.blob().expect("selected blob relation is present");
    assert_eq!(loaded.path(), path);
    assert_eq!(loaded_blob.blob_ref(), format!("default@{suffix}"));
    assert_eq!(loaded_blob.blob_size(), 17);
}

#[test]
fn generated_component_pages_are_stable_and_non_overlapping() {
    common::run_with_large_stack(component_pagination_contract_body);
}

async fn component_pagination_contract_body() {
    let mut context = service_runtime(common::runtime_config())
        .await
        .expect("connect pagination test runtime");
    context.ensure_schema().await.expect("ensure test schema");

    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let tenant_name = format!("Pagination Tenant {suffix}");
    let tenant =
        TenantService::create_tenant(&context, &tenant_name, &format!("pagination-{suffix}"))
            .await
            .expect("create pagination tenant");
    context.set_tenant(tenant.id(), &tenant_name);

    let blob_store = BlobStoreService::create(
        &context,
        &format!("pagination-store-{suffix}"),
        "/tmp/pagination-blobs",
        true,
    )
    .await
    .expect("create pagination blob store");
    let repository = RepositoryService::create(
        &context,
        &format!("pagination-repo-{suffix}"),
        "raw-hosted",
        "HOSTED",
        "RAW",
        "ALLOW_WRITE",
        blob_store.id(),
        true,
        "",
    )
    .await
    .expect("create pagination repository");
    let content_repository =
        RepositoryService::ensure_content_repository(&context, repository.id(), "RAW")
            .await
            .expect("create pagination content repository");

    for ordinal in 1..=3 {
        ComponentService::create(
            &context,
            content_repository.id(),
            "pagination",
            &format!("component-{ordinal}"),
            &ordinal.to_string(),
            &ordinal.to_string(),
            "test-component",
        )
        .await
        .expect("create paged component fixture");
    }

    let first =
        ComponentService::list_by_content_repository(&context, content_repository.id(), 2, 0)
            .await
            .expect("query first component page");
    let second =
        ComponentService::list_by_content_repository(&context, content_repository.id(), 2, 2)
            .await
            .expect("query second component page");

    assert_eq!(first.len(), 2);
    assert_eq!(second.len(), 1);
    assert!(first[0].id() < first[1].id());
    assert!(first[1].id() < second[0].id());
    assert!(first
        .iter()
        .all(|left| second.iter().all(|right| left.id() != right.id())));
}
