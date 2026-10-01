use std::{env, net::SocketAddr};

#[derive(PartialEq, Eq)]
pub(crate) struct ApiConfig {
    pub(crate) bind_address: SocketAddr,
    pub(crate) database_url: String,
}

impl ApiConfig {
    pub(crate) fn from_env() -> Result<Self, String> {
        Self::from_values(
            env::var("API_BIND_ADDRESS").ok(),
            env::var("DATABASE_URL").ok(),
        )
    }

    fn from_values(
        bind_address_value: Option<String>,
        database_url_value: Option<String>,
    ) -> Result<Self, String> {
        let bind_address =
            read_with_default(bind_address_value, "API_BIND_ADDRESS", "127.0.0.1:3000")?
                .parse::<SocketAddr>()
                .map_err(|_| "API_BIND_ADDRESS must be a valid socket address".to_owned())?;

        if !bind_address.ip().is_loopback() {
            return Err("API_BIND_ADDRESS must use a loopback address".to_owned());
        }

        let database_url = read_with_default(
            database_url_value,
            "DATABASE_URL",
            local_stack_proof_persistence_postgres::LOCAL_DATABASE_URL,
        )?;
        local_stack_proof_persistence_postgres::validate_database_url(&database_url).map_err(
            |_| {
                "DATABASE_URL must target the project database at 127.0.0.1:54329/local_stack_proof"
                    .to_owned()
            },
        )?;
        Ok(Self {
            bind_address,
            database_url,
        })
    }
}

fn read_with_default(value: Option<String>, name: &str, default: &str) -> Result<String, String> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(value),
        Some(_) => Err(format!("{name} cannot be empty")),
        None => Ok(default.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::ApiConfig;

    #[test]
    fn defaults_to_loopback_and_project_database() {
        let config = ApiConfig::from_values(None, None)
            .unwrap_or_else(|_| panic!("default config is valid"));
        assert_eq!(
            config.bind_address,
            "127.0.0.1:3000".parse().expect("valid address")
        );
    }

    #[test]
    fn rejects_non_loopback_bind_address() {
        let error = ApiConfig::from_values(Some("0.0.0.0:3000".to_owned()), None)
            .err()
            .expect("public bind addresses are not allowed");
        assert_eq!(error, "API_BIND_ADDRESS must use a loopback address");
    }

    #[test]
    fn rejects_remote_wrong_and_empty_database_urls_without_echoing_credentials() {
        for value in [
            "",
            "postgresql://secret:password@example.com/local_stack_proof",
            "postgresql://127.0.0.1:54329/postgres",
        ] {
            let error = ApiConfig::from_values(None, Some(value.to_owned()))
                .err()
                .unwrap();
            assert!(!error.contains("password"));
            assert!(!error.contains("secret"));
        }
    }
}
