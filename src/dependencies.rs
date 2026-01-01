//! Dependency analysis for Rust projects
//!
//! This module uses cargo_metadata to analyze project dependencies and
//! provide context for smarter instrumentation detection.

use crate::{Error, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use std::collections::HashSet;
use std::path::Path;

/// Known database crates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatabaseCrate {
    /// SQLx async database driver
    Sqlx,
    /// Diesel ORM
    Diesel,
    /// SeaORM async ORM
    SeaOrm,
    /// tokio-postgres driver
    TokioPostgres,
    /// rusqlite for SQLite
    Rusqlite,
    /// mongodb driver
    MongoDb,
}

impl DatabaseCrate {
    /// Returns the crate name as it appears in Cargo.toml
    pub fn crate_name(&self) -> &'static str {
        match self {
            Self::Sqlx => "sqlx",
            Self::Diesel => "diesel",
            Self::SeaOrm => "sea-orm",
            Self::TokioPostgres => "tokio-postgres",
            Self::Rusqlite => "rusqlite",
            Self::MongoDb => "mongodb",
        }
    }

    /// Returns patterns that indicate usage of this database crate
    pub fn usage_patterns(&self) -> &'static [&'static str] {
        match self {
            Self::Sqlx => &["sqlx::query", "Pool", "PgPool", "MySqlPool", "SqlitePool"],
            Self::Diesel => &["diesel::", "table!", "Insertable", "Queryable"],
            Self::SeaOrm => &["sea_orm::", "Entity", "ActiveModel"],
            Self::TokioPostgres => &["tokio_postgres::", "Client"],
            Self::Rusqlite => &["rusqlite::", "Connection"],
            Self::MongoDb => &["mongodb::", "Collection", "Database"],
        }
    }
}

/// Known HTTP client crates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpClientCrate {
    /// reqwest HTTP client
    Reqwest,
    /// hyper HTTP library
    Hyper,
    /// ureq blocking HTTP client
    Ureq,
    /// surf async HTTP client
    Surf,
}

impl HttpClientCrate {
    /// Returns the crate name as it appears in Cargo.toml
    pub fn crate_name(&self) -> &'static str {
        match self {
            Self::Reqwest => "reqwest",
            Self::Hyper => "hyper",
            Self::Ureq => "ureq",
            Self::Surf => "surf",
        }
    }

    /// Returns patterns that indicate HTTP client usage
    pub fn usage_patterns(&self) -> &'static [&'static str] {
        match self {
            Self::Reqwest => &[
                "reqwest::",
                "Client::new",
                ".get(",
                ".post(",
                ".send().await",
            ],
            Self::Hyper => &["hyper::", "Request::builder"],
            Self::Ureq => &["ureq::", "Agent"],
            Self::Surf => &["surf::", "surf::get", "surf::post"],
        }
    }
}

/// Known cache crates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheCrate {
    /// Redis client
    Redis,
    /// Memcache client
    Memcache,
    /// Moka in-memory cache
    Moka,
    /// Cached macro library
    Cached,
}

impl CacheCrate {
    /// Returns the crate name as it appears in Cargo.toml
    pub fn crate_name(&self) -> &'static str {
        match self {
            Self::Redis => "redis",
            Self::Memcache => "memcache",
            Self::Moka => "moka",
            Self::Cached => "cached",
        }
    }

    /// Returns patterns that indicate cache usage
    pub fn usage_patterns(&self) -> &'static [&'static str] {
        match self {
            Self::Redis => &["redis::", "Commands", "AsyncCommands", ".get(", ".set("],
            Self::Memcache => &["memcache::"],
            Self::Moka => &["moka::", "Cache::new"],
            Self::Cached => &["#[cached]", "cached::"],
        }
    }
}

/// Known web framework crates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameworkCrate {
    /// Axum web framework
    Axum,
    /// Actix-web framework
    ActixWeb,
    /// Rocket web framework
    Rocket,
    /// Tonic gRPC framework
    Tonic,
    /// Warp web framework
    Warp,
    /// Poem web framework
    Poem,
}

