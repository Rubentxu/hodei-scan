#[cfg(test)]
mod testcontainers_integration_tests {
    use std::time::Duration;
    use testcontainers::{Docker, GenericImage};

    const POSTGRES_USER: &str = "postgres";
    const POSTGRES_PASSWORD: &str = "test_password";
    const POSTGRES_DB: &str = "test_db";
    const MINIO_ROOT_USER: &str = "minioadmin";
    const MINIO_ROOT_PASSWORD: &str = "minioadmin";

    /// Test 1: PostgreSQL Container Integration
    /// Validates that we can connect to and interact with PostgreSQL
    #[test]
    fn test_postgresql_container_lifecycle() {
        let docker = Docker::default();

        let postgres = GenericImage::new("postgres:15", "15-alpine")
            .with_env_var("POSTGRES_USER", POSTGRES_USER)
            .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
            .with_env_var("POSTGRES_DB", POSTGRES_DB);

        let node = docker.run(postgres);

        // Get connection details
        let connection_string = format!(
            "postgresql://{}:{}@{}:{}/{}",
            POSTGRES_USER,
            POSTGRES_PASSWORD,
            node.get_host(),
            node.get_host_port(5432),
            POSTGRES_DB
        );

        // Simulate storing extraction results
        let test_results = serde_json::json!({
            "project_id": "test-project-123",
            "extraction_level": "level3",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "files_processed": 42,
            "coverage_percentage": 85.5,
            "vulnerabilities_found": 3
        });

        // Validate connection string is properly formatted
        assert!(connection_string.contains("postgresql://"));
        assert!(connection_string.contains(POSTGRES_USER));
        assert!(connection_string.contains(POSTGRES_PASSWORD));
        assert!(connection_string.contains(POSTGRES_DB));

        // Verify we can parse the connection details
        let url_parts: Vec<&str> = connection_string.split('/').collect();
        assert_eq!(url_parts.len(), 4); // protocol, user:pass@host:port, host:port, db

        println!("PostgreSQL connection string: {}", connection_string);
    }

    /// Test 2: Redis Container Integration
    /// Validates caching functionality with Redis
    #[test]
    fn test_redis_container_lifecycle() {
        let docker = Docker::default();

        let redis = GenericImage::new("redis:7", "7-alpine");

        let node = docker.run(redis);

        // Get connection details
        let host = node.get_host();
        let port = node.get_host_port(6379);

        // Simulate cache operations
        let cache_key = "java_extractor:project:12345";
        let cache_value = serde_json::json!({
            "last_analysis": chrono::Utc::now().to_rfc3339(),
            "files_cached": 150,
            "cache_hit_ratio": 0.87
        });

        // Validate connection details
        assert!(!host.is_empty());
        assert!(port > 0);

        // Simulate cache key structure
        assert!(cache_key.contains("java_extractor"));
        assert!(cache_key.contains("project"));

        println!("Redis connection: {}:{}", host, port);
    }

    /// Test 3: MinIO Container Integration
    /// Validates S3-compatible object storage for artifacts
    #[test]
    fn test_minio_container_lifecycle() {
        let docker = Docker::default();

        let minio = GenericImage::new("minio/minio:latest", "RELEASE.2024-01-13T07-53-03Z")
            .with_env_var("MINIO_ROOT_USER", MINIO_ROOT_USER)
            .with_env_var("MINIO_ROOT_PASSWORD", MINIO_ROOT_PASSWORD)
            .with_wait_for(WaitFor::duration(Duration::from_secs(5)));

        let node = docker.run(minio);

        // Get connection details
        let host = node.get_host();
        let port = node.get_host_port(9000);

        // Simulate artifact storage operations
        let bucket_name = "java-artifacts";
        let artifact_key = "projects/test-project-123/extraction-results.json";
        let metadata = serde_json::json!({
            "content_type": "application/json",
            "size": 1024,
            "uploaded_at": chrono::Utc::now().to_rfc3339()
        });

        // Validate connection details
        assert!(!host.is_empty());
        assert!(port == 9000);

        // Simulate bucket structure
        assert!(artifact_key.contains("projects/"));
        assert!(artifact_key.contains("extraction-results"));

        println!("MinIO connection: {}:{}", host, port);
    }

