use csv::{
    WriterBuilder,
    Terminator
};
use std::io::BufWriter;
use serde::{Serialize, Deserialize};
use std::iter::Extend;
use std::iter::IntoIterator;
use std::collections::HashMap;
use ptree::TreeItem;
use ptree::style::Style;
use std::io;
use std::borrow::Cow;
use anyhow::Result;
use std::hash::{Hash, Hasher};
use crate::client;
use uuid::Uuid;
use petgraph::matrix_graph::MatrixGraph;

#[derive(Clone, Debug)]
pub struct Configuration {
    pub base_url: String,
    pub access_token: String,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ResponseContent {
    #[serde(rename = "status")]
    pub status: u32, 
    #[serde(rename = "content")]
    pub content: String,
}

/// Marshals the state into JSON
pub trait ToJson {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error>;
}

/// Marshals the state into CSV
pub trait ToCsv{
    fn to_csv(&self, pretty: bool) -> anyhow::Result<String>;
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Folder {
    #[serde(rename = "id")]
    pub id: u32,
    #[serde(rename = "name")]
    pub name: String,
}

impl Folder {
    pub fn new(id: u32, name: String) -> Self {
        Folder{id, name}
    }
}

impl From<FolderCreateResponse> for Folder {
    fn from(response: FolderCreateResponse) -> Self {
        Folder::new(response.container_id, response.name)
    }
}

impl ToJson for Folder {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }
}

impl ToCsv for Folder {
    fn to_csv(&self, pretty: bool) -> anyhow::Result<String> {

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new().terminator(Terminator::CRLF).from_writer(buf);

        if pretty {
            let columns = vec!["ID", "NAME"];
            writer.write_record(&columns)?;
        }
    
        let mut values: Vec<String> = Vec::new();
    
        values.push(self.id.to_string());
        values.push(self.name.to_owned());
        writer.write_record(&values)?;
       
        writer.flush()?;
    
        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)        
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ListOfFolders {
    #[serde(rename = "folders")]
    pub folders: Vec<Folder>,
}

impl ToJson for ListOfFolders {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        let folders = self.folders.clone();
        if pretty {
            serde_json::to_string_pretty(&folders)
        } else {
            serde_json::to_string(&folders)
        }
    }
}

impl ToCsv for ListOfFolders {
    fn to_csv(&self, pretty: bool) -> anyhow::Result<String> {
        let folders = self.folders.clone();

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new().terminator(Terminator::CRLF).from_writer(buf);

        if pretty {
            let columns = vec!["ID", "NAME"];
            writer.write_record(&columns)?;
        }
    
        for folder in folders {
            let mut values: Vec<String> = Vec::new();
    
            values.push(folder.id.to_string());
            values.push(folder.name);
            writer.write_record(&values)?;
        }
       
        writer.flush()?;
    
        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)        
    }
}

impl From<Vec<Folder>> for ListOfFolders {
    fn from(folders: Vec<Folder>) -> Self {
        let folders = folders.into_iter().map(|f| Folder::from(f)).collect();
        ListOfFolders{folders: folders}
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Model {
    #[serde(rename = "id")]
    pub uuid: Uuid,
    #[serde(rename = "isAssembly")]
    pub is_assembly: bool,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "folderId")]
    pub folder_id: u32,
    #[serde(rename = "ownerId")]
    pub owner_id: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "fileType")]
    pub file_type: String,
    #[serde(rename = "thumbnail", skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(rename = "units")]
    pub units: String,
    #[serde(rename = "state")]
    pub state: String,
    #[serde(rename = "attachmentUrl")]
    pub attachment_url: Option<String>,
    #[serde(rename = "shortId")]
    pub short_id: Option<u64>,
    #[serde(rename = "metadata")]
    pub metadata: Option<ModelMetadata>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "id")]
    pub id: u64,
    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Pair {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PropertyCollection {
    #[serde(rename = "properties")]
    pub properties: Vec<Property>,
}

impl ToJson for PropertyCollection {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }
}

impl ToCsv for PropertyCollection {
    fn to_csv(&self, pretty: bool) -> anyhow::Result<String> {

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new().terminator(Terminator::CRLF).from_writer(buf);

        if pretty {
            let columns = vec!["ID", "NAME"];
            writer.write_record(&columns)?;
        }
    
        for property in &self.properties {
            let mut values: Vec<String> = Vec::new();
            values.push(property.id.to_string());
            values.push(property.name.to_owned());
            writer.write_record(&values)?;
        }
        writer.flush()?;
    
        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)        
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelMetadataItem {
    #[serde(rename = "modelId")]
    pub model_uuid: Uuid,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "value")]
    pub value: String,
}