impl FrameworkCrate {
    /// Returns the crate name as it appears in Cargo.toml
    pub fn crate_name(&self) -> &'static str {
        match self {
            Self::Axum => "axum",
            Self::ActixWeb => "actix-web",
            Self::Rocket => "rocket",
            Self::Tonic => "tonic",
            Self::Warp => "warp",
            Self::Poem => "poem",
        }
    }

    /// Returns patterns that indicate framework usage for endpoint detection
    pub fn endpoint_patterns(&self) -> &'static [&'static str] {
        match self {
            Self::Axum => &["Router::new", ".route(", "routing::get", "routing::post"],
            Self::ActixWeb => &["#[get(", "#[post(", "web::get()", "HttpServer::new"],
            Self::Rocket => &["#[get(", "#[post(", "routes!", "rocket::build"],
            Self::Tonic => &["#[tonic::async_trait]", "tonic::Request", "Server::builder"],
            Self::Warp => &["warp::path!", "warp::get()", "warp::post()"],
            Self::Poem => &["#[handler]", "Route::new", "poem::get"],
        }
    }
}

/// Known observability crates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilityCrate {
    /// tracing ecosystem
    Tracing,
    /// OpenTelemetry
    OpenTelemetry,
    /// log crate
    Log,
    /// metrics crate
    Metrics,
    /// prometheus client
    Prometheus,
}

impl ObservabilityCrate {
    /// Returns the crate name as it appears in Cargo.toml
    pub fn crate_name(&self) -> &'static str {
        match self {
            Self::Tracing => "tracing",
            Self::OpenTelemetry => "opentelemetry",
            Self::Log => "log",
            Self::Metrics => "metrics",
            Self::Prometheus => "prometheus",
        }
    }
}

/// Analyzed project dependencies
#[derive(Debug, Default)]
pub struct ProjectDependencies {
    /// Detected database crates
    pub databases: HashSet<DatabaseCrate>,
    /// Detected HTTP client crates
    pub http_clients: HashSet<HttpClientCrate>,
    /// Detected cache crates
    pub caches: HashSet<CacheCrate>,
    /// Detected web framework crates
    pub frameworks: HashSet<FrameworkCrate>,
    /// Detected observability crates
    pub observability: HashSet<ObservabilityCrate>,
    /// All dependency names (for custom pattern matching)
    pub all_deps: HashSet<String>,
}

impl ProjectDependencies {
    /// Analyze dependencies from a Cargo.toml file
    ///
    /// # Arguments
    ///
    /// * `manifest_path` - Path to the project directory or Cargo.toml
    ///
    /// # Errors
    ///
    /// Returns an error if cargo metadata fails
    pub fn from_manifest(manifest_path: &Path) -> Result<Self> {
        let cargo_toml = if manifest_path.is_dir() {
            manifest_path.join("Cargo.toml")
        } else {
            manifest_path.to_path_buf()
        };

        if !cargo_toml.exists() {
            return Ok(Self::default());
        }

        let metadata = MetadataCommand::new()
            .manifest_path(&cargo_toml)
            .no_deps() // We only care about direct dependencies
            .exec()
            .map_err(|e| Error::Generic(format!("Failed to get cargo metadata: {e}")))?;

        Ok(Self::from_metadata(&metadata))
    }

    /// Create from cargo metadata
    fn from_metadata(metadata: &Metadata) -> Self {
        let mut deps = Self::default();

        // Get the root package dependencies
        if let Some(root) = metadata.root_package() {
            deps.analyze_package(root);
        }

        // Also check workspace members
        for pkg_id in &metadata.workspace_members {
            if let Some(pkg) = metadata.packages.iter().find(|p| &p.id == pkg_id) {
                deps.analyze_package(pkg);
            }
        }

        deps
    }