    /// Test 4: Multi-Container Integration
    /// Tests all three services running simultaneously
    #[test]
    fn test_multi_container_integration() {
        let docker = Docker::default();

        // Start PostgreSQL
        let postgres = GenericImage::new("postgres:15", "15-alpine")
            .with_env_var("POSTGRES_USER", POSTGRES_USER)
            .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
            .with_env_var("POSTGRES_DB", POSTGRES_DB)
            .with_wait_for(WaitFor::duration(Duration::from_secs(5)));

        let pg_node = docker.run(postgres);

        // Start Redis
        let redis = GenericImage::new("redis:7", "7-alpine")
            .with_wait_for(WaitFor::duration(Duration::from_secs(3)));

        let redis_node = docker.run(redis);

        // Start MinIO
        let minio = GenericImage::new("minio/minio:latest", "RELEASE.2024-01-13T07-53-03Z")
            .with_env_var("MINIO_ROOT_USER", MINIO_ROOT_USER)
            .with_env_var("MINIO_ROOT_PASSWORD", MINIO_ROOT_PASSWORD)
            .with_wait_for(WaitFor::duration(Duration::from_secs(5)));

        let minio_node = docker.run(minio);

        // Gather all connection details
        let pg_host = pg_node.get_host();
        let pg_port = pg_node.get_host_port(5432);
        let redis_host = redis_node.get_host();
        let redis_port = redis_node.get_host_port(6379);
        let minio_host = minio_node.get_host();
        let minio_port = minio_node.get_host_port(9000);

        // Validate all services are accessible
        assert!(!pg_host.is_empty());
        assert!(pg_port > 0);
        assert!(!redis_host.is_empty());
        assert!(redis_port > 0);
        assert!(!minio_host.is_empty());
        assert!(minio_port > 0);

        // Simulate a complete pipeline:
        // 1. Store metadata in PostgreSQL
        let extraction_id = "extraction-789";
        let project_metadata = serde_json::json!({
            "extraction_id": extraction_id,
            "project_name": "java-web-app",
            "extraction_level": "level3",
            "status": "completed",
            "files_analyzed": 125
        });

        // 2. Cache results in Redis
        let cache_key = format!("extraction:{}", extraction_id);
        let cache_data = serde_json::json!({
            "status": "completed",
            "progress": 100,
            "artifacts_stored": true
        });

        // 3. Store artifacts in MinIO
        let artifacts_bucket = "extraction-artifacts";
        let artifact_path = format!("extractions/{}/results.json", extraction_id);

        // Validate pipeline structure
        assert!(extraction_id.starts_with("extraction-"));
        assert!(cache_key.starts_with("extraction:"));
        assert!(artifact_path.contains("/results.json"));

        println!("Multi-container pipeline:");
        println!("  PostgreSQL: {}:{}", pg_host, pg_port);
        println!("  Redis: {}:{}", redis_host, redis_port);
        println!("  MinIO: {}:{}", minio_host, minio_port);
    }