impl ModelMetadataItem {
    pub fn new(model_uuid: Uuid, name: String, value: String) -> ModelMetadataItem {
        ModelMetadataItem {
            model_uuid,
            name,
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelMetadata {
    #[serde(rename = "metadata")]
    pub properties: Vec<ModelMetadataItem>,
}

impl ModelMetadata {
    pub fn new(properties: Vec<ModelMetadataItem>) -> ModelMetadata {
        ModelMetadata {
            properties,
        }
    }
}

impl ToJson for ModelMetadata {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }
}

impl ToCsv for ModelMetadata {
    fn to_csv(&self, pretty: bool) -> anyhow::Result<String> {

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new().terminator(Terminator::CRLF).from_writer(buf);

        if pretty {
            let columns = vec!["MODEL_UUID", "NAME", "VALUE"];
            writer.write_record(&columns)?;
        }
    
        for property in &self.properties {
            let mut values: Vec<String> = Vec::new();
            values.push(property.model_uuid.to_string());
            values.push(property.name.to_owned());
            values.push(property.value.to_owned());
            writer.write_record(&values)?;
        }
        writer.flush()?;
    
        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)        
    }
}

impl From<FileUploadResponse> for Model {
    fn from(response: FileUploadResponse) -> Self {
       Model {
        uuid: response.uuid,
        is_assembly: response.is_assembly,
        name: response.name,
        folder_id: response.folder_id,
        owner_id: "".to_string(),
        created_at: response.created_at,
        file_type: "".to_string(),
        thumbnail: response.thumbnail,
        units: response.units,
        state: response.state,
        attachment_url: response.attachment_url,
        short_id: response.short_id,
        metadata: None,
       }
    }
}

impl ToJson for Model {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }
}

impl ToCsv for Model {
    fn to_csv(&self, pretty: bool) -> anyhow::Result<String> {

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new().terminator(Terminator::CRLF).from_writer(buf);

        if pretty {
            let columns = vec!["ID", "NAME", "FOLDER_ID", "IS_ASSEMBLY", "FILE_TYPE", "UNITS", "STATE"];
            writer.write_record(&columns)?;
        }
    
        let mut values: Vec<String> = Vec::new();
    
        values.push(self.uuid.to_string());
        values.push(self.name.to_owned());
        values.push(self.folder_id.to_string());
        values.push(self.is_assembly.to_string());
        values.push(self.file_type.to_string());
        values.push(self.units.to_owned());
        values.push(self.state.to_owned());
        writer.write_record(&values)?;
       
        writer.flush()?;
    
        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)        
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ListOfModels {
    #[serde(rename = "models")]
    pub models: Vec<Model>,
}

impl ToCsv for ListOfModels {
    fn to_csv(&self, pretty: bool) -> anyhow::Result<String> {
        let models = self.models.clone();

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new().terminator(Terminator::CRLF).from_writer(buf);

        if pretty {
            let columns = vec!["ID", "NAME", "FOLDER_ID", "IS_ASSEMBLY", "FILE_TYPE", "UNITS", "STATE"];
            writer.write_record(&columns)?;
        }
    
        for model in models {
            let mut values: Vec<String> = Vec::new();
    
            values.push(model.uuid.to_string());
            values.push(model.name);
            values.push(model.folder_id.to_string());
            values.push(model.is_assembly.to_string());
            values.push(model.file_type.to_string());
            values.push(model.units);
            values.push(model.state);
            writer.write_record(&values)?;
        }
       
        writer.flush()?;
    
        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)        
    }
}

impl ToJson for ListOfModels {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        let models = self.models.clone();
        if pretty {
            serde_json::to_string_pretty(&models)
        } else {
            serde_json::to_string(&models)
        }
    }
}

impl From<Vec<Model>> for ListOfModels {
    fn from(physna_list_of_models_response: Vec<Model>) -> Self {
        let models = physna_list_of_models_response.into_iter().map(|m| Model::from(m)).collect();
        ListOfModels{models: models}
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelAssemblyTree {
    #[serde(rename = "model")]
    pub model: Model,
    #[serde(rename = "children", skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ModelAssemblyTree>>,
}

impl ModelAssemblyTree {
    pub fn new(model: Model, children: Option<Vec<ModelAssemblyTree>>) -> ModelAssemblyTree {
        ModelAssemblyTree {
            model,
            children,
        }
    }
}

impl ToJson for ModelAssemblyTree {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }
}

impl TreeItem for ModelAssemblyTree {
    type Child = Self;