    /// Analyze a single package's dependencies
    fn analyze_package(&mut self, package: &Package) {
        for dep in &package.dependencies {
            let name = dep.name.as_str();
            self.all_deps.insert(name.to_string());

            // Check for database crates
            match name {
                "sqlx" => {
                    self.databases.insert(DatabaseCrate::Sqlx);
                }
                "diesel" => {
                    self.databases.insert(DatabaseCrate::Diesel);
                }
                "sea-orm" => {
                    self.databases.insert(DatabaseCrate::SeaOrm);
                }
                "tokio-postgres" => {
                    self.databases.insert(DatabaseCrate::TokioPostgres);
                }
                "rusqlite" => {
                    self.databases.insert(DatabaseCrate::Rusqlite);
                }
                "mongodb" => {
                    self.databases.insert(DatabaseCrate::MongoDb);
                }
                _ => {}
            }

            // Check for HTTP client crates
            match name {
                "reqwest" => {
                    self.http_clients.insert(HttpClientCrate::Reqwest);
                }
                "hyper" => {
                    self.http_clients.insert(HttpClientCrate::Hyper);
                }
                "ureq" => {
                    self.http_clients.insert(HttpClientCrate::Ureq);
                }
                "surf" => {
                    self.http_clients.insert(HttpClientCrate::Surf);
                }
                _ => {}
            }

            // Check for cache crates
            match name {
                "redis" => {
                    self.caches.insert(CacheCrate::Redis);
                }
                "memcache" => {
                    self.caches.insert(CacheCrate::Memcache);
                }
                "moka" => {
                    self.caches.insert(CacheCrate::Moka);
                }
                "cached" => {
                    self.caches.insert(CacheCrate::Cached);
                }
                _ => {}
            }

            // Check for framework crates
            match name {
                "axum" => {
                    self.frameworks.insert(FrameworkCrate::Axum);
                }
                "actix-web" => {
                    self.frameworks.insert(FrameworkCrate::ActixWeb);
                }
                "rocket" => {
                    self.frameworks.insert(FrameworkCrate::Rocket);
                }
                "tonic" => {
                    self.frameworks.insert(FrameworkCrate::Tonic);
                }
                "warp" => {
                    self.frameworks.insert(FrameworkCrate::Warp);
                }
                "poem" => {
                    self.frameworks.insert(FrameworkCrate::Poem);
                }
                _ => {}
            }

            // Check for observability crates
            match name {
                "tracing" | "tracing-subscriber" | "tracing-opentelemetry" => {
                    self.observability.insert(ObservabilityCrate::Tracing);
                }
                "opentelemetry" | "opentelemetry-jaeger" | "opentelemetry-otlp" => {
                    self.observability.insert(ObservabilityCrate::OpenTelemetry);
                }
                "log" | "env_logger" | "pretty_env_logger" => {
                    self.observability.insert(ObservabilityCrate::Log);
                }
                "metrics" | "metrics-exporter-prometheus" => {
                    self.observability.insert(ObservabilityCrate::Metrics);
                }
                "prometheus" | "prometheus-client" => {
                    self.observability.insert(ObservabilityCrate::Prometheus);
                }
                _ => {}
            }
        }
    }

    /// Check if the project uses any database crate
    pub fn has_database(&self) -> bool {
        !self.databases.is_empty()
    }

    /// Check if the project uses any HTTP client
    pub fn has_http_client(&self) -> bool {
        !self.http_clients.is_empty()
    }

    /// Check if the project uses any cache
    pub fn has_cache(&self) -> bool {
        !self.caches.is_empty()
    }

    /// Check if the project uses any observability crate
    pub fn has_observability(&self) -> bool {
        !self.observability.is_empty()
    }

    /// Check if tracing is already set up
    pub fn has_tracing(&self) -> bool {
        self.observability.contains(&ObservabilityCrate::Tracing)
    }

    /// Get a summary of the project's technology stack
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.frameworks.is_empty() {
            let frameworks: Vec<_> = self.frameworks.iter().map(|f| f.crate_name()).collect();
            parts.push(format!("Frameworks: {}", frameworks.join(", ")));
        }

        if !self.databases.is_empty() {
            let dbs: Vec<_> = self.databases.iter().map(|d| d.crate_name()).collect();
            parts.push(format!("Databases: {}", dbs.join(", ")));
        }

        if !self.http_clients.is_empty() {
            let clients: Vec<_> = self.http_clients.iter().map(|c| c.crate_name()).collect();
            parts.push(format!("HTTP Clients: {}", clients.join(", ")));
        }

        if !self.caches.is_empty() {
            let caches: Vec<_> = self.caches.iter().map(|c| c.crate_name()).collect();
            parts.push(format!("Caches: {}", caches.join(", ")));
        }

        if !self.observability.is_empty() {
            let obs: Vec<_> = self.observability.iter().map(|o| o.crate_name()).collect();
            parts.push(format!("Observability: {}", obs.join(", ")));
        }

        if parts.is_empty() {
            "No recognized infrastructure dependencies".to_string()
        } else {
            parts.join("\n")
        }
    }
}