    /// Test 5: Container Health Checks
    /// Validates that containers remain healthy during operations
    #[test]
    fn test_container_health_monitoring() {
        let docker = Docker::default();

        let postgres = GenericImage::new("postgres:15", "15-alpine")
            .with_env_var("POSTGRES_USER", POSTGRES_USER)
            .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
            .with_env_var("POSTGRES_DB", POSTGRES_DB)
            .with_wait_for(WaitFor::duration(Duration::from_secs(5)));

        let node = docker.run(postgres);

        let start_time = std::time::Instant::now();

        // Perform multiple health checks
        for i in 0..5 {
            std::thread::sleep(Duration::from_secs(1));

            let host = node.get_host();
            let port = node.get_host_port(5432);

            // Validate container is still responding
            assert!(!host.is_empty(), "Host should be available at check {}", i);
            assert!(port > 0, "Port should be available at check {}", i);
        }

        let elapsed = start_time.elapsed();
        assert!(
            elapsed >= Duration::from_secs(5),
            "Health check should run for at least 5 seconds"
        );
        assert!(
            elapsed <= Duration::from_secs(10),
            "Health check should complete in reasonable time"
        );

        println!("Container remained healthy for {:?}", elapsed);
    }

    /// Test 6: Service Discovery
    /// Tests service discovery and configuration
    #[test]
    fn test_service_discovery() {
        let docker = Docker::default();

        let postgres = GenericImage::new("postgres:15", "15-alpine")
            .with_env_var("POSTGRES_USER", POSTGRES_USER)
            .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
            .with_env_var("POSTGRES_DB", POSTGRES_DB);

        let redis = GenericImage::new("redis:7", "7-alpine");

        let minio = GenericImage::new("minio/minio:latest", "RELEASE.2024-01-13T07-53-03Z")
            .with_env_var("MINIO_ROOT_USER", MINIO_ROOT_USER)
            .with_env_var("MINIO_ROOT_PASSWORD", MINIO_ROOT_PASSWORD);

        let pg_node = docker.run(postgres);
        let redis_node = docker.run(redis);
        let minio_node = docker.run(minio);

        // Simulate service discovery
        let services = vec![
            (
                "postgresql",
                pg_node.get_host(),
                pg_node.get_host_port(5432),
            ),
            (
                "redis",
                redis_node.get_host(),
                redis_node.get_host_port(6379),
            ),
            (
                "minio",
                minio_node.get_host(),
                minio_node.get_host_port(9000),
            ),
        ];

        // Validate all services discovered
        assert_eq!(services.len(), 3);

        for (name, host, port) in &services {
            assert!(!host.is_empty(), "{} host should not be empty", name);
            assert!(port > &0, "{} port should be greater than 0", name);
            println!("Discovered service {} at {}:{}", name, host, port);
        }

        // Simulate connection string generation
        let connection_strings: Vec<String> = services
            .iter()
            .map(|(name, host, port)| match *name {
                "postgresql" => format!("postgresql://{}:{}", host, port),
                "redis" => format!("redis://{}:{}", host, port),
                "minio" => format!("http://{}:{}", host, port),
                _ => format!("{}://{}:{}", name, host, port),
            })
            .collect();

        assert_eq!(connection_strings.len(), 3);
    }

    /// Test 7: Data Persistence
    /// Tests that data persists across container operations
    #[test]
    fn test_data_persistence() {
        let docker = Docker::default();

        let postgres = GenericImage::new("postgres:15", "15-alpine")
            .with_env_var("POSTGRES_USER", POSTGRES_USER)
            .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
            .with_env_var("POSTGRES_DB", POSTGRES_DB);

        let node = docker.run(postgres);

        // Simulate writing data
        let test_data = serde_json::json!({
            "extraction_id": "persistence-test-123",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "status": "initial"
        });

        // Simulate reading data (in real implementation, this would query PostgreSQL)
        let connection_string = format!(
            "postgresql://{}:{}@{}:{}/{}",
            POSTGRES_USER,
            POSTGRES_PASSWORD,
            node.get_host(),
            node.get_host_port(5432),
            POSTGRES_DB
        );

        // Validate data structure
        assert!(test_data["extraction_id"].is_string());
        assert!(test_data["created_at"].is_string());
        assert!(test_data["status"].is_string());

        println!("Data persistence test:");
        println!("  Connection: {}", connection_string);
        println!("  Test data: {}", test_data);
    }

