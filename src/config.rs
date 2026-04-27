use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, dotenvy::Error> {
        #[cfg(not(test))]
        dotenvy::dotenv().ok();
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://user:pass@localhost/fast_tx".to_string()),
            server_addr: std::env::var("SERVER_ADDR")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_from_env_with_vars() {
        unsafe {
            env::set_var("DATABASE_URL", "postgres://test:test@localhost/test");
            env::set_var("SERVER_ADDR", "0.0.0.0");
            env::set_var("SERVER_PORT", "8080");
        }

        let config = Config::from_env().unwrap();

        assert_eq!(config.database_url, "postgres://test:test@localhost/test");
        assert_eq!(config.server_addr, "0.0.0.0");
        assert_eq!(config.server_port, 8080);

        // Clean up
        unsafe {
            env::remove_var("DATABASE_URL");
            env::remove_var("SERVER_ADDR");
            env::remove_var("SERVER_PORT");
        }
    }

    #[test]
    fn test_config_from_env_defaults() {
        // Ensure vars are not set
        unsafe {
            env::remove_var("DATABASE_URL");
            env::remove_var("SERVER_ADDR");
            env::remove_var("SERVER_PORT");
        }

        let config = Config::from_env().unwrap();

        assert_eq!(config.database_url, "postgres://user:pass@localhost/fast_tx");
        assert_eq!(config.server_addr, "127.0.0.1");
        assert_eq!(config.server_port, 3000);
    }

    #[test]
    fn test_config_from_env_invalid_port() {
        unsafe {
            env::set_var("SERVER_PORT", "invalid");
        }

        let config = Config::from_env().unwrap();

        // Should default to 3000
        assert_eq!(config.server_port, 3000);

        unsafe {
            env::remove_var("SERVER_PORT");
        }
    }
}