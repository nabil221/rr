use std::{env, net::SocketAddr};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ApiConfig {
    pub(crate) bind_address: SocketAddr,
}

impl ApiConfig {
    pub(crate) fn from_env() -> Result<Self, String> {
        Self::from_values(env::var("API_BIND_ADDRESS").ok())
    }

    fn from_values(bind_address_value: Option<String>) -> Result<Self, String> {
        let bind_address =
            read_with_default(bind_address_value, "API_BIND_ADDRESS", "127.0.0.1:3000")?
                .parse::<SocketAddr>()
                .map_err(|_| "API_BIND_ADDRESS must be a valid socket address".to_owned())?;

        if !bind_address.ip().is_loopback() {
            return Err("API_BIND_ADDRESS must use a loopback address".to_owned());
        }

        Ok(Self { bind_address })
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
    fn defaults_to_loopback_without_database_configuration() {
        let config = ApiConfig::from_values(None).expect("default config is valid");
        assert_eq!(
            config.bind_address,
            "127.0.0.1:3000".parse().expect("valid address")
        );
    }

    #[test]
    fn rejects_non_loopback_bind_address() {
        let error = ApiConfig::from_values(Some("0.0.0.0:3000".to_owned()))
            .expect_err("public bind addresses are not allowed");
        assert_eq!(error, "API_BIND_ADDRESS must use a loopback address");
    }
}
