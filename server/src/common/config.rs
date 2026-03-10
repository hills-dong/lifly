use std::env;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

/// Application-wide configuration populated from environment variables.
///
/// Call [`AppConfig::from_env`] once at startup (after `dotenvy::dotenv()`).
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Postgres connection string.
    pub database_url: String,

    /// Directory where uploaded files are stored.
    pub file_storage_path: PathBuf,

    /// API key sent to the remote LLM service.
    pub llm_api_key: String,

    /// Base URL of the remote LLM service.
    pub llm_api_url: String,

    /// Secret used to sign and verify JWT tokens.
    pub jwt_secret: String,

    /// Host the HTTP server binds to.
    pub server_host: IpAddr,

    /// Port the HTTP server listens on.
    pub server_port: u16,
}

impl AppConfig {
    /// Read configuration from environment variables.
    ///
    /// # Panics
    ///
    /// Panics if any required variable (`DATABASE_URL`, `LLM_API_KEY`,
    /// `LLM_API_URL`, `JWT_SECRET`) is missing.
    pub fn from_env() -> Self {
        Self {
            database_url: required("DATABASE_URL"),
            file_storage_path: PathBuf::from(
                env::var("FILE_STORAGE_PATH").unwrap_or_else(|_| "/data/files".to_string()),
            ),
            llm_api_key: required("LLM_API_KEY"),
            llm_api_url: required("LLM_API_URL"),
            jwt_secret: required("JWT_SECRET"),
            server_host: IpAddr::from_str(
                &env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            )
            .expect("SERVER_HOST must be a valid IP address"),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid u16"),
        }
    }

    /// Convenience method returning the socket address for `TcpListener::bind`.
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.server_host, self.server_port)
    }
}

/// Read a required environment variable, panicking with a clear message if absent.
fn required(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("environment variable {name} is required"))
}
