use crate::client;
use csv::{Terminator, Writer, WriterBuilder};
use log::trace;
use petgraph::matrix_graph::MatrixGraph;
use ptree::style::Style;
use ptree::TreeItem;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::io;
use std::io::BufWriter;
use std::iter::Extend;
use std::iter::IntoIterator;
use std::vec::IntoIter;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("CSV parsing error")]
    CsvParsingError(#[from] csv::Error),
    #[error("Failed to extract value")]
    FailedToExtractCsvValue(#[from] csv::IntoInnerError<csv::Error>),
    #[error("I/O error")]
    InputOutputError(#[from] std::io::Error),
    #[error("Failed to extract value from byte buffer")]
    FailedToExtractValueFromByteBuffer(#[from] std::io::IntoInnerError<BufWriter<Vec<u8>>>),
    #[error("Failed to extract value from CSV buffer")]
    FailedToExtractValueFromCsvBuffer(#[from] csv::IntoInnerError<Writer<BufWriter<Vec<u8>>>>),
    #[error("Conversion error")]
    ConversionError(#[from] std::string::FromUtf8Error),
}

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
pub trait ToCsv {
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError>;
}

/// Marshals the state into HTML
pub trait ToHtml {
    fn to_html(&self) -> Result<String, ParsingError>;
}

#[derive(Clone, Debug, Eq, Default, Serialize, Deserialize)]
pub struct Folder {
    #[serde(rename = "id")]
    pub id: u32,
    #[serde(rename = "name")]
    pub name: String,
}

impl Hash for Folder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash each field individually
        self.id.hash(state);
        self.name.hash(state);
    }
}

impl Folder {
    pub fn new(id: u32, name: String) -> Self {
        Folder { id, name }
    }
}

impl Ord for Folder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Folder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl PartialEq for Folder {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl From<FolderCreateResponse> for Folder {
    fn from(response: FolderCreateResponse) -> Self {
        Folder::new(response.folder.id, response.folder.name)
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
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

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

#[derive(Clone, Debug, PartialEq, Default, Deserialize)]
pub struct ListOfFolders {
    #[serde(rename = "folders")]
    folders: Vec<Folder>,
}

// Implementing IntoIterator for ListOfFolders to iterate over Folder references
impl IntoIterator for ListOfFolders {
    type Item = Folder;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.folders.into_iter()
    }
}

// Implementing Iterator for a borrowed ListOfFolders (yielding references)
impl<'a> IntoIterator for &'a ListOfFolders {
    type Item = &'a Folder;
    type IntoIter = std::slice::Iter<'a, Folder>;

    fn into_iter(self) -> Self::IntoIter {
        self.folders.iter()
    }
}

impl ListOfFolders {
    pub fn get_folder_by_id(&self, id: &u32) -> Option<&Folder> {
        self.folders
            .iter()
            .find(|&folder| folder.id == id.to_owned())
    }

    pub fn get_folder_by_name(&self, name: &str) -> Option<&Folder> {
        self.folders.iter().find(|&folder| folder.name == name)
    }
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
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let folders = self.folders.clone();

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

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

// Implementing FromIterator for references to Folder
impl<'a> FromIterator<&'a Folder> for ListOfFolders {
    fn from_iter<I: IntoIterator<Item = &'a Folder>>(iter: I) -> Self {
        let folders = iter.into_iter().cloned().collect();
        ListOfFolders { folders }
    }
}

impl From<Vec<Folder>> for ListOfFolders {
    fn from(folders: Vec<Folder>) -> Self {
        let folders = folders.into_iter().map(|f| Folder::from(f)).collect();
        ListOfFolders { folders }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Model {
    #[serde(rename = "id")]
    pub uuid: Uuid,
    #[serde(rename = "isAssembly")]
    #[serde(default)]
    pub is_assembly: bool,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "folderId")]
    pub folder_id: u32,
    #[serde(rename = "folderName")]
    pub folder_name: Option<String>,
    #[serde(rename = "ownerId")]
    #[serde(default)]
    pub owner_id: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "fileType")]
    pub file_type: String,
    #[serde(rename = "thumbnail", skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(rename = "units")]
    #[serde(default)]
    pub units: String,
    #[serde(rename = "state")]
    #[serde(default)]
    pub state: String,
    #[serde(rename = "attachmentUrl", skip_serializing_if = "Option::is_none")]
    pub attachment_url: Option<String>,
    #[serde(rename = "shortId", skip_serializing_if = "Option::is_none")]
    pub short_id: Option<u64>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Vec<ModelMetadataItem>>,
}

impl Model {
    pub fn get_metadata_as_properties(&self) -> Option<HashMap<String, String>> {
        match &self.metadata {
            Some(metadata) => {
                let mut properties: HashMap<String, String> = HashMap::new();
                for property in metadata.iter() {
                    properties.insert(property.name.to_owned(), property.value.to_owned());
                }
                Some(properties)
            }
            None => None,
        }
    }
}

use serde::de::Deserializer;
fn deserialize_with_nullable_name<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or("invalid".to_string()))
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "id")]
    pub id: u64,
    #[serde(rename = "name", deserialize_with = "deserialize_with_nullable_name")]
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
    #[serde(rename = "metadataKeys")]
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
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

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
        Ok(String::from_utf8(bytes)?)
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelMetadataItemShort {
    #[serde(rename = "modelId")]
    pub model_uuid: Uuid,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "value")]
    pub value: String,
}

