use crate::token::TokenError;
use crate::{model, token};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("Failed to read configuration file '{0}': {1}")]
    InputOutputError(String, std::io::Error),
    #[error("Failed to parse configuration file '{0}': {1}")]
    YamlParsingError(String, serde_yaml::Error),
    #[error("Token error")]
    TokenError(#[from] TokenError),
}

/// Returns a configuration object used for HTTP calls from the more generic configuration struct
pub fn from_client_configuration(
    configuration: &ClientConfiguration,
    tenant: &String,
) -> Result<model::Configuration, ConfigurationError> {
    let base_path = configuration.base_path_for(tenant);
    let token = token::get_token_for_tenant(configuration, tenant)?;

    Ok(model::Configuration {
        base_url: base_path,
        access_token: token.clone(),
    })
}

/// Reads the client configuration from a file
pub fn initialize(configuration: &String) -> Result<ClientConfiguration, ConfigurationError> {
    let configuration_path = Path::new(configuration.as_str());
    let configuration_content = read_to_string(configuration_path)
        .map_err(|e| ConfigurationError::InputOutputError(configuration.clone(), e))?;
    serde_yaml::from_str(&configuration_content)
        .map_err(|e| ConfigurationError::YamlParsingError(configuration.clone(), e))
}

/// Represents a Physna tenant. The base path, identity provider URL and scope are optional
/// and, when present, take precedence over the global values in the client configuration.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Tenant {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub page_size: Option<u32>,
    #[serde(default)]
    pub base_path: Option<String>,
    #[serde(default)]
    pub identity_provider_url: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

/// The default base path of the Physna API
pub const DEFAULT_BASE_PATH: &str = "https://api.physna.com";

/// The default OAuth scope requested from the identity provider
pub const DEFAULT_SCOPE: &str = "physna-api/access";

/// The client configuration contains the default base path, URL to the identity provider and OAuth scope,
/// as well as the configured tenants, each of which can override those defaults
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientConfiguration {
    #[serde(default)]
    pub base_path: Option<String>,
    #[serde(default)]
    pub identity_provider_url: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    pub tenants: HashMap<String, Tenant>,
}

impl ClientConfiguration {
    /// Returns the base path for the tenant, falling back to the global value and then to the default
    pub fn base_path_for(&self, tenant: &str) -> String {
        self.tenants
            .get(tenant)
            .and_then(|t| t.base_path.clone())
            .or_else(|| self.base_path.clone())
            .unwrap_or_else(|| String::from(DEFAULT_BASE_PATH))
    }

    /// Returns the identity provider URL for the tenant, falling back to the global value
    pub fn identity_provider_url_for(&self, tenant: &str) -> Option<String> {
        self.tenants
            .get(tenant)
            .and_then(|t| t.identity_provider_url.clone())
            .or_else(|| self.identity_provider_url.clone())
    }

    /// Returns the OAuth scope for the tenant, falling back to the global value and then to the default
    pub fn scope_for(&self, tenant: &str) -> String {
        self.tenants
            .get(tenant)
            .and_then(|t| t.scope.clone())
            .or_else(|| self.scope.clone())
            .unwrap_or_else(|| String::from(DEFAULT_SCOPE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> ClientConfiguration {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn test_defaults_when_missing() {
        let configuration = parse("tenants:\n  acme:\n    client_id: abc\n");
        assert_eq!(configuration.base_path_for("acme"), DEFAULT_BASE_PATH);
        assert_eq!(configuration.identity_provider_url_for("acme"), None);
        assert_eq!(configuration.scope_for("acme"), DEFAULT_SCOPE);
    }

    #[test]
    fn test_global_values_apply_to_tenants() {
        let configuration = parse(
            "base_path: https://global.example.com\n\
             identity_provider_url: https://global.example.com/oauth2/token\n\
             scope: tenantApp roles\n\
             tenants:\n  acme:\n    client_id: abc\n",
        );
        assert_eq!(configuration.base_path_for("acme"), "https://global.example.com");
        assert_eq!(
            configuration.identity_provider_url_for("acme").as_deref(),
            Some("https://global.example.com/oauth2/token")
        );
        assert_eq!(configuration.scope_for("acme"), "tenantApp roles");
    }

    #[test]
    fn test_tenant_values_override_global_values() {
        let configuration = parse(
            "base_path: https://global.example.com\n\
             identity_provider_url: https://global.example.com/oauth2/token\n\
             scope: tenantApp roles\n\
             tenants:\n  acme:\n    client_id: abc\n\
             \x20   base_path: https://acme.example.com\n\
             \x20   identity_provider_url: https://acme.example.com/oauth2/token\n\
             \x20   scope: physna-api/access\n\
             \x20 other:\n    client_id: def\n",
        );
        assert_eq!(configuration.base_path_for("acme"), "https://acme.example.com");
        assert_eq!(
            configuration.identity_provider_url_for("acme").as_deref(),
            Some("https://acme.example.com/oauth2/token")
        );
        assert_eq!(configuration.scope_for("acme"), "physna-api/access");
        assert_eq!(configuration.base_path_for("other"), "https://global.example.com");
        assert_eq!(configuration.scope_for("other"), "tenantApp roles");
    }
}
