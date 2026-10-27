use once_cell::sync::Lazy;

pub struct Config {
    pub api_key: String,

    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_db: String,

    pub postgres_pool_max_connections: u32,
    pub postgres_pool_min_connections: u32,
    pub postgres_pool_acquire_timeout_sec: u64,

    pub postgres_statement_timeout_ms: u64,
    pub postgres_lock_timeout_ms: u64,

    pub application_name: String,

    pub sentry_dsn: Option<String>,
}

fn get_env(env: &'static str) -> String {
    std::env::var(env).unwrap_or_else(|_| panic!("Cannot get the {} env variable", env))
}

fn get_env_or(env: &'static str, default: &'static str) -> String {
    std::env::var(env).unwrap_or_else(|_| default.to_string())
}

/// Parses an optional numeric env variable; a set-but-invalid value is a startup error.
fn parse_env_or<T: std::str::FromStr>(env: &'static str, default: T) -> T {
    match std::env::var(env) {
        Ok(value) => value
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("Invalid {} env variable: {:?}", env, value)),
        Err(_) => default,
    }
}

impl Config {
    pub fn load() -> Config {
        let api_key = get_env("API_KEY");
        if api_key.trim().is_empty() {
            panic!("API_KEY env variable must not be empty");
        }

        let config = Config {
            api_key,

            postgres_user: get_env("POSTGRES_USER"),
            postgres_password: get_env("POSTGRES_PASSWORD"),
            postgres_host: get_env("POSTGRES_HOST"),
            postgres_port: get_env("POSTGRES_PORT")
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("Invalid POSTGRES_PORT (expected 1-65535)")),
            postgres_db: get_env("POSTGRES_DB"),

            postgres_pool_max_connections: parse_env_or("POSTGRES_POOL_MAX_CONNECTIONS", 10),
            postgres_pool_min_connections: parse_env_or("POSTGRES_POOL_MIN_CONNECTIONS", 2),
            postgres_pool_acquire_timeout_sec: parse_env_or("POSTGRES_POOL_ACQUIRE_TIMEOUT_SEC", 5),

            postgres_statement_timeout_ms: parse_env_or("POSTGRES_STATEMENT_TIMEOUT_MS", 5000),
            postgres_lock_timeout_ms: parse_env_or("POSTGRES_LOCK_TIMEOUT_MS", 3000),

            application_name: get_env_or("APPLICATION_NAME", "users_settings_server"),

            sentry_dsn: std::env::var("SENTRY_DSN").ok().filter(|s| !s.is_empty()),
        };

        if config.postgres_port == 0 {
            panic!("POSTGRES_PORT must be greater than 0");
        }
        if config.postgres_pool_max_connections == 0 {
            panic!("POSTGRES_POOL_MAX_CONNECTIONS must be greater than 0");
        }
        if config.postgres_pool_max_connections < config.postgres_pool_min_connections {
            panic!(
                "POSTGRES_POOL_MAX_CONNECTIONS ({}) must be >= POSTGRES_POOL_MIN_CONNECTIONS ({})",
                config.postgres_pool_max_connections, config.postgres_pool_min_connections
            );
        }
        if config.postgres_pool_acquire_timeout_sec == 0 {
            panic!("POSTGRES_POOL_ACQUIRE_TIMEOUT_SEC must be greater than 0");
        }

        config
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::load);
