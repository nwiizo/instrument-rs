//! Test for dependency detection

#[cfg(test)]
mod tests {
    use crate::dependencies::{
        CacheCrate, DatabaseCrate, DetectionContext, FrameworkCrate, HttpClientCrate,
        ObservabilityCrate, ProjectDependencies,
    };
    use std::path::Path;

    #[test]
    fn test_e2e_project_dependency_detection() {
        let path = Path::new("/tmp/e2e-test-project");
        let deps = ProjectDependencies::from_manifest(path).unwrap();

        println!("\n=== Dependency Detection Results ===");
        println!("Summary:\n{}", deps.summary());

        // Verify framework detection
        assert!(
            deps.frameworks.contains(&FrameworkCrate::Axum),
            "Should detect axum framework"
        );

        // Verify database detection
        assert!(
            deps.databases.contains(&DatabaseCrate::Sqlx),
            "Should detect sqlx database"
        );

        // Verify HTTP client detection
        assert!(
            deps.http_clients.contains(&HttpClientCrate::Reqwest),
            "Should detect reqwest HTTP client"
        );

        // Verify cache detection
        assert!(
            deps.caches.contains(&CacheCrate::Redis),
            "Should detect redis cache"
        );

        // Verify observability detection
        assert!(
            deps.observability.contains(&ObservabilityCrate::Tracing),
            "Should detect tracing"
        );

        println!("\nAll dependencies correctly detected!");
    }

    #[test]
    fn test_detection_context_with_e2e_deps() {
        let path = Path::new("/tmp/e2e-test-project");
        let deps = ProjectDependencies::from_manifest(path).unwrap();
        let ctx = DetectionContext::from_deps(deps);

        println!("\n=== Detection Context ===");
        println!("DB Priority: {}", ctx.db_priority);
        println!("HTTP Priority: {}", ctx.http_priority);
        println!("Cache Priority: {}", ctx.cache_priority);

        // With sqlx in deps, should detect DB operations
        assert!(ctx.is_likely_db_operation("query_users"));
        assert!(ctx.is_likely_db_operation("fetch_order"));
        assert!(ctx.is_likely_db_operation("insert_product"));

        // With reqwest in deps, should detect HTTP calls (but be specific)
        assert!(ctx.is_likely_http_call("call_api_endpoint"));
        assert!(ctx.is_likely_http_call("send_request_to_service"));
        // But NOT generic "get_user" - this was the false positive we fixed
        assert!(!ctx.is_likely_http_call("get_user"));

        // With redis in deps, should detect cache operations
        assert!(ctx.is_likely_cache_operation("cache_get_user"));
        assert!(ctx.is_likely_cache_operation("invalidate_cache"));

        println!("Context-aware detection working correctly!");
    }

    #[test]
    fn test_no_deps_project() {
        // Test a project without dependencies
        let deps = ProjectDependencies::default();
        let ctx = DetectionContext::from_deps(deps);

        // Without deps, should NOT match anything
        assert!(!ctx.is_likely_db_operation("query_users"));
        assert!(!ctx.is_likely_http_call("send_request"));
        assert!(!ctx.is_likely_cache_operation("cache_get"));

        println!("No-deps project correctly returns no matches!");
    }
}
