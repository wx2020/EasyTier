use std::net::IpAddr;
use std::path::Path;

use anyhow::Context as _;
use serde::Deserialize;

/// The TOML representation of easytier-web's startup configuration.
///
/// Every field is optional so a file can contain only the settings it needs to
/// override. CLI arguments and environment variables are merged separately in
/// `main.rs` and take precedence over these values.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WebConfigFile {
    pub server: ServerConfigFile,
    pub web: WebServerConfigFile,
    pub features: FeatureConfigFile,
    pub oidc: OidcConfigFile,
    pub webhook: WebhookConfigFile,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ServerConfigFile {
    pub db: Option<String>,
    pub console_log_level: Option<String>,
    pub file_log_level: Option<String>,
    pub file_log_dir: Option<String>,
    pub config_server_port: Option<u16>,
    pub config_server_protocol: Option<String>,
    pub api_server_port: Option<u16>,
    pub api_server_addr: Option<IpAddr>,
    pub geoip_db: Option<String>,
    pub heartbeat_min_response_ms: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WebServerConfigFile {
    pub web_server_port: Option<u16>,
    pub web_server_addr: Option<IpAddr>,
    pub no_web: Option<bool>,
    pub api_host: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FeatureConfigFile {
    pub disable_registration: Option<bool>,
    pub allow_auto_create_user: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OidcConfigFile {
    pub issuer_url: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub username_claim: Option<String>,
    pub provider_name: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub redirect_url: Option<String>,
    pub disable_pkce: Option<bool>,
    pub frontend_base_url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WebhookConfigFile {
    pub url: Option<String>,
    pub secret: Option<String>,
    pub internal_auth_token: Option<String>,
    pub instance_id: Option<String>,
    pub instance_api_base_url: Option<String>,
}

impl WebConfigFile {
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default());
        };

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read web config file: {}", path.display()))?;
        Self::parse(&content)
            .with_context(|| format!("failed to parse web config file: {}", path.display()))
    }

    pub fn parse(content: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str(content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_partial_configuration() {
        let config = WebConfigFile::parse(
            r#"
                [server]
                db = "web.db"
                api_server_port = 12000

                [oidc]
                issuer_url = "https://auth.example.com"
                scopes = ["openid", "profile", "email"]

                [features]
                disable_registration = true
            "#,
        )
        .unwrap();

        assert_eq!(config.server.db.as_deref(), Some("web.db"));
        assert_eq!(config.server.api_server_port, Some(12000));
        assert_eq!(
            config.oidc.issuer_url.as_deref(),
            Some("https://auth.example.com")
        );
        assert_eq!(
            config.oidc.scopes.as_ref().unwrap(),
            &vec!["openid".to_string(), "profile".to_string(), "email".to_string()]
        );
        assert_eq!(config.features.disable_registration, Some(true));
    }

    #[test]
    fn rejects_unknown_fields() {
        assert!(WebConfigFile::parse("[server]\nunknown = true").is_err());
    }
}
