use crate::token::TokenError;
use crate::{model, token};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("I/O error")]
    InputOutputError(#[from] std::io::Error),
    #[error("JSON parsing error")]
    YamlParsingError(#[from] serde_yaml::Error),
    #[error("Token error")]
    TokenError(#[from] TokenError),
}

/// Returns a configuration object used for HTTP calls from the more generic configuration struct
pub fn from_client_configuration(
    configuration: &ClientConfiguration,
    tenant: &String,
) -> Result<model::Configuration, ConfigurationError> {
    let base_path = configuration.base_path.clone();
    let token = token::get_token_for_tenant(configuration, tenant)?;

    Ok(model::Configuration {
        base_url: base_path,
        access_token: token.clone(),
    })
}

/// Reads the client configuration from a file
pub fn initialize(configuration: &String) -> Result<ClientConfiguration, ConfigurationError> {
    let configuration = Path::new(configuration.as_str());
    let configuration = read_to_string(configuration)?;
    Ok(serde_yaml::from_str(&configuration)?)
}

/// Represents a Physna tenant
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Tenant {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub page_size: Option<u32>,
}

/// The client configuration contains the base path, URL to the identity provider and the currently selected tenant
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientConfiguration {
    pub base_path: String,
    pub identity_provider_url: String,
    pub tenants: HashMap<String, Tenant>,
}