    fn write_self<W: io::Write>(&self, f: &mut W, style: &Style) -> io::Result<()> {
        write!(f, "{}:[{}]", style.paint(self.model.name.clone()), style.paint(self.model.uuid.clone()))
    }

    fn children(&self) -> Cow<[Self::Child]> {
        if self.children.is_none() {
            Cow::from(vec![])
        } else {
            Cow::from(self.children.clone().unwrap())
        }
    }
}

// #[derive(Clone, Debug, Default)]
// pub struct AssemblyGraph {
//     pub flat_bom: IndexMap<Uuid, NodeIndex>,
//     pub graph: MatrixGraph<(), f64>,
//     last_node_index: AtomicUsize,
// }

// impl AssemblyGraph {
//     pub fn new() -> AssemblyGraph {
//         AssemblyGraph { 
//             flat_bom: IndexMap::new(),
//             graph: MatrixGraph::new(),
//             last_node_index: AtomicUsize::new(1),
//         }
//     }

//     pub fn extend_with_tree(&mut self, flat_bom: &mut ModelNodeIndexMap, assebmly_tree: &AssemblyTree) -> Result<NodeIndex<u16>> {

//         type Edge = (u16, u16, f64);
//         let mut edges: Vec<Edge> = Vec::new();
//         let root_uuid = Uuid::parse_str(assebmly_tree.uuid.as_str())?;
//         let root_index = flat_bom.get(&root_uuid);
//         let root_node_index: ModelNodeIndex = match root_index {
//             Some(root_index) => root_index.to_owned(),
//             None => {
//                 let index = NodeIndex::new(self.last_node_index.fetch_add(1, Ordering::SeqCst));
//                 flat_bom.inner.insert(ModelNodeIndex::new(root_uuid, name.to_owned(), index.to_owned()));
//                 index
//             },
//         };

//         // Add root vertex
//         edges.push((root_node_index.index() as u16, root_node_index.index() as u16, 1.0));
//         //self.graph.add_edge(root_node_index, root_node_index, 1.0);

//         // Add all children
//         match &assebmly_tree.children {
//             Some(children) => {
//                 for child in children {
//                     let child_node_index = self.extend_with_tree(flat_bom, child)?;
//                     self.graph.add_edge(root_node_index, child_node_index, 1.0);
//                 }
//             },
//             None => (),
//         }

//         Ok(root_node_index)
//     }
// }

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FlatBom {
    #[serde(rename = "models")]
    pub inner: HashMap<String, Model>,
}

impl FlatBom {
    pub fn new(elements: HashMap<String, Model>) -> Self {
        FlatBom{
            inner: elements.to_owned(),
        }
    }

    pub fn empty() -> Self {
        FlatBom{
            inner: HashMap::new(),
        }
    }

    pub fn extend(&mut self, bom: &FlatBom) {
        self.inner.extend(bom.inner.to_owned());
    }
}

impl From<ModelAssemblyTree> for FlatBom {
    fn from(assembly_tree: ModelAssemblyTree) -> Self {
        let mut items: HashMap<String, Model> = HashMap::new();

        // Insert the model of the root assembply itself
        items.insert(assembly_tree.model.uuid.to_string(), assembly_tree.model.to_owned());

        // Recursivelly insert the models of all children models
        match assembly_tree.children {
            Some(children) => {
                for child in children {
                    let sub_bom = FlatBom::from(child);
                    items.extend(sub_bom.inner);
                }
            },
            None => {
                // Nothing to do here. The root assembly already added above.
            }
        }

       FlatBom::new(items)
    }
}

impl ToJson for FlatBom {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }
}

impl ToCsv for FlatBom {
    fn to_csv(&self, pretty: bool) -> anyhow::Result<String> {
        let models = self.inner.clone();

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new().terminator(Terminator::CRLF).from_writer(buf);

        if pretty {
            let columns = vec!["UUID", "NAME"];
            writer.write_record(&columns)?;
        }
    
        for (uuid, model) in models {
            let mut values: Vec<String> = Vec::new();
            values.push(uuid);
            values.push(model.name.to_owned());
            writer.write_record(&values)?;
        }
       
        writer.flush()?;
    
        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)        
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModelMatch {
    #[serde(rename = "model")]
    pub model: Model,
    #[serde(rename = "percentage")]
    pub percentage: f64,
    #[serde(rename = "comparisonUrl")]
    pub comparison_url: Option<String>,
    #[serde(rename = "modelOneThumbnail")]
    pub model_one_thumbnail: Option<String>,
    #[serde(rename = "modelTwoThumbnail")]
    pub model_two_thumbnail: Option<String>,
}

impl PartialEq for ModelMatch {
    fn eq(&self, other: &Self) -> bool {
        self.model.name.eq(&other.model.name)
    }
}
impl Eq for ModelMatch {}

impl ModelMatch {
    pub fn new(model: Model, percentage: f64, comparison_url: Option<String>, model_one_thumbnail: Option<String>, model_two_thumbnail: Option<String>) -> ModelMatch {
        ModelMatch {
            model,
            percentage,
            comparison_url,
            model_one_thumbnail,
            model_two_thumbnail,
        }
    }
}

#[derive(Debug)]
pub struct ListOfModelMatches {
    pub inner: Box<Vec<ModelMatch>>,
}

impl ListOfModelMatches {
    pub fn new(matches: Box<Vec<ModelMatch>>) -> ListOfModelMatches {
        ListOfModelMatches{
            inner: matches,
        }
    }
}

impl ToJson for ListOfModelMatches {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(&self.inner)
        } else {
            serde_json::to_string(&self.inner)
        }
    }
}

