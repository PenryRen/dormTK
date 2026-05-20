use serde::Deserialize;
use std::{env, fs, net::SocketAddr, path::PathBuf};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    app_host: Option<String>,
    app_port: Option<u16>,
    db_host: Option<String>,
    db_port: Option<u16>,
    db_name: Option<String>,
    db_user: Option<String>,
    db_password: Option<String>,
    jwt_secret: Option<String>,
    jwt_expiration_seconds: Option<u64>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let raw = load_raw_config()?;

        let server = ServerConfig {
            host: env_or("APP_HOST", raw.app_host, "0.0.0.0"),
            port: env_or_parse("APP_PORT", raw.app_port, 8080)?,
        };

        let database = DatabaseConfig {
            host: env_or("DB_HOST", raw.db_host, "db"),
            port: env_or_parse("DB_PORT", raw.db_port, 5432)?,
            name: env_or("DB_NAME", raw.db_name, "dormtk"),
            user: env_or("DB_USER", raw.db_user, "dormtk"),
            password: env_or("DB_PASSWORD", raw.db_password, "dormtk"),
        };

        let jwt = JwtConfig {
            secret: env_or("JWT_SECRET", raw.jwt_secret, "dev-only-change-me"),
            expiration_seconds: env_or_parse(
                "JWT_EXPIRATION_SECONDS",
                raw.jwt_expiration_seconds,
                86_400,
            )?,
        };

        Ok(Self {
            server,
            database,
            jwt,
        })
    }
}

impl ServerConfig {
    pub fn socket_addr(&self) -> Result<SocketAddr, ConfigError> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|source| ConfigError::InvalidSocketAddr {
                value: format!("{}:{}", self.host, self.port),
                source,
            })
    }
}

impl DatabaseConfig {
    pub fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.name
        )
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseToml {
        path: PathBuf,
        source: toml::de::Error,
    },
    ParseEnv {
        name: &'static str,
        value: String,
    },
    InvalidSocketAddr {
        value: String,
        source: std::net::AddrParseError,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "failed to read config {}: {}", path.display(), source)
            }
            Self::ParseToml { path, source } => {
                write!(f, "failed to parse config {}: {}", path.display(), source)
            }
            Self::ParseEnv { name, value } => {
                write!(f, "failed to parse env {}={}", name, value)
            }
            Self::InvalidSocketAddr { value, source } => {
                write!(f, "invalid socket addr {}: {}", value, source)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

fn load_raw_config() -> Result<RawConfig, ConfigError> {
    let path = config_path();
    let content = fs::read_to_string(&path).map_err(|source| ConfigError::Read {
        path: path.clone(),
        source,
    })?;

    toml::from_str(&content).map_err(|source| ConfigError::ParseToml { path, source })
}

fn config_path() -> PathBuf {
    if let Ok(path) = env::var("DORMTK_CONFIG") {
        return PathBuf::from(path);
    }

    let local = PathBuf::from("config/dev.toml");
    if local.exists() {
        return local;
    }

    PathBuf::from("backend/config/dev.toml")
}

fn env_or(name: &'static str, file_value: Option<String>, default: &'static str) -> String {
    env::var(name)
        .ok()
        .or(file_value)
        .unwrap_or_else(|| default.to_owned())
}

fn env_or_parse<T>(name: &'static str, file_value: Option<T>, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
{
    match env::var(name) {
        Ok(value) => value
            .parse()
            .map_err(|_| ConfigError::ParseEnv { name, value }),
        Err(_) => Ok(file_value.unwrap_or(default)),
    }
}