/// Detection context based on project dependencies
///
/// This provides hints for smarter pattern matching based on
/// what crates the project actually uses.
#[derive(Debug)]
pub struct DetectionContext {
    /// Project dependencies
    pub deps: ProjectDependencies,
    /// Priority boost for database patterns (0.0 - 1.0)
    pub db_priority: f64,
    /// Priority boost for HTTP client patterns (0.0 - 1.0)
    pub http_priority: f64,
    /// Priority boost for cache patterns (0.0 - 1.0)
    pub cache_priority: f64,
}

impl DetectionContext {
    /// Create a detection context from project dependencies
    pub fn from_deps(deps: ProjectDependencies) -> Self {
        let db_priority = if deps.has_database() { 0.9 } else { 0.3 };
        let http_priority = if deps.has_http_client() { 0.85 } else { 0.3 };
        let cache_priority = if deps.has_cache() { 0.8 } else { 0.2 };

        Self {
            deps,
            db_priority,
            http_priority,
            cache_priority,
        }
    }

    /// Check if a function name likely matches a database operation
    /// based on the project's dependencies
    pub fn is_likely_db_operation(&self, name: &str) -> bool {
        if !self.deps.has_database() {
            return false;
        }

        let lower = name.to_lowercase();
        let db_patterns = [
            "query",
            "execute",
            "fetch",
            "insert",
            "update",
            "delete",
            "select",
            "transaction",
            "commit",
            "rollback",
            "pool",
        ];
        db_patterns.iter().any(|p| lower.contains(p))
    }

    /// Check if a function name likely matches an HTTP client operation
    /// based on the project's dependencies
    pub fn is_likely_http_call(&self, name: &str) -> bool {
        if !self.deps.has_http_client() {
            return false;
        }

        let lower = name.to_lowercase();
        // Be more specific to avoid false positives like "get_user"
        let http_patterns = [
            "send_request",
            "http_get",
            "http_post",
            "call_api",
            "fetch_from",
            "remote_",
            "_client",
            "api_call",
        ];
        http_patterns.iter().any(|p| lower.contains(p))
    }

    /// Check if a function name likely matches a cache operation
    /// based on the project's dependencies
    pub fn is_likely_cache_operation(&self, name: &str) -> bool {
        if !self.deps.has_cache() {
            return false;
        }

        let lower = name.to_lowercase();
        let cache_patterns = [
            "cache_get",
            "cache_set",
            "get_cached",
            "set_cache",
            "invalidate",
            "cache_",
            "_cache",
        ];
        cache_patterns.iter().any(|p| lower.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_crate_patterns() {
        let sqlx = DatabaseCrate::Sqlx;
        assert_eq!(sqlx.crate_name(), "sqlx");
        assert!(!sqlx.usage_patterns().is_empty());
    }

    #[test]
    fn test_http_client_patterns() {
        let reqwest = HttpClientCrate::Reqwest;
        assert_eq!(reqwest.crate_name(), "reqwest");
        assert!(!reqwest.usage_patterns().is_empty());
    }

    #[test]
    fn test_detection_context_no_deps() {
        let deps = ProjectDependencies::default();
        let ctx = DetectionContext::from_deps(deps);

        // Without dependencies, should not match
        assert!(!ctx.is_likely_db_operation("query_users"));
        assert!(!ctx.is_likely_http_call("send_request"));
        assert!(!ctx.is_likely_cache_operation("cache_get"));
    }

    #[test]
    fn test_detection_context_with_db() {
        let mut deps = ProjectDependencies::default();
        deps.databases.insert(DatabaseCrate::Sqlx);
        let ctx = DetectionContext::from_deps(deps);

        // With sqlx dependency, should match DB patterns
        assert!(ctx.is_likely_db_operation("query_users"));
        assert!(ctx.is_likely_db_operation("fetch_orders"));
        assert!(!ctx.is_likely_db_operation("get_user")); // Too generic
    }

    #[test]
    fn test_project_dependencies_summary() {
        let mut deps = ProjectDependencies::default();
        deps.frameworks.insert(FrameworkCrate::Axum);
        deps.databases.insert(DatabaseCrate::Sqlx);
        deps.http_clients.insert(HttpClientCrate::Reqwest);

        let summary = deps.summary();
        assert!(summary.contains("axum"));
        assert!(summary.contains("sqlx"));
        assert!(summary.contains("reqwest"));
    }
}