impl ToCsv for ListOfModelMatches {
    fn to_csv(&self, pretty: bool) -> anyhow::Result<String> {

        let matches = *self.inner.clone();
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new().terminator(Terminator::CRLF).from_writer(buf);

        if pretty {
            let columns = vec!["MATCH_PERCENTAGE", "ID", "NAME", "FOLDER_ID", "IS_ASSEMBLY", "FILE_TYPE", "UNITS", "STATE"];
            writer.write_record(&columns)?;
        }
    
        for m in matches {
            let model = m.model;
            let percentage = m.percentage;
            let mut values: Vec<String> = Vec::new();
    
            values.push(format!("{:.4}", percentage));
            values.push(model.uuid.to_string());
            values.push(model.name);
            values.push(model.folder_id.to_string());
            values.push(model.is_assembly.to_string());
            values.push(model.file_type.to_string());
            values.push(model.units);
            values.push(model.state);
            writer.write_record(&values)?;
        }
       
        writer.flush()?;
    
        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)        
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelNodeIndex {
    #[serde(rename = "uuid")]
    pub uuid: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "nodeIndex")]
    pub node_index: u16,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelMatchReportItem {
    #[serde(rename = "uuid")]
    pub uuid: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "fodler_id")]
    pub folder_id: u32,
    #[serde(rename = "matches")]
    pub matches: Vec<ModelMatch>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SimpleDuplicatesMatchReport {
    #[serde(rename = "matches")]
    pub inner: HashMap<String, ModelMatchReportItem>,
}

impl SimpleDuplicatesMatchReport {
    pub fn new() -> Self {
        SimpleDuplicatesMatchReport {
            inner: HashMap::new(),
        }
    }
}

impl ToJson for SimpleDuplicatesMatchReport {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(&self.inner)
        } else {
            serde_json::to_string(&self.inner)
        }
    }
}

impl ToCsv for SimpleDuplicatesMatchReport {
    fn to_csv(&self, pretty: bool) -> anyhow::Result<String> {

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new().terminator(Terminator::CRLF).from_writer(buf);

        if pretty {
            let columns = vec!["MODEL_NAME", "MATCHING_MODEL_NAME", "MATCH", "SOURCE_UUID", "MATCHING_UUID", "SOURCE_FOLDER_ID", "MATCHING_FOLDER_ID", "COMPARISON_URL", "MODEL_ONE_THUMBNAIL_URL", "MODEL_TWO_THUMBNAIL_URL"];
            writer.write_record(&columns)?;
        }
    
        for (_uuid, item) in &self.inner {
            
            let model_name = item.name.to_owned();
            let source_uuid = item.uuid.to_string();
            let source_folder_id = item.folder_id.to_string();
            
            for m in &item.matches {
                let mut values: Vec<String> = Vec::new();

                values.push(model_name.to_owned());
                values.push(m.model.name.to_owned());
                values.push(m.percentage.to_string());
                values.push(source_uuid.to_owned());
                values.push(m.model.uuid.to_string());
                values.push(source_folder_id.to_owned());
                values.push(m.model.folder_id.to_string());

                match &m.comparison_url {
                    Some(url) => values.push(url.to_owned()),
                    None => values.push("".to_string()),
                }
                match &m.model_one_thumbnail {
                    Some(thumb) => values.push(thumb.to_owned()),
                    None => values.push("".to_string()),
                }
                match &m.model_two_thumbnail {
                    Some(thumb) => values.push(thumb.to_owned()),
                    None => values.push("".to_string()),
                }

                writer.write_record(&values)?;
            }
        }
        writer.flush()?;
    
        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)        
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelStatusRecord {
    pub folder_id: u32,
    pub folder_name: String,
    pub file_type: String,
    pub state: String,
    pub count: u64,
}

impl ModelStatusRecord {
    pub fn new(folder_id: u32, folder_name: String, file_type: String, state: String, count: u64) -> Self {
        ModelStatusRecord{
            folder_id,
            folder_name,
            file_type,
            state,
            count,
        }
    }
}

impl Hash for ModelStatusRecord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.folder_id.hash(state);
        self.file_type.hash(state);
        self.state.hash(state);
    }
}

