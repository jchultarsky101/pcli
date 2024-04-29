use base64::engine::general_purpose;
use base64::Engine;
use dirs::home_dir;
use http::StatusCode;
use jsonwebtoken::decode_header;
use log;
use rpassword;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Failed to decode token")]
    FailedToDecode,
    #[error("I/O error")]
    InputOutputError(#[from] std::io::Error),
    #[error("Empty client ID provided")]
    EmptyClientId,
    #[error("HTTP error")]
    HttpError(#[from] reqwest::Error),
    #[error("Failed to obtain security token from provider")]
    FailedToObtainTokenFromProvider,
    #[error("Unknown tenant {0}")]
    UnknownTenant(String),
}

pub fn get_token_for_tenant(
    configuration: &crate::configuration::ClientConfiguration,
    tenant: &String,
) -> Result<String, TokenError> {
    log::trace!("Obtaining new token from the provider...");
    let token = read_token_from_file(tenant);

    match token {
        Ok(token) => {
            log::trace!("Validating previously acquired token...");
            match validate_token(token) {
                Ok(token) => {
                    log::trace!("The current token is still valid");
                    Ok(token)
                }
                Err(_) => {
                    log::trace!("The existing token is no longer valid!");
                    let token = request_new_token_from_provider(configuration, tenant)?;
                    write_token_to_file(tenant, &token)?;
                    Ok(token)
                }
            }
        }
        Err(_e) => {
            log::trace!("No existing token found");
            let token = request_new_token_from_provider(configuration, tenant)?;
            write_token_to_file(tenant, &token)?;
            Ok(token)
        }
    }
}

pub fn validate_token(token: String) -> Result<String, TokenError> {
    match decode_header(&token) {
        Ok(_header) => Ok(token),
        Err(_) => Err(TokenError::FailedToDecode),
    }
}

pub fn resolve_file_name(tenant: &String) -> String {
    let home_directory = home_dir().unwrap();
    let home_directory = String::from(home_directory.to_str().unwrap());
    let default_token_file_path = home_directory;

    let mut file_name = String::from(default_token_file_path);
    file_name.push_str("/.pcli.");
    file_name.push_str(tenant.as_str());
    file_name.push_str(".token");

    file_name
}

pub fn write_token_to_file(tenant: &String, token: &String) -> Result<(), TokenError> {
    let file_name = resolve_file_name(&tenant);
    log::trace!(
        "Writing access token for tenant {} from file {}...",
        tenant,
        file_name
    );
    fs::write(file_name, token)?;

    Ok(())
}

pub fn read_token_from_file(tenant: &String) -> Result<String, TokenError> {
    let file_name = resolve_file_name(&tenant);
    log::trace!(
        "Reading access token for tenant {} to file {}...",
        tenant,
        file_name
    );
    Ok(fs::read_to_string(file_name)?)
}

pub fn invalidate_token(tenant: &String) -> Result<(), TokenError> {
    let file_name = resolve_file_name(&tenant);
    log::trace!(
        "Invalidating access token for tenant {} in file {}...",
        tenant,
        file_name
    );
    match fs::remove_file(file_name) {
        // There is nothing we can do if the file does not exist or it is locked.
        Ok(()) => (),
        Err(_) => (),
    }
    Ok(())
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AuthenticationResponse {
    token_type: String, //e.g. "Bearer"
    expires_in: u64,    //e.g. 36000
    access_token: String,
    scope: String, //e.g. "tenantApp"
}

fn read_client_secret_from_console() -> String {
    log::trace!("User is required to enter the client secret via the console.");
    rpassword::prompt_password("Enter client secret: ").unwrap()
}

fn request_new_token_from_provider(
    configuration: &crate::configuration::ClientConfiguration,
    tenant: &String,
) -> Result<String, TokenError> {
    log::trace!("Requesting new token...");
    let active_tenant = configuration.tenants.get(tenant);

    match active_tenant {
        Some(active_tenant) => {
            let client_id = active_tenant.client_id.clone();
            let client_secret = active_tenant.client_secret.clone();
            let actual_client_secret;
            let security_provider_url = configuration.identity_provider_url.clone();

            log::trace!("Requesting for tenant {:?}...", tenant.to_owned());

            match client_secret {
                Some(client_secret) => {
                    actual_client_secret = client_secret;
                }
                None => {
                    actual_client_secret = read_client_secret_from_console();
                }
            }

            if client_id.is_empty() {
                return Err(TokenError::EmptyClientId);
            }

            // 0. Encode Base64: clientId + ":" + clientSecret
            // 1. Set the headers
            // "Authorization", "Basic " + encodedCredentials
            // "cache-control", "no-cache"
            // 2. Prepare multi value request body:
            // "grant_type", "client_credentials"
            // "scope", "tenantApp"
            // 3. POST to the provider URL

            let combined_credentials = [client_id.clone(), actual_client_secret.clone()]
                .join(":")
                .to_owned();

            let encoded_credentials =
                general_purpose::STANDARD.encode(combined_credentials.to_owned());
            //let encoded_credentials = encode(combined_credentials);

            let mut authorization_header_value = String::from("Basic ");
            authorization_header_value.push_str(encoded_credentials.as_str());

            let params = [
                ("grant_type", "client_credentials"),
                ("scope", "tenantApp roles"),
            ];

            // Create the HTTP client instance
            //let client = reqwest::Client::new();
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(20))
                .build()?;

            let response = client
                .post(security_provider_url)
                .header("Authorization", authorization_header_value.as_str())
                .header("cache-control", "no-cache")
                .form(&params)
                .send();
            match response {
                Ok(response) => {
                    let status = response.status();

                    if status == StatusCode::OK {
                        let response_text = response.text();
                        match response_text {
                            Ok(response_text) => {
                                let response: AuthenticationResponse =
                                    serde_yaml::from_str(&response_text).unwrap();
                                let token = response.access_token;
                                Ok(token)
                            }
                            Err(_) => Err(TokenError::FailedToObtainTokenFromProvider),
                        }
                    } else {
                        Err(TokenError::FailedToObtainTokenFromProvider)
                    }
                }
                Err(_) => Err(TokenError::FailedToObtainTokenFromProvider),
            }
        }
        None => Err(TokenError::UnknownTenant(tenant.to_owned())),
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TokenEnvironment {
    token: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TokenContainer {
    environments: HashMap<String, TokenEnvironment>,
}