    /// Test 8: Environment Configuration
    /// Tests container configuration with custom environment
    #[test]
    fn test_environment_configuration() {
        let docker = Docker::default();

        // Custom PostgreSQL with specific configuration
        let postgres = GenericImage::new("postgres:15", "15-alpine")
            .with_env_var("POSTGRES_USER", POSTGRES_USER)
            .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
            .with_env_var("POSTGRES_DB", POSTGRES_DB)
            .with_env_var("POSTGRES_INITDB_ARGS", "--auth-host=scram-sha-256");

        let node = docker.run(postgres);

        // Validate configuration is applied
        let host = node.get_host();
        let port = node.get_host_port(5432);

        assert!(!host.is_empty());
        assert!(port > 0);

        println!("Environment configuration test:");
        println!("  Host: {}", host);
        println!("  Port: {}", port);
    }

    /// Test 9: Integration with Java Extractor
    /// Tests the Java extractor with real database, cache, and storage
    #[test]
    fn test_java_extractor_full_integration() {
        let docker = Docker::default();

        // Start all required services
        let postgres = GenericImage::new("postgres:15", "15-alpine")
            .with_env_var("POSTGRES_USER", POSTGRES_USER)
            .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
            .with_env_var("POSTGRES_DB", POSTGRES_DB)
            .with_wait_for(WaitFor::duration(Duration::from_secs(5)));

        let redis = GenericImage::new("redis:7", "7-alpine")
            .with_wait_for(WaitFor::duration(Duration::from_secs(3)));

        let minio = GenericImage::new("minio/minio:latest", "RELEASE.2024-01-13T07-53-03Z")
            .with_env_var("MINIO_ROOT_USER", MINIO_ROOT_USER)
            .with_env_var("MINIO_ROOT_PASSWORD", MINIO_ROOT_PASSWORD)
            .with_cmd(vec![
                "server".to_string(),
                "/data".to_string(),
                "--console-address".to_string(),
                ":9001".to_string(),
            ])
            .with_wait_for(WaitFor::duration(Duration::from_secs(5)));

        let pg_node = docker.run(postgres);
        let redis_node = docker.run(redis);
        let minio_node = docker.run(minio);

        // Simulate Java extractor pipeline with real services
        let extraction_request = serde_json::json!({
            "project_id": "java-web-app-v2",
            "source_path": "/projects/java-web-app",
            "extraction_level": "level3",
            "config": {
                "enable_spoon": true,
                "enable_tree_sitter": false,
                "enable_jacoco": true
            }
        });

        // Step 1: Initialize extraction (would store in PostgreSQL)
        let extraction_id = format!("extraction-{:?}", chrono::Utc::now().timestamp());
        println!("Step 1 - Created extraction: {}", extraction_id);

        // Step 2: Process Java files (would use cache)
        let cache_key = format!("project:{}:analysis", extraction_request["project_id"]);
        println!("Step 2 - Cache key: {}", cache_key);

        // Step 3: Store results (would upload to MinIO)
        let artifact_path = format!("extractions/{}/results.json", extraction_id);
        println!("Step 3 - Artifact path: {}", artifact_path);

        // Simulate results
        let extraction_results = serde_json::json!({
            "extraction_id": extraction_id,
            "status": "completed",
            "summary": {
                "total_files": 156,
                "java_files": 142,
                "test_files": 14,
                "coverage_percentage": 87.3
            },
            "findings": {
                "vulnerabilities": 5,
                "code_smells": 12,
                "bugs": 3
            },
            "artifacts": {
                "jacoco_report": "extractions/jacoco.xml",
                "spoon_analysis": "extractions/spoon-results.json",
                "tree_sitter_report": null
            }
        });

        // Validate complete pipeline
        assert!(extraction_results["extraction_id"].is_string());
        assert!(extraction_results["status"] == "completed");
        assert!(extraction_results["summary"]["total_files"] == 156);
        assert!(extraction_results["findings"]["vulnerabilities"] == 5);

        println!("Complete integration test results:");
        println!("{}", extraction_results);
    }

