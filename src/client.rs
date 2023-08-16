use crate::model::{
    FileUploadResponse, FolderCreateResponse, GeoMatch, ImageClassifier, ImageMatch,
    ListOfClassificationScores, ListOfGeoClassifiers, ListOfGeoLabels, ListOfModels, Model,
    ModelCreateMetadataResponse, ModelMetadata, ModelMetadataItem, Property, PropertyCollection,
};
use anyhow::{anyhow, Result};
use log::trace;
use reqwest::{
    self,
    blocking::{
        multipart::{Form, Part},
        Client,
    },
    StatusCode,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{fs::File, path::Path};
use substring::Substring;
use url::{self, Url};
use uuid::Uuid;

fn urlencode<T: AsRef<str>>(s: T) -> String {
    url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

#[derive(Clone, Debug)]
pub enum ClientError {
    Parsing(String),
    Unauthorized,
    Forbidden,
    NotFound,
    FailedToDeleteFolder(String),
    Unsupported(String),
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        ""
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> ClientError {
        let code = err.status();
        if let Some(StatusCode::UNAUTHORIZED) = code {
            ClientError::Unauthorized
        } else {
            ClientError::Unsupported(format!("{}", err))
        }
    }
}

impl std::convert::From<serde_json::Error> for ClientError {
    fn from(err: serde_json::Error) -> ClientError {
        ClientError::Parsing(format!("{}", err))
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parsing(message) => write!(f, "Parsing error: {}", message),
            Self::Unauthorized => write!(
                f,
                "Request is unauthorized! Please, renew your access token"
            ),
            Self::Forbidden => write!(f, "Request is forbidden!"),
            Self::NotFound => write!(f, "Resource not found!"),
            Self::FailedToDeleteFolder(message) => write!(f, "Error: {}", message),
            Self::Unsupported(message) => write!(f, "Error: {}", message),
        }
    }
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

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FolderFilterData {
    #[serde(rename = "id")]
    pub id: u32,
    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PropertyFilterData {
    #[serde(rename = "id")]
    pub id: u32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "values")]
    pub values: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FilterData {
    #[serde(rename = "folders")]
    pub folders: Vec<FolderFilterData>,
    #[serde(rename = "properties")]
    pub properties: Vec<PropertyFilterData>,
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
pub struct FolderListResponse {
    #[serde(rename = "folders")]
    pub folders: Vec<Folder>,
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

    fn evaluate_satus(&self, status: StatusCode) -> Result<(), ClientError> {
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
            | StatusCode::BAD_REQUEST
            | StatusCode::PAYMENT_REQUIRED
            | StatusCode::METHOD_NOT_ALLOWED
            | StatusCode::NOT_ACCEPTABLE
            | StatusCode::PROXY_AUTHENTICATION_REQUIRED
            | StatusCode::REQUEST_TIMEOUT
            | StatusCode::CONFLICT
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
            | StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::NOT_IMPLEMENTED
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
            | StatusCode::HTTP_VERSION_NOT_SUPPORTED
            | StatusCode::VARIANT_ALSO_NEGOTIATES
            | StatusCode::INSUFFICIENT_STORAGE
            | StatusCode::LOOP_DETECTED
            | StatusCode::NOT_EXTENDED
            | StatusCode::NETWORK_AUTHENTICATION_REQUIRED => {
                return Err(ClientError::Unsupported(format!("Status: {:?}", status)))
            }
            _ => {
                return Err(ClientError::Unsupported(
                    "Unexpected query status code".to_string(),
                ))
            }
        };

        Ok(())
    }

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
        trace!("GET {}", request.url());
        let response = self.client.execute(request)?;

        trace!("Status: {}", response.status());

        self.evaluate_satus(response.status())?;

        let content = response.text()?;
        //trace!("{}", content);
        Ok(content)
    }

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

        let mut query_parameters: Vec<(String, String)> = Vec::new();
        query_parameters.push(("threshold".to_string(), threshold.to_string()));
        query_parameters.push(("perPage".to_string(), per_page.to_string()));
        query_parameters.push(("page".to_string(), page.to_string()));

        let json = self.get(url.as_str(), Some(query_parameters))?;
        //trace!("{}", json);
        let result: PartToPartMatchResponse = serde_json::from_str(&json)?;
        Ok(result)
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

        let mut query_parameters: Vec<(String, String)> = Vec::new();
        query_parameters.push(("threshold".to_string(), threshold.to_string()));
        query_parameters.push(("perPage".to_string(), per_page.to_string()));
        query_parameters.push(("page".to_string(), page.to_string()));

        let json = self.get(url.as_str(), Some(query_parameters))?;
        // trace!("{}", json);

        //trace!("Parsing JSON to PartToPartMatchResponse...");
        //std::fs::write("scan.json", &json).expect("Unable to write file");

        let result: PartToPartMatchResponse = serde_json::from_str(&json)?;
        trace!("Object deserialized");
        Ok(result)
    }

    pub fn get_list_of_folders(&self) -> Result<FolderListResponse, ClientError> {
        trace!("Reading list of folders...");
        let url = format!("{}/v2/folders", self.base_url);

        let json = self.get(url.as_str(), None)?;
        //trace!("{}", json);
        let result: FolderListResponse = serde_json::from_str(&json)?;
        Ok(result)
    }

    pub fn delete_folder(&self, folders: &Vec<u32>) -> Result<(), ClientError> {
        trace!("Deleting folder {:?}...", folders);
        let url = format!("{}/v2/folders", self.base_url);
        let mut query_parameters: Vec<(String, String)> = Vec::new();

        for folder in folders {
            query_parameters.push(("ids".to_string(), folder.to_string()));
        }

        let builder = self
            .client
            .request(reqwest::Method::DELETE, url)
            .timeout(Duration::from_secs(180))
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", self.tenant.to_owned())
            .query(&query_parameters)
            .json(&folders);

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        let response = self.client.execute(request)?;
        let status = response.status();

        if status.is_client_error() {
            return Err(ClientError::FailedToDeleteFolder("Error deleting folder. Make sure the folder is empty first or use the --force flag".to_string()));
        } else {
            self.evaluate_satus(status)?;
        }

        Ok(())
    }

    pub fn create_folder(&self, name: &String) -> Result<FolderCreateResponse, ClientError> {
        trace!("Creating folder {}...", &name);
        let url = format!("{}/v2/folders", self.base_url);
        let mut query_parameters: Vec<(String, String)> = Vec::new();
        query_parameters.push(("name".to_string(), name.clone()));

        let bearer: String = format!("Bearer {}", self.access_token);
        let response = self
            .client
            .post(url)
            .timeout(Duration::from_secs(180))
            .header("Authorization", bearer)
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            .header("Content-Length", 0)
            .query(&query_parameters)
            .send()?;

        let status = response.status();
        self.evaluate_satus(status)?;
        let json = response.text().unwrap();
        trace!("{}", json);
        let result: FolderCreateResponse = serde_json::from_str(&json)?;

        Ok(result)
    }

    pub fn get_model(&self, uuid: &Uuid) -> Result<SingleModelResponse, ClientError> {
        let url = format!(
            "{}/v2/models/{id}",
            self.base_url,
            id = urlencode(uuid.to_string())
        );
        trace!("Reading model {}...", uuid.to_string());

        let json = self.get(url.as_str(), None)?;
        //trace!("{}", json);
        let result: SingleModelResponse = serde_json::from_str(&json)?;
        Ok(result)
    }

    pub fn delete_model(&self, uuid: &Uuid) -> Result<(), ClientError> {
        let url = format!(
            "{}/v2/models/{id}",
            self.base_url,
            id = urlencode(uuid.to_string())
        );
        trace!("Deleting model {}...", uuid.to_string());

        let builder = self
            .client
            .request(reqwest::Method::DELETE, url)
            .timeout(Duration::from_secs(180))
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", self.tenant.to_owned());

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        let response = self.client.execute(request)?;
        self.evaluate_satus(response.status())?;
        Ok(())
    }

    pub fn reprocess_model(&self, uuid: &Uuid) -> Result<()> {
        let url = format!("{}/v1/{}/models/reprocess", self.base_url, self.tenant);
        let bearer: String = format!("Bearer {}", self.access_token);

        let form = Form::new().part("uuid", Part::text(uuid.to_string()));

        trace!("Reprocessing model {}", url);

        let response = self
            .client
            .post(url)
            .timeout(Duration::from_secs(180))
            .header("Authorization", bearer)
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            .multipart(form)
            .send()?;

        let status = self.evaluate_satus(response.status());
        let json = response.text().unwrap();
        trace!("{}", json);

        match status {
            Ok(_) => Ok(()),
            Err(e) => return Err(anyhow!(e)),
        }
    }

    pub fn get_model_metadata(&self, uuid: &Uuid) -> Result<Option<ModelMetadata>, ClientError> {
        let url = format!(
            "{}/v2/models/{id}/metadata",
            self.base_url,
            id = urlencode(uuid.to_string())
        );

        let mut query_parameters: Vec<(String, String)> = Vec::new();
        let per_page = 10000;
        let page = 1;
        query_parameters.push(("perPage".to_string(), per_page.to_string()));
        query_parameters.push(("page".to_string(), page.to_string()));

        let json = self.get(url.as_str(), Some(query_parameters))?;

        //trace!("{}", &json);
        let response: Option<ModelMetadataResponse> = serde_json::from_str(&json)?;

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

        let json = self.get(url.as_str(), None)?;
        //trace!("{}", json);
        let result: AssemblyTree = serde_json::from_str(&json)?;
        Ok(result)
    }

    pub fn get_list_of_models_page(
        &self,
        folders: Vec<u32>,
        search: Option<&String>,
        per_page: u32,
        page: u32,
    ) -> Result<ModelListResponse, ClientError> {
        let url = format!("{}/v2/models", self.base_url);

        let mut query_parameters: Vec<(String, String)> = Vec::new();

        for folder in folders {
            query_parameters.push(("folderIds".to_string(), folder.to_string()));
        }

        if search.is_some() {
            query_parameters.push(("search".to_string(), search.unwrap().to_owned()));
        }

        query_parameters.push(("perPage".to_string(), per_page.to_string()));
        query_parameters.push(("page".to_string(), page.to_string()));

        let json = self.get(url.as_str(), Some(query_parameters))?;

        //std::fs::write("./dump.json", &json).expect("Unable to write file");
        let result: ModelListResponse = serde_json::from_str::<ModelListResponse>(&json)?;

        //trace!("Parsed OK");
        Ok(result)
    }

    pub fn upload_file_chunk(
        &self,
        folder_id: u32,
        file: &str,
        source_id: &str,
        batch_uuid: Uuid,
        units: &str,
        start_index: u64,
        end_index: u64,
        file_size: u64,
        bytes: Box<Vec<u8>>,
    ) -> Result<Option<Model>> {
        trace!("Uploading file chunk...");

        let url = format!("{}/v1/{}/models", self.base_url, self.tenant);
        let bearer: String = format!("Bearer {}", self.access_token);
        let file_name = Path::new(&file.to_owned())
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        let form = Form::new()
            .part("file", Part::bytes(*bytes).file_name(file_name))
            .part("units", Part::text(units.to_owned()))
            .part("containerId", Part::text(folder_id.to_string()))
            .part("sourceId", Part::text(source_id.to_owned()))
            .part("fileSize", Part::text(file_size.to_string()))
            .part("batch", Part::text(batch_uuid.to_string()));

        let mut range_value = String::from("bytes ");
        range_value.push_str(start_index.to_string().as_str());
        range_value.push_str("-");
        range_value.push_str(end_index.to_string().as_str());
        range_value.push_str("/");
        range_value.push_str(file_size.to_string().as_str());

        trace!("Uploading {}...", range_value);
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
            .multipart(form)
            .send();

        let json = match response {
            Ok(response) => response.text(),
            Err(e) => return Err(anyhow!(e)),
        };

        let json = json.unwrap();
        //trace!("{}", json);
        let mut model: FileUploadResponse = serde_json::from_str(&json)?;

        let attachment_url = model.attachment_url.to_owned();
        match attachment_url {
            Some(attachment_url) => {
                let pos = attachment_url.rfind('/');
                if pos.is_some() {
                    let pos = pos.unwrap() + 1;
                    let short_id = attachment_url.as_str().substring(pos, attachment_url.len());
                    let short_id = short_id.parse::<u64>()?;
                    model.short_id = Some(short_id.to_owned());
                };
            }
            None => (),
        }

        let model = Model::from(model);
        Ok(Some(model))
    }

    pub fn get_list_of_properties(&self) -> Result<PropertyCollection, ClientError> {
        let url = format!("{}/v2/metadata-keys", self.base_url);
        let json = self.get(url.as_str(), None)?;
        //trace!("{}", json);
        let result: PropertyCollection = serde_json::from_str(&json)?;

        Ok(result)
    }

    pub fn post_property(&self, name: &String) -> Result<Property> {
        let url = format!("{}/v2/metadata-keys", self.base_url);
        let bearer: String = format!("Bearer {}", self.access_token);

        trace!(
            "Registering a new property with name of \"{}\"...",
            name.clone()
        );
        trace!("POST {}", url);

        let request = PropertyRequest::new(name.to_owned());
        trace!("Request: {:?}", &request);

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
            .send()?;

        let status = self.evaluate_satus(response.status());
        let json = response.text().unwrap();
        //trace!("{}", json);

        match status {
            Ok(_) => (),
            Err(e) => return Err(anyhow!(e)),
        }

        let result: PropertyResponse = serde_json::from_str(&json)?;
        Ok(result.property)
    }

    pub fn put_model_property(
        &self,
        model_uuid: &Uuid,
        id: &u64,
        item: &ModelMetadataItem,
    ) -> Result<ModelMetadataItem> {
        let url = format!("{}/v2/models/{}/metadata/{}", self.base_url, model_uuid, id);
        let bearer: String = format!("Bearer {}", self.access_token);

        trace!("PUT {}", url);

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
            .send()?;

        let status = self.evaluate_satus(response.status());
        let json = response.text().unwrap();
        trace!("{}", json);

        match status {
            Ok(_) => (),
            Err(e) => return Err(anyhow!(e)),
        }

        let result: ModelCreateMetadataResponse = serde_json::from_str(&json)?;
        Ok(result.metadata)
    }

    pub fn delete_model_property(&self, model_uuid: &Uuid, id: &u64) -> Result<()> {
        let url = format!("{}/v2/models/{}/metadata/{}", self.base_url, model_uuid, id);
        let bearer: String = format!("Bearer {}", self.access_token);

        trace!("DELETE {}", url);

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
            .send()?;

        let status = self.evaluate_satus(response.status());
        let json = response.text().unwrap();
        trace!("{}", json);

        match status {
            Ok(_) => (),
            Err(e) => return Err(anyhow!(e)),
        }

        Ok(())
    }

    pub fn create_image_classifier(
        &self,
        name: &String,
        folders: Vec<u32>,
    ) -> Result<ImageClassifierCreateResponse, ClientError> {
        let url = format!("{}/v1/{}/image-classifiers", self.base_url, self.tenant);
        let bearer: String = format!("Bearer {}", self.access_token);

        let req = ImageClassifierCreateRequest::new(name.clone(), folders);
        trace!("POST {}", url);

        let response = self
            .client
            .post(url)
            .timeout(Duration::from_secs(180))
            .header("Authorization", bearer)
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            .json(&req)
            .send()?;

        self.evaluate_satus(response.status())?;
        let json = response.text().unwrap();
        trace!("{}", json);
        let result: ImageClassifierCreateResponse = serde_json::from_str(&json)?;

        Ok(result)
    }

    pub fn get_image_classifiers(&self) -> Result<Vec<ImageClassifier>, ClientError> {
        let url = format!("{}/v1/{}/image-classifiers", self.base_url, self.tenant);
        trace!("GET {}", url.to_string());

        let json = self.get(url.as_str(), None)?;
        //trace!("{}", json);
        let result: Vec<ImageClassifier> = serde_json::from_str(&json)?;

        Ok(result)
    }

    pub fn get_classification_scores(
        &self,
        classifier_uuid: Uuid,
        file_name: String,
        bytes: Box<Vec<u8>>,
    ) -> Result<ListOfClassificationScores, ClientError> {
        let url = format!(
            "{}/v1/{}/image-classifiers/{}/predictions",
            self.base_url,
            self.tenant,
            classifier_uuid.to_string()
        );
        let bearer: String = format!("Bearer {}", self.access_token);

        trace!("POST {}", url);

        let f = file_name.to_owned();
        let form = Form::new()
            .part("classifierUuid", Part::text(classifier_uuid.to_string()))
            .part("image", Part::bytes(*bytes).file_name(f));

        let response = self
            .client
            .post(url)
            .timeout(Duration::from_secs(180))
            .header("Authorization", bearer)
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            .multipart(form)
            .send()?;

        self.evaluate_satus(response.status())?;
        let json = response.text().unwrap();
        trace!("{}", json);
        let result: ListOfClassificationScores = serde_json::from_str(&json)?;

        Ok(result)
    }

    pub fn get_geo_classifiers(&self) -> Result<ListOfGeoClassifiers, ClientError> {
        let url = format!("{}/v2/geo-classifiers", self.base_url);
        trace!("GET {}", url.to_string());

        let json = self.get(url.as_str(), None)?;
        //trace!("{}", json);
        let result: ListOfGeoClassifiers = serde_json::from_str(&json)?;

        Ok(result)
    }

    pub fn get_geo_labels(&self) -> Result<ListOfGeoLabels, ClientError> {
        let url = format!("{}/v2/geo-labels", self.base_url);
        trace!("GET {}", url.to_string());

        let json = self.get(url.as_str(), None)?;
        //trace!("{}", json);
        let result: ListOfGeoLabels = serde_json::from_str(&json)?;

        Ok(result)
    }

    pub fn get_geo_match_page(
        &self,
        uuid: &Uuid,
        label_id: &u32,
        threshold: &f64,
        per_page: u32,
        page: u32,
    ) -> Result<GeoMatchPageResponse, ClientError> {
        let url = format!(
            "{}/v2/models/{}/geo-label-matches/{}",
            self.base_url,
            urlencode(uuid.to_string()),
            label_id.to_string()
        );
        //trace!("GET {}", url.to_string());

        let mut query_parameters: Vec<(String, String)> = Vec::new();
        query_parameters.push(("threshold".to_string(), threshold.to_string()));
        query_parameters.push(("perPage".to_string(), per_page.to_string()));
        query_parameters.push(("page".to_string(), page.to_string()));

        let json = self.get(url.as_str(), Some(query_parameters))?;
        //trace!("{}", json);
        let result: GeoMatchPageResponse = serde_json::from_str(&json)?;
        Ok(result)
    }

    pub fn get_image_upload_specs(&self, path: &Path) -> Result<ImageUploadResponse> {
        if !path.is_file() {
            return Err(anyhow!("Input is not a file"));
        }

        let url = format!("{}/v2/images", self.base_url);
        let bearer: String = format!("Bearer {}", self.access_token);

        let filename = match path.file_name() {
            Some(filename) => filename.to_str().unwrap(),
            None => return Err(anyhow!("Error extracting file name from input path")),
        };

        trace!("Requesting upload specs for image {}", &filename);
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
    ) -> Result<()> {
        trace!("Uploading image file {}...", path.to_str().unwrap());
        //trace!("Upload URL: {}", url.to_string());

        let max_size = upload_size_requirements.max_size_in_bytes;
        let file = File::open(path)?;
        let file_size = file.metadata().unwrap().len();
        if file_size > max_size {
            return Err(anyhow!("File too large"));
        }

        //let curl = format!("curl -X PUT --upload-file {} -H 'Content-Type: {mime}' -H 'X-Goog-Content-Length-Range: {content_range}' {}", path.to_str().unwrap(), url.to_string());
        //trace!("Example: {}", curl);

        let response = self
            .client
            .put(url)
            .timeout(Duration::from_secs(180))
            .header("Content-Type", mime)
            .header("X-Goog-Content-Length-Range", content_range)
            .body(file)
            .send();

        let _json = match response {
            Ok(response) => response.text(),
            Err(e) => return Err(anyhow!(e)),
        };

        //let json = json.unwrap();
        //trace!("{}", json);

        Ok(())
    }

    fn get_image_search_matches_page(
        &self,
        id: String,
        search: Option<String>,
        filter: Option<String>,
        page: u32,
        per_page: u32,
    ) -> Result<ImageMatchPageResponse> {
        trace!("Searching matching models for image with ID {id}...");

        let url = format!("{}/v2/images/model-matches", self.base_url);
        let mut query_parameters: Vec<(String, String)> = Vec::new();
        query_parameters.push(("id".to_string(), id));
        query_parameters.push(("perPage".to_string(), per_page.to_string()));
        query_parameters.push(("page".to_string(), page.to_string()));
        match search {
            Some(search) => query_parameters.push(("search".to_string(), search)),
            None => (),
        }
        match filter {
            Some(filter) => query_parameters.push(("filter".to_string(), filter)),
            None => (),
        }

        let json = self.get(url.as_str(), Some(query_parameters));
        let json = match json {
            Ok(json) => json,
            Err(e) => return Err(anyhow!("Failed to find matches for image: {}", e)),
        };
        let result: ImageMatchPageResponse = serde_json::from_str(&json)?;
        Ok(result)
    }

    pub fn get_image_search_maches(
        &self,
        id: String,
        search: Option<String>,
        filter: Option<String>,
        max_matches: u32,
    ) -> Result<ListOfModels> {
        let mut page = 1;
        let per_page = 20;
        let mut models: Vec<Model> = Vec::new();

        trace!("Limit={}", max_matches);

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

            trace!("Page {}", page);
            trace!("size={}", local_size);
            trace!("models.size={}", models.len());

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
