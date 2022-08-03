use log::trace;
use anyhow::{
    anyhow,
    Result
};
use reqwest::{
    self, 
    blocking::{
        Client, 
        multipart::{
            Part,
            Form,
        },
    },
    StatusCode
};
use std::time::Duration;
use url;
use uuid::Uuid;
use serde::{
    Serialize, 
    Deserialize
};
use substring::Substring;
use crate::model::{
    Model,
    ModelMetadataItem,
    ModelMetadata,
    FileUploadResponse,
    PropertyCollection,
    Property,
};
use std::{
    collections::HashMap,
    path::Path
};

fn urlencode<T: AsRef<str>>(s: T) -> String {
    url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

#[derive(Clone, Debug)]
pub enum ClientError {
    Parsing(String),
    Unauthorized,
    Forbidden,
    NotFound,
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
            Self::Parsing(message) =>  write!(f, "Parsing error: {}", message),
            Self::Unauthorized => write!(f, "Request is unauthorized! Please, renew your access token"),
            Self::Forbidden => write!(f, "Request is forbidden!"),
            Self::NotFound => write!(f, "Resource not found!"),
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
    #[serde(rename = "filterData")]
    pub filter_data: FilterData,
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
    #[serde(rename = "modelProperties")]
    pub properties: Option<Vec<ModelMetadataItem>>,
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
    #[serde(rename = "propertyName")]
    name: String,
}

impl PropertyRequest {
    pub fn new(name: String) -> PropertyRequest {
        PropertyRequest {
            name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
struct PropertyValueRequest {
    value: String,
}

impl PropertyValueRequest {
    pub fn new(value: String) -> PropertyValueRequest {
        PropertyValueRequest {
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
struct PropertyResponse {
    #[serde(rename = "property")]
    property: Property,
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
        let client =  reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .unwrap();

        ApiClient {
            client: client,
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
            StatusCode::OK |
            StatusCode::CREATED |
            StatusCode::ACCEPTED |
            StatusCode::NON_AUTHORITATIVE_INFORMATION |
            StatusCode::NO_CONTENT |
            StatusCode::RESET_CONTENT => (), // Nothing to do, continue
            StatusCode::FORBIDDEN => {
                return Err(ClientError::Forbidden)
            },
            StatusCode::NOT_FOUND => {
                return Err(ClientError::NotFound)
            },
            StatusCode::UNAUTHORIZED => {
                return Err(ClientError::Unauthorized)            
            }
            StatusCode::CONTINUE |
            StatusCode::SWITCHING_PROTOCOLS |
            StatusCode::PROCESSING |
            StatusCode::PARTIAL_CONTENT |
            StatusCode::MULTI_STATUS |
            StatusCode::ALREADY_REPORTED |
            StatusCode::IM_USED |
            StatusCode::MULTIPLE_CHOICES |
            StatusCode::MOVED_PERMANENTLY |
            StatusCode::FOUND |
            StatusCode::SEE_OTHER |
            StatusCode::NOT_MODIFIED |
            StatusCode::USE_PROXY |
            StatusCode::TEMPORARY_REDIRECT |
            StatusCode::PERMANENT_REDIRECT |
            StatusCode::BAD_REQUEST | 
            StatusCode::PAYMENT_REQUIRED |
            StatusCode::METHOD_NOT_ALLOWED |
            StatusCode::NOT_ACCEPTABLE |
            StatusCode::PROXY_AUTHENTICATION_REQUIRED |
            StatusCode::REQUEST_TIMEOUT |
            StatusCode::CONFLICT |
            StatusCode::GONE |
            StatusCode::LENGTH_REQUIRED |
            StatusCode::PRECONDITION_FAILED |
            StatusCode::PAYLOAD_TOO_LARGE |
            StatusCode::URI_TOO_LONG |
            StatusCode::UNSUPPORTED_MEDIA_TYPE |
            StatusCode::RANGE_NOT_SATISFIABLE |
            StatusCode::EXPECTATION_FAILED |
            StatusCode::IM_A_TEAPOT |
            StatusCode::MISDIRECTED_REQUEST |
            StatusCode::UNPROCESSABLE_ENTITY | 
            StatusCode::LOCKED |
            StatusCode::FAILED_DEPENDENCY |
            StatusCode::UPGRADE_REQUIRED |
            StatusCode::PRECONDITION_REQUIRED |
            StatusCode::TOO_MANY_REQUESTS |
            StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE |
            StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS |
            StatusCode::INTERNAL_SERVER_ERROR |
            StatusCode::NOT_IMPLEMENTED |
            StatusCode::BAD_GATEWAY |
            StatusCode::SERVICE_UNAVAILABLE |
            StatusCode::GATEWAY_TIMEOUT |
            StatusCode::HTTP_VERSION_NOT_SUPPORTED |
            StatusCode::VARIANT_ALSO_NEGOTIATES |
            StatusCode::INSUFFICIENT_STORAGE |
            StatusCode::LOOP_DETECTED |
            StatusCode::NOT_EXTENDED |
            StatusCode::NETWORK_AUTHENTICATION_REQUIRED => {
                return Err(ClientError::Unsupported(format!("Status: {:?}", status)))
            },
            _ => {
                return Err(ClientError::Unsupported("Unexpected query status code".to_string()))
            },
        };

        Ok(())
    }

    pub fn get(&self, url: &str, query_parameters: Option<HashMap<String, String>>) -> Result<String, ClientError> {

        trace!("GET: {}", url);

        let mut builder = self.client.request(reqwest::Method::GET, url)
                        .timeout(Duration::from_secs(180))
                        .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
                        .header("X-PHYSNA-TENANTID", self.tenant.to_owned());

        match query_parameters {
            Some(query_parametes) => {
                for (key, value) in query_parametes {
                    builder = builder.query(&[(key.to_owned(), value.to_owned())]);
                }
            },
            None => (),
        }

        let request = builder.bearer_auth(self.access_token.to_owned()).build()?;
        let response = self.client.execute(request)?;
        
        trace!("Status: {}", response.status());

        self.evaluate_satus(response.status())?;

        let content = response.text()?;
        trace!("{}", content);
        Ok(content)
    }

    pub fn get_model_match_page(&self, uuid: &Uuid, threshold: f64, per_page: u32, page: u32) -> Result<PartToPartMatchResponse, ClientError> {
        let url = format!("{}/v2/models/{id}/part-to-part-matches", self.base_url, id=urlencode(uuid.to_string()));

        let mut query_parameters: HashMap<String, String> = HashMap::new();
        query_parameters.insert("threshold".to_string(), threshold.to_string());
        query_parameters.insert("perPage".to_string(), per_page.to_string());
        query_parameters.insert("page".to_string(), page.to_string());

        let json = self.get(url.as_str(), Some(query_parameters))?;
        //trace!("{}", json);
        let result: PartToPartMatchResponse = serde_json::from_str(&json)?;
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

    pub fn get_model(&self, uuid: &Uuid) -> Result<SingleModelResponse, ClientError> {
        let url = format!("{}/v2/models/{id}", self.base_url, id=urlencode(uuid.to_string()));
        trace!("Reading model {}...", uuid.to_string());
        
        let json = self.get(url.as_str(), None)?;
        //trace!("{}", json);
        let result: SingleModelResponse = serde_json::from_str(&json)?;
        Ok(result)
    }

    pub fn delete_model(&self, uuid: &Uuid) -> Result<(), ClientError> {
        let url = format!("{}/v2/models/{id}", self.base_url, id=urlencode(uuid.to_string()));
        trace!("Deleting model {}...", uuid.to_string());
        
        let builder = self.client.request(reqwest::Method::DELETE, url)
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

        trace!("POST {}", url);

        let response = self.client.post(url)
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
        let url = format!("{}/v2/models/{id}/properties", self.base_url, id=urlencode(uuid.to_string()));

        let json = self.get(url.as_str(), None)?;
        //trace!("{}", &json);
        let response: Option<ModelMetadataResponse> = serde_json::from_str(&json)?;

        match response {
            Some(response) => {
                if response.properties.is_some() {
                    let props: HashMap<u64, ModelMetadataItem> = response.properties.unwrap().into_iter().map(|property| (property.id, ModelMetadataItem::new(property.model_uuid, property.id, property.name, property.value))).collect();
                    return Ok(Some(ModelMetadata::new(props)));
                } else {
                    return Ok(None);
                }
            },
            None => Ok(None),
        }
    }

    pub fn get_assembly_tree_for_model(&self, uuid: &Uuid) -> Result<AssemblyTree, ClientError> {
        let url = format!("{}/v2/models/{id}/assembly-tree", self.base_url, id=urlencode(uuid.to_string()));

        let json = self.get(url.as_str(), None)?;
        //trace!("{}", json);
        let result: AssemblyTree = serde_json::from_str(&json)?;
        Ok(result)
    }

    pub fn get_list_of_models_page(&self, folders: Option<Vec<u32>>, search: Option<String>, per_page: u32, page: u32) -> Result<ModelListResponse, ClientError> {
        let url = format!("{}/v2/models", self.base_url);

        let mut query_parameters: HashMap<String, String> = HashMap::new();

        if folders.is_some() {
            let folder_ids_str = folders.unwrap().into_iter().map(|folder_id| folder_id.to_string()).collect::<Vec<String>>().join(",").to_string();
            query_parameters.insert("folderIds".to_string(), folder_ids_str);
        }
        if search.is_some() {
            query_parameters.insert("search".to_string(), search.unwrap().to_owned());
        }
        query_parameters.insert("perPage".to_string(), per_page.to_string());
        query_parameters.insert("page".to_string(), page.to_string());

        let json = self.get(url.as_str(), Some(query_parameters))?;
        //trace!("{}", json);
        let result: ModelListResponse = serde_json::from_str(&json)?;
        Ok(result)
    }

    pub fn upload_file_chunk(&self, folder_id: u32, file: &str, source_id: &str, batch_uuid: Uuid, units: &str, start_index: u64, end_index: u64, file_size: u64, bytes: Box<Vec<u8>>) -> Result<Option<Model>> {    
        
        let url = format!("{}/v1/{}/models", self.base_url, self.tenant);
        let bearer: String = format!("Bearer {}", self.access_token);
        let file_name = Path::new(&file.to_owned()).file_name().unwrap().to_os_string().into_string().unwrap();

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
        let response = self.client.post(url)
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

        let attachment_url =  model.attachment_url.to_owned();
        match attachment_url {
            Some(attachment_url) => {
                let pos = attachment_url.rfind('/');
                if pos.is_some() {
                    let pos = pos.unwrap() + 1;
                    let short_id = attachment_url.as_str().substring(pos, attachment_url.len());
                    let short_id = short_id.parse::<u64>()?;
                    model.short_id = Some(short_id.to_owned());
                };
            },
            None => (),
        }

        let model = Model::from(model);
        Ok(Some(model))
    }

    pub fn get_list_of_properties(&self) -> Result<PropertyCollection, ClientError> {
        let url = format!("{}/v2/properties", self.base_url);
        trace!("GET {}", url.to_string());

        let json = self.get(url.as_str(), None)?;
        //trace!("{}", json);
        let result: PropertyCollection = serde_json::from_str(&json)?;

        Ok(result)
    }

    pub fn post_property(&self, name: &String) -> Result<Property> {
        let url = format!("{}/v2/properties", self.base_url);
        let bearer: String = format!("Bearer {}", self.access_token);

        trace!("POST {}", url);

        let response = self.client.post(url)
            .timeout(Duration::from_secs(180))
            .header("Authorization", bearer)
            .header("cache-control", "no-cache")
            .header(reqwest::header::USER_AGENT, APP_USER_AGENT)
            .header("X-PHYSNA-TENANTID", &self.tenant)
            .header("scope", "tenantApp")
            //.header("Content-Range", range_value.to_owned())
            .json(&PropertyRequest::new(name.to_owned()))
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

    pub fn put_model_property(&self, item: &ModelMetadataItem) -> Result<ModelMetadataItem> {
        let url = format!("{}/v2/models/{}/properties/{}", self.base_url, item.model_uuid, item.id);
        let bearer: String = format!("Bearer {}", self.access_token);
    
        trace!("PUT {}", url);
    
        let response = self.client.put(url)
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
        //trace!("{}", json);
    
        match status {
            Ok(_) => (),
            Err(e) => return Err(anyhow!(e)),
        }
    
        let result: ModelMetadataItem = serde_json::from_str(&json)?;
        Ok(result)
    }
}

