/// gRPC server implementation for hodei-server
pub mod server;

pub use server::HodeiGrpcServer;

// Public re-export of generated proto types
pub mod proto {
    // This will be populated by tonic-build
    // For now, provide placeholder types
    
    /// Placeholder types until proto is generated
    pub mod analysis {
        use tonic::Status;

        #[derive(Debug)]
        pub struct Metadata {
            pub build_url: String,
            pub author: String,
            pub ci_run_id: String,
            pub scan_duration_ms: u64,
            pub rule_version: String,
        }

        pub type Analysis = ();
        pub type Finding = ();
        pub type FindingLocation = ();
        
        pub const Severity_CRITICAL: i32 = 0;
        pub const Severity_MAJOR: i32 = 1;
        pub const Severity_MINOR: i32 = 2;
        pub const Severity_INFO: i32 = 3;
        
        pub const TrendDirection_IMPROVING: i32 = 0;
        pub const TrendDirection_DEGRADING: i32 = 1;
        pub const TrendDirection_STABLE: i32 = 2;
    }

    pub mod health_server {
        use tonic::Status;

        #[tonic::async_trait]
        pub trait Health: Send + Sync + 'static {
            async fn check(
                &self,
                request: tonic::Request<()>,
            ) -> Result<tonic::Response<HealthCheckResponse>, Status>;
        }

        #[derive(Debug)]
        pub struct HealthServer<T> {
            inner: T,
        }

        impl<T: Health> HealthServer<T> {
            pub fn new(inner: T) -> Self {
                Self { inner }
            }
        }

        #[derive(Debug)]
        pub struct HealthCheckRequest {
            pub service: String,
        }

        #[derive(Debug)]
        pub struct HealthCheckResponse {
            pub status: i32,
            pub version: String,
            pub message: String,
        }
    }
}
