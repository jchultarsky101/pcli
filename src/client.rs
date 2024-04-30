use crate::model::{
    FolderCreateResponse, GeoMatch, ImageMatch, ListOfModels, Model, ModelCreateMetadataResponse,
    ModelMetadata, ModelMetadataItem, Property, PropertyCollection,
};
use core::str::FromStr;
use log;
use reqwest::{
    self,
    blocking::Client,
    blocking::Response,
    header::{HeaderMap, HeaderName, HeaderValue},
    StatusCode,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use std::{fs::File, path::Path};
use std::{io::Read, path::PathBuf};
use thiserror::Error;
use url::{self, Url};
use uuid::Uuid;

fn urlencode<T: AsRef<str>>(s: T) -> String {
    url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Parsing error")]
    Parsing(String),
    #[error("Action is unauthorized")]
    Unauthorized,
    #[error("Action is forbidden")]
    Forbidden,
    #[error("Resource not found")]
    NotFound,
    #[error("Failed to delete folder")]
    FailedToDeleteFolder(String),
    #[error("Unsupported operation")]
    Unsupported(String),
    #[error("{0}")]
    ServerError(String),
    #[error("The request is badly formed")]
    BadRequest,
    #[error("Resource already exists on the server")]
    Conflict(String),
    #[error("Invalid input file")]
    InvalidInputFile,
    #[error("I/O error")]
    InputOutputError(#[from] std::io::Error),
    #[error("HTTP error")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON parsing error")]
    JsonError(#[from] serde_json::Error),
    #[error("The input is not a file")]
    InputNotFile,
    #[error("Failed to extract the file ane from the path")]
    CannotExtractFileNameFromPath,
    #[error("The file size is too large")]
    FileTooLarge,
    #[error("Failed to find any matches for image")]
    FailedToFindMatchesForImage,
}

#[derive(Debug, Clone, Deserialize)]
struct ServerErrorDetails {
    #[serde(rename = "message")]
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerError {
    #[serde(rename = "error")]
    error_details: ServerErrorDetails,
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PageData {
    #[serde(rename = "total")]
    pub total: u32,
    #[serde(rename = "perPage")]
    pub per_page: u32,
    #[serde(rename = "currentPage")]
    pub current_page: u32,
    #[serde(rename = "lastPage")]
    pub last_page: u32,
    #[serde(rename = "startIndex")]
    pub start_index: u32,
    #[serde(rename = "endIndex")]
    pub end_index: u32,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PartToPartMatch {
    #[serde(rename = "matchedModel")]
    pub matched_model: Model,
    #[serde(rename = "matchPercentage")]
    pub match_percentage: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FolderFilterData {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id")]
    pub id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "name")]
    pub name: Option<String>,
}

impl Default for FolderFilterData {
    fn default() -> Self {
        Self {
            id: None,
            name: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize)]
pub struct PropertyFilterData {
    #[serde(rename = "id")]
    pub id: u32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "values")]
    pub values: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FilterData {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "folders")]
    pub folders: Option<Vec<FolderFilterData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "properties")]
    pub properties: Option<Vec<PropertyFilterData>>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PartToPartMatchResponse {
    #[serde(rename = "matches")]
    pub matches: Vec<PartToPartMatch>,
    #[serde(rename = "pageData")]
    pub page_data: PageData,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Folder {
    #[serde(rename = "id")]
    pub id: u32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "ownerId", skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FolderListPageResponse {
    #[serde(rename = "folders")]
    pub folders: Vec<Folder>,
    #[serde(rename = "pageData")]
    pub page_data: Box<PageData>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FolderListResponse {
    #[serde(rename = "folders")]
    pub folders: Vec<Folder>,
}

impl From<FolderListPageResponse> for FolderListResponse {
    fn from(page_response: FolderListPageResponse) -> Self {
        Self {
            folders: page_response.folders,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SingleModelResponse {
    #[serde(rename = "model")]
    pub model: Box<Model>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelMetadataResponse {
    #[serde(rename = "metadata")]
    pub metadata: Vec<ModelMetadataItem>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "assemblyTree")]
    AssemblyTree,
    #[serde(rename = "assemblyPart")]
    AssemblyPart,
    #[serde(rename = "subAssembly")]
    SubAssembly,
}

impl Default for Type {
    fn default() -> Type {
        Self::AssemblyTree
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct AssemblyTree {
    #[serde(rename = "type")]
    pub _type: Type,
    #[serde(rename = "modelId")]
    pub uuid: String,
    #[serde(rename = "children", skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<AssemblyTree>>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelListResponse {
    #[serde(rename = "models")]
    pub models: Vec<Model>,
    #[serde(rename = "pageData")]
    pub page_data: Box<PageData>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
struct PropertyRequest {
    #[serde(rename = "metadataKeyName")]
    name: String,
}

impl PropertyRequest {
    pub fn new(name: String) -> PropertyRequest {
        PropertyRequest { name }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
struct PropertyValueRequest {
    value: String,
}

impl PropertyValueRequest {
    pub fn new(value: String) -> PropertyValueRequest {
        PropertyValueRequest { value }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
struct PropertyResponse {
    #[serde(rename = "metadataKey")]
    property: Property,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelFilter {
    #[serde(rename = "containerIds")]
    pub container_ids: Vec<u32>,
}

impl ModelFilter {
    pub fn new(folders: Vec<u32>) -> Self {
        ModelFilter {
            container_ids: folders,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageClassifierCreateRequest {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "modelFilter")]
    pub filter: ModelFilter,
}

impl ImageClassifierCreateRequest {
    pub fn new(name: String, folders: Vec<u32>) -> Self {
        let filter = ModelFilter::new(folders);
        ImageClassifierCreateRequest { name, filter }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageClassifierCreateResponse {
    #[serde(rename = "id")]
    pub id: Uuid,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "modelFilter")]
    pub model_filter: ModelFilter,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct GeoMatchPageResponse {
    #[serde(rename = "matches")]
    pub matches: Vec<GeoMatch>,
    #[serde(rename = "pageData")]
    pub page_data: PageData,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ImageUploadSpecsRequest {
    filename: String,
}

impl ImageUploadSpecsRequest {
    fn new(filename: String) -> Self {
        Self { filename }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ImageUploadSizeRequirements {
    #[serde(rename = "minSizeInBytes")]
    pub min_size_in_bytes: u64,
    #[serde(rename = "maxSizeInBytes")]
    pub max_size_in_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ImageUploadHeaders {
    #[serde(rename = "Content-Type")]
    pub content_type: String,
    #[serde(rename = "X-Goog-Content-Length-Range")]
    pub content_length_range: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ImageUploadResponse {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "uploadUrl")]
    pub upload_url: String,
    #[serde(rename = "headers")]
    pub headers: ImageUploadHeaders,
    #[serde(rename = "fileSizeRequirements")]
    pub file_size_requirements: ImageUploadSizeRequirements,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ImageUploadSpecsResponse {
    pub image: ImageUploadResponse,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ImageMatchPageResponse {
    #[serde(rename = "matches")]
    pub matches: Vec<ImageMatch>,
    #[serde(rename = "pageData")]
    pub page_data: PageData,
}

#[derive(Clone, Debug, Serialize)]
struct ModelUploadRequestModelElement {
    #[serde(rename = "filePath")]
    file_path: String,
    #[serde(rename = "name")]
    name: String,
}

impl ModelUploadRequestModelElement {
    fn new(folder: &str, name: &str) -> Self {
        let path = [folder, name].join("/");
        Self {
            file_path: path.to_string(),
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct ModelUploadRequest {
    #[serde(rename = "models")]
    models: Vec<ModelUploadRequestModelElement>,
}

impl ModelUploadRequest {
    fn new(folder: &str, name: &str) -> Self {
        let element = ModelUploadRequestModelElement::new(folder, name);
        Self {
            models: vec![element],
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct UploadInfoResponse {
    #[serde(rename = "uploadUrl")]
    url: String,
    #[serde(rename = "headers")]
    headers: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ModelUploadElementResponse {
    #[serde(rename = "uploadInfo")]
    info: UploadInfoResponse,
    #[serde(rename = "model")]
    model: Model,
}

#[derive(Clone, Debug, Deserialize)]
struct ModelUploadResponse {
    #[serde(rename = "models")]
    models: Vec<ModelUploadElementResponse>,
}

struct CustomHeaderName(String);

impl CustomHeaderName {
    fn new(name: String) -> Self {
        Self(name.to_owned())
    }

    fn into_header_name(&self) -> Option<HeaderName> {
        HeaderName::from_str(self.0.as_str()).ok()
    }
}

impl From<String> for CustomHeaderName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl ToString for CustomHeaderName {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}

#[derive(Clone, Debug, Deserialize)]
struct SourceFileResponse {
    #[serde(rename = "sourceFile")]
    source_file_url: Url,
}

#[derive(Clone, Debug)]
pub struct ApiClient {
    pub client: Client,
    pub base_url: String,
    pub tenant: String,
    pub access_token: String,
}

impl ApiClient {
    pub fn connect(base_url: &String, tenant: &String, access_token: &String) -> ApiClient {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(180))
            .build()
            .unwrap();

        ApiClient {
            client,
            base_url: base_url.to_owned(),
            tenant: tenant.to_owned(),
            access_token: access_token.to_owned(),
        }
    }

    fn evaluate_response(&self, response: &Response) -> Result<(), ClientError> {
        let status = response.status();
        if status.is_success() {
            ()
        }

        match status {
            StatusCode::OK
            | StatusCode::CREATED
            | StatusCode::ACCEPTED
            | StatusCode::NON_AUTHORITATIVE_INFORMATION
            | StatusCode::NO_CONTENT
            | StatusCode::RESET_CONTENT => (), // Nothing to do, continue
            StatusCode::FORBIDDEN => return Err(ClientError::Forbidden),
            StatusCode::NOT_FOUND => return Err(ClientError::NotFound),
            StatusCode::UNAUTHORIZED => return Err(ClientError::Unauthorized),
            StatusCode::BAD_REQUEST => return Err(ClientError::BadRequest),
            StatusCode::CONFLICT => {
                return Err(ClientError::Conflict(String::from(
                    "Resource already exists on the server",
                )))
            }
            StatusCode::CONTINUE
            | StatusCode::SWITCHING_PROTOCOLS
            | StatusCode::PROCESSING
            | StatusCode::PARTIAL_CONTENT
            | StatusCode::MULTI_STATUS
            | StatusCode::ALREADY_REPORTED
            | StatusCode::IM_USED
            | StatusCode::MULTIPLE_CHOICES
            | StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::SEE_OTHER
            | StatusCode::NOT_MODIFIED
            | StatusCode::USE_PROXY
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
            | StatusCode::PAYMENT_REQUIRED
            | StatusCode::METHOD_NOT_ALLOWED
            | StatusCode::NOT_ACCEPTABLE
            | StatusCode::PROXY_AUTHENTICATION_REQUIRED
            | StatusCode::REQUEST_TIMEOUT
            | StatusCode::GONE
            | StatusCode::LENGTH_REQUIRED
            | StatusCode::PRECONDITION_FAILED
            | StatusCode::PAYLOAD_TOO_LARGE
            | StatusCode::URI_TOO_LONG
            | StatusCode::UNSUPPORTED_MEDIA_TYPE
            | StatusCode::RANGE_NOT_SATISFIABLE
            | StatusCode::EXPECTATION_FAILED
            | StatusCode::IM_A_TEAPOT
            | StatusCode::MISDIRECTED_REQUEST
            | StatusCode::UNPROCESSABLE_ENTITY
            | StatusCode::LOCKED
            | StatusCode::FAILED_DEPENDENCY
            | StatusCode::UPGRADE_REQUIRED
            | StatusCode::PRECONDITION_REQUIRED
            | StatusCode::TOO_MANY_REQUESTS
            | StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE
            | StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS
            | StatusCode::NOT_IMPLEMENTED
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
            | StatusCode::HTTP_VERSION_NOT_SUPPORTED
            | StatusCode::VARIANT_ALSO_NEGOTIATES
            | StatusCode::INSUFFICIENT_STORAGE
            | StatusCode::LOOP_DETECTED
            | StatusCode::NOT_EXTENDED
            | StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::NETWORK_AUTHENTICATION_REQUIRED => {
                return Err(ClientError::Unsupported(format!(
                    "Server responded with error status: {:?}",
                    status
                )))
            }
            _ => {
                return Err(ClientError::Unsupported(
                    "Unexpected query status code".to_string(),
                ))
            }
        };

        Ok(())
    }

    /*
        pub fn get(
            &self,
            url: &str,
            query_parameters: Option<Vec<(String, String)>>,
        ) -> Result<String, ClientError> {
            let mut builder = self
                .client
                .request(reqwest::Method::GET, url)
                .timeout(Duration::from_secs(180))
                .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
                .header("X-PHYSNA-TENANTID", self.tenant.to_owned());

            match query_parameters {
                Some(query_parametes) => {
                    for (key, value) in query_parametes {
                        builder = builder.query(&[(key.to_owned(), value.to_owned())]);
                    }
                }
                None => (),
            }

            let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
            log::trace!("GET {}", request.url());
            let response = self.client.execute(request);

            self.handle_response::<String>(response)
        }
    */

    pub fn get_model_match_page(
        &self,
        uuid: &Uuid,
        threshold: f64,
        per_page: u32,
        page: u32,
    ) -> Result<PartToPartMatchResponse, ClientError> {
        let url = format!(
            "{}/v2/models/{id}/part-to-part-matches",
            self.base_url,
            id = urlencode(uuid.to_string())
        );

        let builder = self
            .client
            .get(url)
            .timeout(Duration::from_secs(180))
            .query(&[
                ("threshold", threshold.to_string().as_str()),
                ("perPage", per_page.to_string().as_str()),
                ("page", page.to_string().as_str()),
            ])
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", self.tenant.to_owned());

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("GET {}", request.url());
        let response = self.client.execute(request);

        Ok(self.handle_response::<PartToPartMatchResponse>(response)?)
    }

    pub fn get_model_scan_match_page(
        &self,
        uuid: &Uuid,
        threshold: f64,
        per_page: u32,
        page: u32,
    ) -> Result<PartToPartMatchResponse, ClientError> {
        let url = format!(
            "{}/v2/models/{id}/scan-matches",
            self.base_url,
            id = urlencode(uuid.to_string())
        );

        let builder = self
            .client
            .get(url)
            .timeout(Duration::from_secs(180))
            .query(&[
                ("threshold", threshold.to_string().as_str()),
                ("perPage", per_page.to_string().as_str()),
                ("page", page.to_string().as_str()),
            ])
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", self.tenant.to_owned());

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("GET {}", request.url());
        let response = self.client.execute(request);

        Ok(self.handle_response::<PartToPartMatchResponse>(response)?)
    }

    fn get_list_of_folders_page(
        &self,
        page: u32,
        per_page: u32,
        filter: Option<String>,
    ) -> Result<FolderListPageResponse, ClientError> {
        let url = format!("{}/v2/folders", self.base_url);

        let mut query: Vec<(&str, String)> = Vec::new();
        query.push(("page", page.to_string()));
        query.push(("perPage", per_page.to_string()));
        if filter.is_some() {
            query.push(("filter", filter.unwrap_or_default()));
        };

        let builder = self
            .client
            .get(url)
            .timeout(Duration::from_secs(30))
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header(
                reqwest::header::ACCEPT,
                HeaderValue::from_static("application/json"),
            )
            .header("X-PHYSNA-TENANTID", self.tenant.to_owned())
            .query(&query);

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("GET {}", request.url());
        let response = self.client.execute(request);
        Ok(self.handle_response::<FolderListPageResponse>(response)?)
    }

    pub fn get_list_of_folders(
        &self,
        desired_folders: Option<HashSet<String>>,
    ) -> Result<FolderListResponse, ClientError> {
        log::trace!("Reading list of folders...");

        let mut current_page: u32 = 1;
        let per_page: u32 = 1000;
        let filter: Option<String> = match desired_folders {
            Some(desired_folders) => {
                let folder_filter: Vec<String> = desired_folders
                    .into_iter()
                    .map(|f| format!("'{}'", f))
                    .collect();
                let folder_filter: String = folder_filter.join(",");
                let folder_filter = format!("name(in({}))", folder_filter);
                Some(folder_filter)
            }
            None => Some(String::default()),
        };

        let mut folders: Vec<Folder> = Vec::new();
        loop {
            let page = self.get_list_of_folders_page(current_page, per_page, filter.to_owned())?;
            folders.extend(page.folders);
            if current_page >= page.page_data.last_page {
                break;
            }
            current_page += 1;
        }

        Ok(FolderListResponse { folders })
    }

    pub fn delete_folder(&self, folders: &HashSet<String>) -> Result<(), ClientError> {
        log::trace!("Deleting folder {:?}...", folders);
        let url = format!("{}/v2/folders", self.base_url);
        let mut query_parameters: Vec<(String, String)> = Vec::new();

        for folder in folders {
            query_parameters.push(("ids".to_string(), folder.to_string()));
        }

        let builder = self
            .client
            .delete(url)
            .timeout(Duration::from_secs(180))
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", self.tenant.to_owned())
            .query(&query_parameters)
            .json(&folders);

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("DELETE {}", request.url());
        let response = self.client.execute(request);
        self.handle_response::<()>(response)
    }

    pub fn create_folder(&self, name: &String) -> Result<FolderCreateResponse, ClientError> {
        log::trace!("Creating folder {}...", &name);
        let url = format!("{}/v2/folders", self.base_url);

        let bearer: String = format!("Bearer {}", self.access_token);
        let builder = self
            .client
            .post(url)
            .timeout(Duration::from_secs(180))
            .header("Authorization", bearer)
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            .header("Content-Length", 0)
            .query(&[("name", name.to_owned())]);

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("POST {}", request.url());
        let response = self.client.execute(request);
        self.handle_response::<FolderCreateResponse>(response)
    }

    pub fn get_model(&self, uuid: &Uuid) -> Result<SingleModelResponse, ClientError> {
        let url = format!(
            "{}/v2/models/{id}",
            self.base_url,
            id = urlencode(uuid.to_string())
        );
        log::trace!("Reading model {}...", uuid.to_string());

        let builder = self
            .client
            .get(url)
            .timeout(Duration::from_secs(180))
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", self.tenant.to_owned());

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("GET {}", request.url());
        let response = self.client.execute(request);

        Ok(self.handle_response::<SingleModelResponse>(response)?)
    }

    pub fn delete_model(&self, uuid: &Uuid) -> Result<(), ClientError> {
        let url = format!(
            "{}/v2/models/{id}",
            self.base_url,
            id = urlencode(uuid.to_string())
        );
        log::trace!("Deleting model {}...", uuid.to_string());

        let builder = self
            .client
            .delete(url)
            .timeout(Duration::from_secs(180))
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", self.tenant.to_owned());

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("DELETE {}", request.url());
        let response = self.client.execute(request);
        self.handle_response::<()>(response)
    }

    pub fn reprocess_model(&self, uuid: &Uuid) -> Result<(), ClientError> {
        let url = format!("{}/v2/models/{}/reprocess", self.base_url, uuid.to_string());
        log::trace!("Reprocessing model {}", url);

        let builder = self
            .client
            .post(url)
            .timeout(Duration::from_secs(180))
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant);

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("POST {}", request.url());
        let response = self.client.execute(request);
        self.handle_response::<()>(response)
    }

    pub fn get_model_metadata(&self, uuid: &Uuid) -> Result<Option<ModelMetadata>, ClientError> {
        let url = format!(
            "{}/v2/models/{id}/metadata",
            self.base_url,
            id = urlencode(uuid.to_string())
        );
        let per_page = 10000;
        let page = 1;

        let builder = self
            .client
            .get(url)
            .timeout(Duration::from_secs(180))
            .query(&[
                ("perPage", per_page.to_string().as_str()),
                ("page", page.to_string().as_str()),
            ])
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", self.tenant.to_owned());

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("GET {}", request.url());
        let response = self.client.execute(request);

        let response: Option<ModelMetadataResponse> =
            self.handle_response::<Option<ModelMetadataResponse>>(response)?;

        match response {
            Some(response) => {
                if !response.metadata.is_empty() {
                    let props: Vec<ModelMetadataItem> = response
                        .metadata
                        .into_iter()
                        .map(|property| {
                            ModelMetadataItem::new(property.key_id, property.name, property.value)
                        })
                        .collect();
                    return Ok(Some(ModelMetadata::new(props)));
                } else {
                    return Ok(None);
                }
            }
            None => Ok(None),
        }
    }

    pub fn get_assembly_tree_for_model(&self, uuid: &Uuid) -> Result<AssemblyTree, ClientError> {
        let url = format!(
            "{}/v2/models/{id}/assembly-tree",
            self.base_url,
            id = urlencode(uuid.to_string())
        );

        let builder = self
            .client
            .post(url)
            .timeout(Duration::from_secs(180))
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            .header("Content-Length", 0);

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("POST {}", request.url());
        let response = self.client.execute(request);
        Ok(self.handle_response::<AssemblyTree>(response)?)
    }

    /// Returns a single-page response for list of models
    ///
    /// Parameters:
    ///
    /// folders - a list of folder IDs. If the list is empty, models from all folders will be included
    /// search - a search clause (e.g. part number)
    /// per_page - how many records to return per page
    /// page - the current page number
    pub fn get_list_of_models_page(
        &self,
        folders: HashSet<u32>,
        search: Option<&String>,
        per_page: u32,
        page: u32,
    ) -> Result<ModelListResponse, ClientError> {
        let url = format!("{}/v2/models", self.base_url);

        let mut query_parameters: Vec<(String, String)> = Vec::new();

        if folders.len() > 0 {
            let filter: Vec<String> = folders.iter().map(|f| f.to_string()).collect();
            let filter_operations = format!("folderId(in({}))", filter.join(","));

            log::trace!("Filter Operations: {}", filter_operations.to_owned());
            query_parameters.push(("filter".to_string(), filter_operations));
        }

        if search.is_some() {
            query_parameters.push(("search".to_string(), search.unwrap().to_owned()));
        }

        query_parameters.push(("perPage".to_string(), per_page.to_string()));
        query_parameters.push(("page".to_string(), page.to_string()));

        log::trace!("GET {}", url.to_string());
        let builder = self
            .client
            .get(url)
            .timeout(Duration::from_secs(60))
            .header("Cache-Control", "no-cache")
            .header("Content-Length", "0")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            .query(&query_parameters);

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("GET {}", request.url());
        let response = self.client.execute(request);
        self.handle_response::<ModelListResponse>(response)
    }

    /// Checks the response from an HTTP operation for errors and if none, parses the response body into specific type
    ///
    /// Parameters:
    ///
    /// response - thre result from the response
    fn handle_response<'de, T>(
        &self,
        response: Result<Response, reqwest::Error>,
    ) -> Result<T, ClientError>
    where
        T: DeserializeOwned + 'static,
    {
        log::trace!("Analyzing HTTP response...");
        match response {
            Ok(response) => {
                log::trace!("Evaluating the HTTP status ({})...", response.status());
                match self.evaluate_response(&response) {
                    Ok(_) => {
                        // normal exit status from the HTTP operation
                        log::trace!("The exit status code indicates normal operation");

                        // get the JSON from the response body
                        let json = response.text();
                        match json {
                            Ok(json) => {
                                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<()>() {
                                    // Correctly return `()` for `T`
                                    unsafe { return Ok(std::mem::transmute_copy(&())) }
                                } else {
                                    let object = serde_json::from_str::<T>(&json)?;
                                    Ok(object)
                                }
                            }
                            Err(e) => Err(ClientError::ServerError(e.to_string())),
                        }
                    }
                    Err(e) => {
                        // the response status indicates an error

                        // attempting to get the message sent by the server...
                        let json = response.text();
                        match json {
                            Ok(json) => {
                                // the response has a payload
                                log::trace!("Response: {}", json.to_owned());

                                match serde_json::from_str::<ServerError>(&json) {
                                    Ok(server_error) => Err(ClientError::ServerError(
                                        server_error.error_details.message,
                                    )),
                                    Err(_) => Err(e),
                                }
                            }
                            Err(_) => Err(e),
                        }
                    }
                }
            }
            Err(e) => Err(ClientError::ServerError(e.to_string())),
        }
    }

    pub fn upload_model(&self, folder: &str, path: &PathBuf) -> Result<Option<Model>, ClientError> {
        let url = format!("{}/v2/models", self.base_url);

        let name = path.file_name();
        if name.is_none() {
            return Err(ClientError::InvalidInputFile);
        }
        let name = String::from(name.unwrap().to_string_lossy());

        log::trace!("Uploading model data...");
        let request = ModelUploadRequest::new(folder, name.as_str());

        log::trace!("POST {}", url.to_string());
        let builder = self
            .client
            .post(url)
            .timeout(Duration::from_secs(360))
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .query(&[("createMissingFolders", "false")])
            //.header("Content-Range", range_value.to_owned())
            .json(&request);

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("GET {}", request.url());
        let response = self.client.execute(request);
        let response: ModelUploadResponse =
            self.handle_response::<ModelUploadResponse>(response)?;

        log::trace!("Response: {:?}", response);

        let response_model = response.models.get(0);
        match response_model {
            Some(response_model) => {
                let response_model = response_model.to_owned();
                let url = response_model.info.url;
                let model = response_model.model;
                let mut headers: HeaderMap = HeaderMap::new();
                response_model.info.headers.into_iter().for_each(|(k, v)| {
                    let header_name: HeaderName = CustomHeaderName::from(k.to_owned())
                        .into_header_name()
                        .unwrap();
                    let header_value: HeaderValue = HeaderValue::from_str(v.as_str()).unwrap();
                    headers.append(header_name, header_value);
                });

                let mut file = std::fs::File::open(path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;

                let _ = self
                    .client
                    .put(url)
                    .timeout(Duration::from_secs(180))
                    .headers(headers)
                    .body(buffer)
                    .send();

                Ok(Some(model.to_owned()))
            }
            None => Ok(None),
        }
    }

    pub fn download_model(&self, uuid: &Uuid) -> Result<(), ClientError> {
        let url = format!(
            "{}/v2/models/{}/source-file",
            self.base_url,
            uuid.to_string()
        );
        let bearer: String = format!("Bearer {}", self.access_token);
        log::trace!("Downloading model source file...");

        log::trace!("GET {}", url.to_string());
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(360))
            .header("Authorization", bearer)
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            .send();

        let response_source_file = self.handle_response::<SourceFileResponse>(response)?;
        let url = response_source_file.source_file_url;

        let url_for_path = url.clone();
        let file_name = url_for_path.path_segments().unwrap().next_back().unwrap();
        log::trace!("Extraced file name is {}", file_name.to_owned());

        log::trace!("GET {}", url.to_string());
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(120))
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .send()?;

        log::trace!("Download request is a success");

        let path = dirs::download_dir().unwrap();
        let path = path.join(file_name);

        log::trace!("Downloading file {}", path.to_string_lossy());

        let body = response.bytes()?;
        std::fs::write(path, &body)?;

        log::trace!("File downloaded");

        Ok(())
    }

    pub fn get_list_of_properties(&self) -> Result<PropertyCollection, ClientError> {
        let url = format!("{}/v2/metadata-keys", self.base_url);

        let builder = self
            .client
            .request(reqwest::Method::GET, url)
            .timeout(Duration::from_secs(180))
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", self.tenant.to_owned());

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("GET {}", request.url());
        let response = self.client.execute(request);

        Ok(self.handle_response::<PropertyCollection>(response)?)
    }

    pub fn post_property(&self, name: &String) -> Result<Property, ClientError> {
        let url = format!("{}/v2/metadata-keys", self.base_url);
        let bearer: String = format!("Bearer {}", self.access_token);

        log::trace!(
            "Registering a new property with name of \"{}\"...",
            name.clone()
        );
        log::trace!("POST {}", url);

        let request = PropertyRequest::new(name.to_owned());
        log::trace!("Request: {:?}", &request);

        let response = self
            .client
            .post(url)
            .timeout(Duration::from_secs(180))
            .header("Authorization", bearer)
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            //.header("Content-Range", range_value.to_owned())
            .json(&request)
            .send();

        let result = self.handle_response::<PropertyResponse>(response)?;
        Ok(result.property)
    }

    pub fn put_model_property(
        &self,
        model_uuid: &Uuid,
        id: &u64,
        item: &ModelMetadataItem,
    ) -> Result<ModelMetadataItem, ClientError> {
        let url = format!("{}/v2/models/{}/metadata/{}", self.base_url, model_uuid, id);
        let bearer: String = format!("Bearer {}", self.access_token);

        log::trace!("PUT {}", url);

        let response = self
            .client
            .put(url)
            .timeout(Duration::from_secs(180))
            .header("Authorization", bearer)
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            //.header("Content-Range", range_value.to_owned())
            .json(&PropertyValueRequest::new(item.value.to_owned()))
            .send();

        let result = self.handle_response::<ModelCreateMetadataResponse>(response)?;
        Ok(result.metadata)
    }

    pub fn delete_model_property(&self, model_uuid: &Uuid, id: &u64) -> Result<(), ClientError> {
        let url = format!("{}/v2/models/{}/metadata/{}", self.base_url, model_uuid, id);
        let bearer: String = format!("Bearer {}", self.access_token);

        log::trace!("DELETE {}", url);

        let response = self
            .client
            .delete(url)
            .timeout(Duration::from_secs(180))
            .header("Authorization", bearer)
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            //.header("Content-Range", range_value.to_owned())
            .send();

        self.handle_response::<()>(response)
    }

    pub fn get_image_upload_specs(&self, path: &Path) -> Result<ImageUploadResponse, ClientError> {
        if !path.is_file() {
            return Err(ClientError::InputNotFile);
        }

        let url = format!("{}/v2/images", self.base_url);
        let bearer: String = format!("Bearer {}", self.access_token);

        let filename = match path.file_name() {
            Some(filename) => filename.to_str().unwrap(),
            None => return Err(ClientError::CannotExtractFileNameFromPath),
        };

        log::trace!("Requesting upload specs for image {}", &filename);
        let response = self
            .client
            .post(url)
            .timeout(Duration::from_secs(180))
            .header("Authorization", bearer)
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            .json(&ImageUploadSpecsRequest::new(filename.to_string()))
            .send()?;
        let json = response.text().unwrap();
        //trace!("{}", json);

        let file_size_requirements: ImageUploadSpecsResponse = serde_json::from_str(&json)?;
        //trace!("File size requirements: {:?}", &file_size_requirements);

        Ok(file_size_requirements.image)
    }

    pub fn upload_image_file(
        &self,
        url: Url,
        upload_size_requirements: ImageUploadSizeRequirements,
        path: &Path,
        mime: String,
        content_range: String,
    ) -> Result<(), ClientError> {
        log::trace!("Uploading image file {}...", path.to_str().unwrap());
        //trace!("Upload URL: {}", url.to_string());

        let max_size = upload_size_requirements.max_size_in_bytes;
        let file = File::open(path)?;
        let file_size = file.metadata().unwrap().len();
        if file_size > max_size {
            return Err(ClientError::FileTooLarge);
        }

        let _response = self
            .client
            .put(url)
            .timeout(Duration::from_secs(180))
            .header("Content-Type", mime)
            .header("X-Goog-Content-Length-Range", content_range)
            .body(file)
            .send()?;

        Ok(())
    }

    fn get_image_search_matches_page(
        &self,
        id: String,
        search: Option<&String>,
        filter: Option<&String>,
        page: u32,
        per_page: u32,
    ) -> Result<ImageMatchPageResponse, ClientError> {
        log::trace!("Searching matching models for image with ID {id}...");

        let url = format!("{}/v2/images/model-matches", self.base_url);
        let mut query_parameters: Vec<(String, String)> = Vec::new();
        query_parameters.push(("id".to_string(), id));
        query_parameters.push(("perPage".to_string(), per_page.to_string()));
        query_parameters.push(("page".to_string(), page.to_string()));

        match search {
            Some(search) => query_parameters.push(("search".to_string(), search.to_owned())),
            None => (),
        }
        match filter {
            Some(filter) => query_parameters.push(("filter".to_string(), filter.to_owned())),
            None => (),
        }

        let builder = self
            .client
            .request(reqwest::Method::GET, url)
            .timeout(Duration::from_secs(180))
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", self.tenant.to_owned());

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        log::trace!("GET {}", request.url());
        let response = self.client.execute(request);

        Ok(self.handle_response::<ImageMatchPageResponse>(response)?)
    }

    pub fn get_image_search_maches(
        &self,
        id: String,
        search: Option<&String>,
        filter: Option<&String>,
        max_matches: u32,
        per_page: u32,
    ) -> Result<ListOfModels, ClientError> {
        let mut page = 1;
        // let per_page = 20;
        let mut models: Vec<Model> = Vec::new();

        log::trace!("Limit={}", max_matches);

        loop {
            let page_result = self.get_image_search_matches_page(
                id.clone(),
                search.clone(),
                filter.clone(),
                page,
                per_page,
            )?;
            let page_models: Vec<Model> = page_result
                .matches
                .into_iter()
                .map(|m| Model::from(m.model))
                .collect();
            let local_size = page_models.len();
            models.extend(page_models);
            if models.len() > max_matches as usize {
                models.truncate(max_matches as usize);
            }

            log::trace!("Page {}", page);
            log::trace!("size={}", local_size);
            log::trace!("models.size={}", models.len());

            page += 1;
            if local_size < per_page as usize || models.len() >= max_matches as usize {
                break;
            }
        }

        Ok(ListOfModels::from(models))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_deserialization_of_model_with_metadata() {
        let json = r#"
 {
   "models": [
    {
      "thumbnail": "https://localhost/images/test.svg",
      "createdAt": "2022-11-03T14:54:57.801Z",
      "fileType": ".STL",
      "folderId": 1,
      "id": "9438bec9-eaff-4802-839f-ff9ca029debb",
      "isAssembly": false,
      "metadata": [
        {
          "metadataKeyId": 1,
          "name": "name1",
          "value": "value1"
        },
        {
          "metadataKeyId": 2,
          "name": "name2",
          "value": "value2"
        },
        {
          "metadataKeyId": 3,
          "name": "name3",
          "value": "value3"
        },
        {
          "metadataKeyId": 4,
          "name": "name4",
          "value": "value4"
        },
        {
          "metadataKeyId": 5,
          "name": "name5",
          "value": "value5"
        },
        {
          "metadataKeyId": 6,
          "name": "",
          "value": "8"
        },
        {
          "metadataKeyId": 7,
          "name": "name7",
          "value": "value7"
        }
      ],
      "name": "Some Part",
      "ownerId": "1e9caaf7-2ab1-408f-adc0-f32776f2ab26",
      "state": "finished",
      "units": "mm"
    }
  ],
  "pageData": {
     "total": 1,
     "perPage": 50,
     "currentPage": 1,
     "lastPage": 1,
     "startIndex": 0,
     "endIndex": 600
  }
}
        "#;

        let result = serde_json::from_str::<ModelListResponse>(&json);
        match result {
            Ok(_models) => (),
            Err(e) => panic!("Parsing of JSON failed: {}", e),
        }
    }
}