impl ModelMetadataItemShort {
    pub fn to_item(&self, key_id: u64) -> ModelExtendedMetadataItem {
        ModelExtendedMetadataItem {
            model_uuid: self.model_uuid,
            key_id,
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelMetadataItem {
    #[serde(rename = "metadataKeyId")]
    pub key_id: u64,
    #[serde(rename = "name", deserialize_with = "deserialize_with_nullable_name")]
    pub name: String,
    #[serde(rename = "value")]
    pub value: String,
}

impl ModelMetadataItem {
    pub fn new(key_id: u64, name: String, value: String) -> ModelMetadataItem {
        ModelMetadataItem {
            key_id,
            name,
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelExtendedMetadataItem {
    #[serde(rename = "metadataKeyId")]
    pub key_id: u64,
    #[serde(rename = "modelId")]
    pub model_uuid: Uuid,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "value")]
    pub value: String,
}

impl ModelExtendedMetadataItem {
    pub fn new(
        model_uuid: Uuid,
        key_id: u64,
        name: String,
        value: String,
    ) -> ModelExtendedMetadataItem {
        ModelExtendedMetadataItem {
            model_uuid,
            key_id,
            name,
            value,
        }
    }

    pub fn to_item(&self) -> ModelMetadataItem {
        ModelMetadataItem {
            key_id: self.key_id,
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelMetadata {
    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub properties: Vec<ModelMetadataItem>,
}

impl ModelMetadata {
    pub fn new(properties: Vec<ModelMetadataItem>) -> ModelMetadata {
        ModelMetadata { properties }
    }

    pub fn add(&mut self, new_item: &ModelMetadataItem) {
        self.properties.push(new_item.to_owned());
    }

    pub fn to_enhanced_csv(&self, uuid: &Uuid, pretty: bool) -> Result<String, ParsingError> {
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

        if pretty {
            let columns = vec!["MODEL_UUID", "NAME", "VALUE"];
            writer.write_record(&columns)?;
        }

        for property in &self.properties {
            let mut values: Vec<String> = Vec::new();
            values.push(uuid.to_string());
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
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

        if pretty {
            let columns = vec!["NAME", "VALUE"];
            writer.write_record(&columns)?;
        }

        for property in &self.properties {
            let mut values: Vec<String> = Vec::new();
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
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        log::trace!("Preparing CSV output for a model...");

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

        let standard_columns = vec![
            "ID",
            "NAME",
            "FOLDER_ID",
            "IS_ASSEMBLY",
            "FILE_TYPE",
            "UNITS",
            "STATE",
        ];
        let mut columns: HashSet<String> = HashSet::new();

        let meta = self.metadata.clone();
        match meta {
            Some(meta) => {
                for property in &meta {
                    let name = property.name.to_owned();
                    columns.insert(name);
                }
            }
            None => (),
        }

        let mut all_columns: Vec<&str> = standard_columns.clone();
        // using a HashSet first to guard against the backend returning duplicate property names
        let all_property_columns: HashSet<&str> = columns.iter().map(|n| n.as_str()).collect();
        let mut all_property_columns: Vec<&str> =
            all_property_columns.iter().map(|name| *name).collect();
        all_property_columns.sort();
        all_columns.append(&mut all_property_columns);

        trace!("Columns: {:?}", all_columns);

        if pretty {
            writer.write_record(&all_columns)?;
        }

        let mut values: Vec<String> = Vec::new();

        values.push(self.uuid.to_string());
        values.push(self.name.to_owned());
        values.push(self.folder_id.to_string());
        values.push(self.is_assembly.to_string());
        values.push(self.file_type.to_string());
        values.push(self.units.to_owned());
        values.push(self.state.to_owned());

        let mut properties: HashMap<String, String> = HashMap::new();
        let meta = self.metadata.clone();

        trace!("Preparing the name/value pairs for metadata properties...");
        match meta {
            Some(meta) => {
                for property in meta {
                    let name = property.name;
                    let value = property.value;

                    trace!("{}={}", &name, &value);

                    properties.insert(name, value);
                }
            }
            None => (),
        }

        let number_of_columns = all_columns.len();
        for i in 7..number_of_columns {
            let column_name = all_columns[i];
            let value = match properties.get(column_name) {
                Some(value) => value.to_owned(),
                None => String::from(""),
            };

            trace!("Set {}={}", &column_name, &value);
            values.push(value);
        }

        writer.write_record(&values)?;
        writer.flush()?;

        let bytes = writer.into_inner()?.into_inner()?;
        Ok(String::from_utf8(bytes)?)
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ListOfModels {
    #[serde(rename = "models")]
    pub models: Vec<Model>,
}

impl ToCsv for ListOfModels {
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let models = self.models.clone();
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

        let mut columns: HashSet<String> = HashSet::new();
        let standard_columns = vec![
            "ID",
            "NAME",
            "FOLDER_ID",
            "IS_ASSEMBLY",
            "FILE_TYPE",
            "UNITS",
            "STATE",
        ];

        // populate the column names with the names of all properties found in models
        for model in &models {
            let meta = model.metadata.clone();

            match meta {
                Some(meta) => {
                    for property in &meta {
                        let name = property.name.to_owned();
                        columns.insert(name);
                    }
                }
                None => (),
            }
        }

        let mut all_columns: Vec<&str> = standard_columns.clone();
        let mut all_property_columns: Vec<&str> = columns.iter().map(|n| n.as_str()).collect();
        all_property_columns.sort();
        all_columns.append(&mut all_property_columns);

        if pretty {
            writer.write_record(&all_columns)?;
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

            let meta = model.metadata.clone();
            let mut properties: HashMap<String, String> = HashMap::new();
            match meta {
                Some(meta) => {
                    for property in meta {
                        let name = property.name;
                        let value = property.value;
                        properties.insert(name, value);
                    }
                }
                None => (),
            }

            let number_of_columns = all_columns.len();
            for i in 7..number_of_columns {
                let column_name = all_columns[i];
                let value = match properties.get(column_name) {
                    Some(value) => value.to_owned(),
                    None => String::from(""),
                };
                values.push(value);
            }

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
        let models = physna_list_of_models_response
            .into_iter()
            .map(|m| Model::from(m))
            .collect();
        ListOfModels { models }
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
        ModelAssemblyTree { model, children }
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
        write!(
            f,
            "{}:[{}]",
            style.paint(self.model.name.clone()),
            style.paint(self.model.uuid.clone())
        )
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
        FlatBom {
            inner: elements.to_owned(),
        }
    }

    pub fn empty() -> Self {
        FlatBom {
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
        items.insert(
            assembly_tree.model.uuid.to_string(),
            assembly_tree.model.to_owned(),
        );

        // Recursivelly insert the models of all children models
        match assembly_tree.children {
            Some(children) => {
                for child in children {
                    let sub_bom = FlatBom::from(child);
                    items.extend(sub_bom.inner);
                }
            }
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
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let models = self.inner.clone();

        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

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
    #[serde(rename = "comparisonUrl", skip_serializing_if = "Option::is_none")]
    pub comparison_url: Option<String>,
}

impl PartialEq for ModelMatch {
    fn eq(&self, other: &Self) -> bool {
        self.model.name.eq(&other.model.name)
    }
}
impl Eq for ModelMatch {}

impl ModelMatch {
    pub fn new(model: Model, percentage: f64, comparison_url: Option<String>) -> ModelMatch {
        ModelMatch {
            model,
            percentage,
            comparison_url,
        }
    }
}

#[derive(Debug)]
pub struct ListOfModelMatches {
    pub inner: Box<Vec<ModelMatch>>,
}

impl ListOfModelMatches {
    pub fn new(matches: Box<Vec<ModelMatch>>) -> ListOfModelMatches {
        ListOfModelMatches { inner: matches }
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
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let matches = *self.inner.clone();
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

        let mut columns: HashSet<String> = HashSet::new();
        let standard_columns = vec![
            "MATCH_PERCENTAGE",
            "ID",
            "NAME",
            "FOLDER_ID",
            "IS_ASSEMBLY",
            "FILE_TYPE",
            "UNITS",
            "STATE",
        ];

        // populate the column names with the names of all properties found in the result
        for model_match in &matches {
            let model = model_match.model.clone();
            let meta = model.metadata;

            match meta {
                Some(meta) => {
                    for property in &meta {
                        let name = property.name.to_owned();
                        columns.insert(name);
                    }
                }
                None => (),
            }
        }

        let mut all_columns: Vec<&str> = standard_columns.clone();
        let mut all_property_columns: Vec<&str> = columns.iter().map(|n| n.as_str()).collect();
        all_property_columns.sort();
        all_columns.append(&mut all_property_columns);

        if pretty {
            writer.write_record(&all_columns)?;
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

            let meta = model.metadata.clone();
            let mut properties: HashMap<String, String> = HashMap::new();
            match meta {
                Some(meta) => {
                    for property in meta {
                        let name = property.name;
                        let value = property.value;
                        properties.insert(name, value);
                    }
                }
                None => (),
            }

            for column_name in &columns {
                let value = match properties.get(column_name) {
                    Some(value) => value.to_owned(),
                    None => String::from(""),
                };
                values.push(value);
            }

            writer.write_record(&values)?;
        }

        writer.flush()?;

        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VisuallyMatchedModel {
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileType")]
    pub file_type: String,
    #[serde(rename = "folderId")]
    pub folder_id: u32,
    #[serde(rename = "id")]
    pub uuid: Uuid,
    #[serde(rename = "isAssembly")]
    pub is_assembly: bool,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "units")]
    pub units: String,
    #[serde(rename = "state")]
    pub state: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListOfVisualModelMatches {
    pub models: Box<Vec<VisuallyMatchedModel>>,
}

impl ListOfVisualModelMatches {
    pub fn new(models: Box<Vec<VisuallyMatchedModel>>) -> Self {
        Self { models }
    }
}

impl ToJson for ListOfVisualModelMatches {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(&self.models)
        } else {
            serde_json::to_string(&self.models)
        }
    }
}

impl ToCsv for ListOfVisualModelMatches {
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let matches = *self.models.clone();
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

        let standard_columns = vec![
            "ID",
            "NAME",
            "FOLDER_ID",
            "IS_ASSEMBLY",
            "FILE_TYPE",
            "UNITS",
            "STATE",
        ];

        if pretty {
            writer.write_record(&standard_columns)?;
        }

        for m in matches {
            let model = m.clone();
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VisualMatchItem {
    #[serde(rename = "matchedModel")]
    pub model: VisuallyMatchedModel,
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
    #[serde(rename = "fodler_name")]
    pub folder_name: String,
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
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

        let mut columns: HashSet<String> = HashSet::new();
        let standard_columns = vec![
            "MODEL_NAME",
            "MATCHING_MODEL_NAME",
            "MATCH",
            "SOURCE_UUID",
            "MATCHING_UUID",
            "SOURCE_FOLDER_NAME",
            "MATCHING_FOLDER_NAME",
            "COMPARISON_URL",
        ];

        // populate the column names with the names of all properties found in the result
        for (_uuid, item) in &self.inner {
            for model_match in &item.matches {
                let model = model_match.model.clone();
                let meta = model.metadata;

                match meta {
                    Some(meta) => {
                        for property in &meta {
                            let name = property.name.to_owned();
                            columns.insert(name);
                        }
                    }
                    None => (),
                }
            }
        }

        let mut all_columns: Vec<&str> = standard_columns.clone();
        let mut all_property_columns: Vec<&str> = columns.iter().map(|n| n.as_str()).collect();
        all_columns.append(&mut all_property_columns);

        if pretty {
            writer.write_record(&all_columns)?;
        }

        for (_uuid, item) in &self.inner {
            let model_name = item.name.to_owned();
            let source_uuid = item.uuid.to_string();
            let source_folder_name = item.folder_name.to_owned();

            for m in &item.matches {
                let mut values: Vec<String> = Vec::new();

                values.push(model_name.to_owned());
                values.push(m.model.name.to_owned());
                values.push(m.percentage.to_string());
                values.push(source_uuid.to_owned());
                values.push(m.model.uuid.to_string());
                values.push(source_folder_name.to_owned());
                values.push(m.model.folder_name.to_owned().unwrap_or_default());

                match &m.comparison_url {
                    Some(url) => values.push(url.to_owned()),
                    None => values.push("".to_string()),
                }

                let meta = m.model.metadata.clone();
                let mut properties: HashMap<String, String> = HashMap::new();
                match meta {
                    Some(meta) => {
                        for property in meta {
                            let name = property.name;
                            let value = property.value;
                            properties.insert(name, value);
                        }
                    }
                    None => (),
                }

                for column_name in &columns {
                    let value = match properties.get(column_name) {
                        Some(value) => value.to_owned(),
                        None => String::from(""),
                    };
                    values.push(value);
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

impl ToHtml for SimpleDuplicatesMatchReport {
    fn to_html(&self) -> Result<String, ParsingError> {
        Ok(String::default())
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
    pub fn new(
        folder_id: u32,
        folder_name: String,
        file_type: String,
        state: String,
        count: u64,
    ) -> Self {
        ModelStatusRecord {
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
        EnvironmentStatusReport { stats: Vec::new() }
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
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

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
        Folder::new(folder.id, folder.name)
    }
}

impl From<client::FolderListResponse> for ListOfFolders {
    fn from(response: client::FolderListResponse) -> Self {
        let folders = response
            .folders
            .into_iter()
            .map(|f| Folder::from(f))
            .collect();
        ListOfFolders { folders }
    }
}

impl From<client::SingleModelResponse> for Model {
    fn from(response: client::SingleModelResponse) -> Self {
        Model {
            uuid: response.model.uuid,
            is_assembly: response.model.is_assembly,
            name: response.model.name,
            folder_id: response.model.folder_id,
            folder_name: None,
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
        ModelMatch::new(model, percentage, None)
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
pub struct FolderCreateResponse {
    #[serde(rename = "folder")]
    pub folder: Folder,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ModelCreateMetadataResponse {
    #[serde(rename = "metadata")]
    pub metadata: ModelMetadataItem,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct GeoLabel {
    #[serde(rename = "id")]
    pub id: u32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "geoClassifierId")]
    pub geo_classifier_id: u32,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ListOfGeoLabels {
    #[serde(rename = "geoLabels")]
    pub labels: Vec<GeoLabel>,
}

impl ListOfGeoLabels {
    pub fn new(labels: Vec<GeoLabel>) -> Self {
        ListOfGeoLabels { labels }
    }
}

impl From<Vec<GeoLabel>> for ListOfGeoLabels {
    fn from(labels: Vec<GeoLabel>) -> Self {
        ListOfGeoLabels::new(labels)
    }
}

impl ToJson for ListOfGeoLabels {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }
}

impl ToCsv for ListOfGeoLabels {
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

        if pretty {
            let columns = vec!["ID", "NAME", "CLASSIFIER_ID"];
            writer.write_record(&columns)?;
        }

        for label in &self.labels {
            let id = label.id.to_string();
            let name = label.name.to_owned();
            let classifier_id = label.geo_classifier_id.to_string();

            let mut values: Vec<String> = Vec::new();
            values.push(id);
            values.push(name);
            values.push(classifier_id);

            writer.write_record(&values)?;
        }
        writer.flush()?;

        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GeoMatch {
    #[serde(rename = "matchedModel")]
    pub model: Model,
    #[serde(rename = "confidence")]
    pub confidence: f64,
    #[serde(rename = "geoLabelId")]
    pub label_id: u32,
}

impl PartialEq for GeoMatch {
    fn eq(&self, other: &Self) -> bool {
        self.model.name.eq(&other.model.name)
    }
}
impl Eq for GeoMatch {}

impl GeoMatch {
    pub fn new(model: Model, confidence: f64, label_id: u32) -> GeoMatch {
        GeoMatch {
            model,
            confidence,
            label_id,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ImageMatch {
    #[serde(rename = "matchedModel")]
    pub model: Model,
}

impl PartialEq for ImageMatch {
    fn eq(&self, other: &Self) -> bool {
        self.model.name.eq(&other.model.name)
    }
}
impl Eq for ImageMatch {}

impl ImageMatch {
    pub fn new(model: Model) -> Self {
        Self { model }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ListOfGeoClassifierPredictions {
    #[serde(rename = "matches")]
    pub matches: Vec<GeoMatch>,
}

impl ListOfGeoClassifierPredictions {
    pub fn new(matches: Vec<GeoMatch>) -> Self {
        ListOfGeoClassifierPredictions { matches }
    }
}

impl From<Vec<GeoMatch>> for ListOfGeoClassifierPredictions {
    fn from(matches: Vec<GeoMatch>) -> Self {
        ListOfGeoClassifierPredictions::new(matches)
    }
}

impl ToJson for ListOfGeoClassifierPredictions {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }
}

impl ToCsv for ListOfGeoClassifierPredictions {
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

        if pretty {
            let columns = vec!["ID", "NAME", "CONFIDENCE", "IS_ASSEMBLY", "FOLDER"];
            writer.write_record(&columns)?;
        }

        for m in &self.matches {
            let id = m.model.uuid.to_string();
            let name = m.model.name.to_owned();
            let confidence = m.confidence.to_string();
            let is_assembly = m.model.is_assembly.to_string();
            let folder = m.model.folder_id.to_string();

            let mut values: Vec<String> = Vec::new();
            values.push(id);
            values.push(name);
            values.push(confidence);
            values.push(is_assembly);
            values.push(folder);

            writer.write_record(&values)?;
        }
        writer.flush()?;

        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct MatchedMetadataItem {
    #[serde(rename = "MODEL_UUID")]
    pub uuid: Uuid,
    #[serde(rename = "NAME")]
    pub name: String,
    #[serde(rename = "VALUE")]
    pub value: String,
    #[serde(rename = "MATCH_PERCENT")]
    pub score: f64,
}

impl MatchedMetadataItem {
    pub fn new(uuid: Uuid, name: String, value: String, score: f64) -> Self {
        Self {
            uuid,
            name,
            value,
            score,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ListOfMatchedMetadataItems {
    pub items: Vec<MatchedMetadataItem>,
}

impl ListOfMatchedMetadataItems {
    pub fn new(items: Vec<MatchedMetadataItem>) -> Self {
        Self { items }
    }
}

impl ToJson for ListOfMatchedMetadataItems {
    fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }
}

impl ToCsv for ListOfMatchedMetadataItems {
    fn to_csv(&self, pretty: bool) -> Result<String, ParsingError> {
        let buf = BufWriter::new(Vec::new());
        let mut writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(buf);

        if pretty {
            let columns = vec!["MODEL_UUID", "NAME", "VALUE", "MATCH_SCORE"];
            writer.write_record(&columns)?;
        }

        for item in &self.items {
            let uuid = item.uuid.to_string();
            let name = item.name.to_owned();
            let value = item.value.to_owned();
            let score = item.score.to_string();

            let mut values: Vec<String> = Vec::new();
            values.push(uuid);
            values.push(name);
            values.push(value);
            values.push(score);

            writer.write_record(&values)?;
        }
        writer.flush()?;

        let bytes = writer.into_inner()?.into_inner()?;
        let result = String::from_utf8(bytes)?;
        Ok(result)
    }
}