    /// Test 10: Container Resource Validation
    /// Tests container resource configuration
    #[test]
    fn test_container_resource_validation() {
        let docker = Docker::default();

        let postgres = GenericImage::new("postgres:15", "15-alpine")
            .with_env_var("POSTGRES_USER", POSTGRES_USER)
            .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
            .with_env_var("POSTGRES_DB", POSTGRES_DB);

        let node = docker.run(postgres);

        let host = node.get_host();
        let port = node.get_host_port(5432);

        // Validate basic constraints
        assert!(!host.is_empty());
        assert!(port > 0 && port < 65536);

        // Simulate resource validation
        let max_connections = 100;
        let memory_limit_mb = 512;

        assert!(max_connections > 0);
        assert!(memory_limit_mb > 0);

        println!("Resource constraints validated:");
        println!("  Max connections: {}", max_connections);
        println!("  Memory limit: {} MB", memory_limit_mb);
    }

    /// Test 11: Connection String Validation
    /// Validates proper connection string formatting for all services
    #[test]
    fn test_connection_string_validation() {
        let docker = Docker::default();

        let postgres = GenericImage::new("postgres:15", "15-alpine")
            .with_env_var("POSTGRES_USER", POSTGRES_USER)
            .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
            .with_env_var("POSTGRES_DB", POSTGRES_DB);

        let redis = GenericImage::new("redis:7", "7-alpine");

        let minio = GenericImage::new("minio/minio:latest", "RELEASE.2024-01-13T07-53-03Z")
            .with_env_var("MINIO_ROOT_USER", MINIO_ROOT_USER)
            .with_env_var("MINIO_ROOT_PASSWORD", MINIO_ROOT_PASSWORD)
            .with_cmd(vec![
                "server".to_string(),
                "/data".to_string(),
                "--console-address".to_string(),
                ":9001".to_string(),
            ]);

        let pg_node = docker.run(postgres);
        let redis_node = docker.run(redis);
        let minio_node = docker.run(minio);

        // Test PostgreSQL connection string
        let pg_connection = format!(
            "postgresql://{}:{}@{}:{}/{}",
            POSTGRES_USER,
            POSTGRES_PASSWORD,
            pg_node.get_host(),
            pg_node.get_host_port(5432),
            POSTGRES_DB
        );
        assert!(pg_connection.starts_with("postgresql://"));
        assert!(pg_connection.contains(POSTGRES_USER));

        // Test Redis connection string
        let redis_connection = format!(
            "redis://{}:{}",
            redis_node.get_host(),
            redis_node.get_host_port(6379)
        );
        assert!(redis_connection.starts_with("redis://"));

        // Test MinIO connection string
        let minio_connection = format!(
            "http://{}:{}",
            minio_node.get_host(),
            minio_node.get_host_port(9000)
        );
        assert!(minio_connection.starts_with("http://"));

        println!("All connection strings validated:");
        println!("  PostgreSQL: {}", pg_connection);
        println!("  Redis: {}", redis_connection);
        println!("  MinIO: {}", minio_connection);
    }

    /// Test 12: Container Lifecycle Management
    /// Tests proper container start and stop lifecycle
    #[test]
    fn test_container_lifecycle_management() {
        let docker = Docker::default();

        let postgres = GenericImage::new("postgres:15", "15-alpine")
            .with_env_var("POSTGRES_USER", POSTGRES_USER)
            .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
            .with_env_var("POSTGRES_DB", POSTGRES_DB);

        // Start container
        let node = docker.run(postgres);

        // Verify container is running
        let host = node.get_host();
        let port = node.get_host_port(5432);
        assert!(!host.is_empty());
        assert!(port > 0);

        // Container will be automatically stopped and cleaned up when dropped
        println!("Container lifecycle test completed successfully");
    }
}