pub struct EnvironmentStatusReport {
    pub stats: Vec<ModelStatusRecord>,
}

impl EnvironmentStatusReport {
    pub fn new() -> Self {
        EnvironmentStatusReport{
            stats: Vec::new(),
        }
    }
}

impl ToJson for EnvironmentStatusReport {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(&self.stats)
        } else {
            serde_json::to_string(&self.stats)
        }
    }
}

impl ToCsv for EnvironmentStatusReport {
    fn to_csv(&self, pretty: bool) -> anyhow::Result<String> {

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new().terminator(Terminator::CRLF).from_writer(buf);

        if pretty {
            let columns = vec!["FOLDER_ID", "FOLDER_NAME", "FILE_TYPE", "STATE", "COUNT"];
            writer.write_record(&columns)?;
        }
    
        for stat in &self.stats {
            
            let folder_id = stat.folder_id.to_string().to_owned();
            let folder_name = stat.folder_name.to_owned();
            let file_type = stat.file_type.to_owned();
            let state = stat.state.to_owned();
            let count = stat.count.to_string().to_owned();
            
            let mut values: Vec<String> = Vec::new();
            values.push(folder_id);
            values.push(folder_name);
            values.push(file_type);
            values.push(state);
            values.push(count);

            writer.write_record(&values)?;
        }
        writer.flush()?;
    
        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)        
    }
}

impl From<client::Folder> for Folder {
    fn from(folder: client::Folder) -> Self {
        Folder::new(
            folder.id,
            folder.name,
        )
    }
}

impl From<client::FolderListResponse> for ListOfFolders {
    fn from(response: client::FolderListResponse) -> Self {
        let folders = response.folders.into_iter().map(|f| Folder::from(f)).collect();
        ListOfFolders{folders: folders}
    }
}

impl From<client::SingleModelResponse> for Model {
    fn from(response: client::SingleModelResponse) -> Self {
        Model {
            uuid: response.model.uuid,
            is_assembly: response.model.is_assembly,
            name: response.model.name,
            folder_id: response.model.folder_id,
            file_type: response.model.file_type,
            thumbnail: response.model.thumbnail,
            owner_id: response.model.owner_id,
            created_at: response.model.created_at,
            units: response.model.units,
            state: response.model.state,
            short_id: response.model.short_id,
            attachment_url: response.model.attachment_url,
            metadata: None,
        }
    }
}

impl From<client::PartToPartMatch> for ModelMatch {
    fn from(m: client::PartToPartMatch) -> Self {
        let model = Model::from((m.matched_model).clone());
        let percentage = m.match_percentage;
        let model_one_thumbnail = model.thumbnail.clone();
        let model_two_thumbnail = m.matched_model.thumbnail.clone();
        ModelMatch::new(model, percentage, None, model_one_thumbnail, model_two_thumbnail)
    }
}

pub struct ModelMatchReport {
    pub duplicates: SimpleDuplicatesMatchReport,
    pub dictionary: HashMap<Uuid, PartNodeDictionaryItem>,
    pub graph: MatrixGraph<String, f64>,
    //pub matrix: Compressed<f64>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PartNodeDictionaryItem {
    pub name: String,
    pub node: usize,
    pub uuid: Uuid,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FileUploadResponse {
    #[serde(rename = "uuid")]
    pub uuid: Uuid,
    #[serde(rename = "isAssembly")]
    pub is_assembly: bool,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "containerId")]
    pub folder_id: u32,
    #[serde(rename = "created")]
    pub created_at: String,
    #[serde(rename = "thumbnail", skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(rename = "units")]
    pub units: String,
    #[serde(rename = "processedState")]
    pub state: String,
    #[serde(rename = "attachmentUrl")]
    pub attachment_url: Option<String>,
    #[serde(rename = "shortId")]
    pub short_id: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FolderCreateResponse {
    #[serde(rename = "ContainerId")]
    pub container_id: u32,
    #[serde(rename = "Name")]
    pub name: String
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelCreateMetadataResponse {
    #[serde(rename = "metadata")]
    pub metadata: ModelMetadataItem
}