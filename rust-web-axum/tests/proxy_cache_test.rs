use teaql_registry::engine::ProxyNegativeCache;

#[test]
fn test_proxy_negative_cache_lifecycle() {
    let cache = ProxyNegativeCache::default();

    assert!(!cache.is_negative_cached("maven-central", "/org/missing/lib.jar"));

    // Record 404 with 60 seconds TTL
    cache.record_not_found("maven-central", "/org/missing/lib.jar", 60);
    assert!(cache.is_negative_cached("maven-central", "/org/missing/lib.jar"));

    // Other paths are not affected
    assert!(!cache.is_negative_cached("maven-central", "/org/present/lib.jar"));

    // Invalidation
    cache.invalidate("maven-central", "/org/missing/lib.jar");
    assert!(!cache.is_negative_cached("maven-central", "/org/missing/lib.jar"));
}
