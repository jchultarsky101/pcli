use crate::client::{ApiClient, AssemblyTree, ClientError};
use crate::format::{format_list_of_matched_properties, Format};
use crate::model::{
    EnvironmentStatusReport, FlatBom, Folder, ListOfFolders, ListOfMatchedMetadataItems,
    ListOfModelMatches, ListOfModels, ListOfVisualModelMatches, MatchedMetadataItem, Model,
    ModelAssemblyTree, ModelMatch, ModelMatchReport, ModelMatchReportItem, ModelMetadata,
    ModelMetadataItem, ModelMetadataItemShort, ModelStatusRecord, PartNodeDictionaryItem, Property,
    PropertyCollection, SimpleDuplicatesMatchReport, VisuallyMatchedModel,
};
use log::debug;
use log::{error, trace, warn};
use petgraph::matrix_graph::MatrixGraph;
use petgraph::matrix_graph::NodeIndex;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::rc::Rc;
use tempfile::tempfile;
use thiserror::Error;
use unicase::UniCase;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    ClientError(#[from] ClientError),
    #[error("Folder not found '{0}'")]
    FolderNotFound(String),
    #[error("I/O error")]
    InputOutputError(#[from] std::io::Error),
    #[error("CSV error")]
    CsvError(#[from] csv::Error),
    #[error("Failed to read data: {0}")]
    FailedToRead(String),
    #[error("Data format error: {0}")]
    FormatError(#[from] crate::format::FormatError),
}

pub struct Api {
    model_cache: HashMap<Uuid, Model>,
    client: Box<ApiClient>,
}

impl Api {
    pub fn new(base_url: String, tenant: String, access_token: String) -> Api {
        Api {
            model_cache: HashMap::new(),
            client: Box::new(ApiClient::connect(
                &base_url.to_owned(),
                &tenant.to_owned(),
                &access_token.to_owned(),
            )),
        }
    }

    pub fn tenant(&self) -> String {
        self.client.tenant.to_owned()
    }

    pub fn get_list_of_folders(
        &self,
        desired_folders: Option<HashSet<String>>,
    ) -> Result<ListOfFolders, ApiError> {
        log::trace!("Listing folders...");
        let list = self.client.get_list_of_folders(desired_folders)?;
        Ok(ListOfFolders::from(list))
    }

    pub fn create_folder(&self, name: &String) -> Result<Folder, ApiError> {
        log::trace!("Creating folder {}...", name);
        let folder = self.client.create_folder(name)?;
        Ok(Folder::from(folder))
    }

    pub fn delete_folder(&self, folders: HashSet<String>) -> Result<(), ApiError> {
        let folder_names = folders
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join(",");

        log::trace!("Deleting folder(s): {}...", folder_names.to_owned());
        let folders = self.get_list_of_folders(Some(folders))?;
        let folder_ids: HashSet<u32> = folders.into_iter().map(|f| f.id).collect();

        if folder_ids.len() > 0 {
            self.client.delete_folder(&folder_ids)?;
            Ok(())
        } else {
            Err(ApiError::FolderNotFound(folder_names))
        }
    }

    pub fn get_model_metadata(&self, uuid: &Uuid) -> Result<Option<ModelMetadata>, ApiError> {
        log::trace!("Reading model metadata for {}...", uuid.to_string());
        Ok(self.client.get_model_metadata(uuid)?)
    }

    pub fn delete_model_metadata_property(&self, uuid: &Uuid, id: &u64) -> Result<(), ApiError> {
        log::trace!("Deleting model metadata property...");
        self.client.delete_model_property(uuid, id)?;
        Ok(())
    }

    pub fn get_model(
        &mut self,
        uuid: &Uuid,
        use_cache: bool,
        meta: bool,
    ) -> Result<Model, ApiError> {
        if use_cache {
            let model_from_cache = self.model_cache.get(uuid);
            if let Some(model) = model_from_cache {
                trace!("Model cache hit for {}", uuid.to_string());
                return Ok(model.clone());
            }
        }
        let model = self.client.get_model(uuid)?;
        let mut model = Model::from(model);

        if meta {
            let metadata = self.get_model_metadata(uuid);
            match metadata {
                Ok(metadata) => match metadata {
                    Some(metadata) => {
                        model.metadata = Some(metadata.properties.to_owned());
                    }
                    None => model.metadata = None,
                },
                Err(_) => (),
            }
        }

        self.model_cache
            .insert(model.uuid.to_owned(), model.to_owned());
        Ok(model)
    }

    pub fn reprocess_model(&self, uuid: &Uuid) -> Result<(), ApiError> {
        trace!("Reprocessing {}...", uuid.to_string());
        self.client.reprocess_model(uuid)?;
        Ok(())
    }

    pub fn delete_model(&self, uuid: &Uuid) -> Result<(), ApiError> {
        self.client.delete_model(uuid)?;
        Ok(())
    }

    pub fn get_model_assembly_tree(&mut self, uuid: &Uuid) -> Result<ModelAssemblyTree, ApiError> {
        trace!("Reading assembly tree data for {}...", uuid.to_string());
        let tree = self.client.get_assembly_tree_for_model(uuid)?;
        Ok(self.enhance_assembly_tree_with_model(uuid, &tree)?)
    }

    fn enhance_assembly_tree_with_model(
        &mut self,
        uuid: &Uuid,
        tree: &AssemblyTree,
    ) -> Result<ModelAssemblyTree, ApiError> {
        trace!("Enhancing model data for {}...", uuid.to_string());

        let model = self.get_model(uuid, true, false)?;
        let assembly_tree = match &tree.children {
            Some(tree_children) => {
                let mut assembly_children: Vec<ModelAssemblyTree> = Vec::new();
                for child in tree_children {
                    let child_uuid = Uuid::parse_str(&child.uuid.as_str()).unwrap();
                    assembly_children
                        .push(self.enhance_assembly_tree_with_model(&child_uuid, child)?);
                }
                ModelAssemblyTree::new(model, Some(assembly_children))
            }
            None => ModelAssemblyTree::new(model, None),
        };

        Ok(assembly_tree)
    }

    /// Returns a list of models that match the search and filter criteria
    ///
    /// Parameters:
    ///
    /// folders - list of folder names to be used as a filter. If empty, all folders are included
    /// search - search text
    /// meta - if true, the metadata is included in the response
    pub fn list_all_models(
        &self,
        folders: Option<HashSet<String>>,
        search: Option<&String>,
    ) -> Result<ListOfModels, ApiError> {
        trace!("Listing all models...");

        let folder_ids: Option<HashSet<u32>> = match folders {
            Some(folders) => {
                if folders.len() > 0 {
                    let existing_folders = self.get_list_of_folders(None)?;

                    let folders = self.validate_folders(&existing_folders, &folders)?;

                    let folder_ids: HashSet<u32> = folders.into_iter().map(|f| f.id).collect();
                    Some(folder_ids)
                } else {
                    None
                }
            }
            None => None,
        };

        let mut list_of_models: Vec<Model> = Vec::new();

        let mut has_more = true;
        let mut page: u32 = 1;
        let per_page: u32 = 50;
        while has_more {
            let result = self.client.get_list_of_models_page(
                folder_ids.clone(),
                search.to_owned(),
                per_page,
                page,
            )?;
            if result.page_data.total > 0 {
                let models = result.models;
                if !models.is_empty() {
                    for m in models {
                        list_of_models.push(Model::from(m.clone()));
                    }
                }
            }
            has_more = result.page_data.current_page < result.page_data.last_page;
            page = result.page_data.current_page + 1;
        }

        let result = ListOfModels::from(list_of_models);

        //trace!("List of Models: {:?}", result);
        Ok(result)
    }

    pub fn match_model(
        &self,
        uuid: &Uuid,
        threshold: f64,
        with_meta: bool,
        with_reference_meta: bool,
        classification: Option<&String>,
        tag: Option<&String>,
    ) -> Result<ListOfModelMatches, ApiError> {
        let reference_metadata: Option<ModelMetadata> = if with_reference_meta {
            self.client.get_model_metadata(uuid)?
        } else {
            None
        };

        trace!("Matching model {}...", uuid);
        let mut list_of_matches: Vec<ModelMatch> = Vec::new();

        let mut has_more = true;
        let mut page: u32 = 1;
        let per_page: u32 = 50;
        while has_more {
            let result = self
                .client
                .get_model_match_page(uuid, threshold, per_page, page)?;
            if result.page_data.total > 0 {
                let matches = result.matches;
                if !matches.is_empty() {
                    trace!("Reading the list of properties for model {}...", uuid);
                    let properties = match classification {
                        Some(_) => Some(self.client.get_list_of_properties()?),
                        None => None,
                    };

                    for m in matches {
                        let mut model_match = ModelMatch::from(m);
                        let model = model_match.model.clone();
                        let metadata: Option<ModelMetadata> = if with_meta {
                            let matching_metadata = self.get_model_metadata(&model.uuid)?;

                            if matching_metadata.is_some() || reference_metadata.is_some() {
                                let mut combined_meta = ModelMetadata::default();

                                matching_metadata
                                    .unwrap()
                                    .properties
                                    .iter()
                                    .for_each(|item| combined_meta.add(item));

                                reference_metadata
                                    .as_ref()
                                    .unwrap()
                                    .properties
                                    .iter()
                                    .for_each(|item| {
                                        combined_meta.add(&ModelMetadataItem::new(
                                            item.key_id,
                                            format!("reference.{}", item.name),
                                            item.value.to_owned(),
                                        ))
                                    });

                                Some(combined_meta)
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        //log::trace!("Model metadata: {:?}", &metadata);

                        match classification {
                            Some(classification) => {
                                let property =
                                    properties.as_ref().unwrap().properties.iter().find(|p| {
                                        p.name.eq_ignore_ascii_case(classification.as_str())
                                    });
                                let property = match property {
                                    Some(property) => property.clone(),
                                    None => {
                                        self.client.post_property(&String::from(classification))?
                                    }
                                };

                                let item = ModelMetadataItem::new(
                                    property.id.clone(),
                                    String::from(classification),
                                    String::from(tag.unwrap()),
                                );

                                trace!(
                                    "Setting property {} to value of {} for model {}",
                                    classification,
                                    tag.unwrap(),
                                    model.uuid
                                );
                                self.client.put_model_property(&uuid, &property.id, &item)?;
                            }
                            None => (),
                        }

                        match metadata {
                            Some(metadata) => {
                                model_match.model.metadata = Some(metadata.properties.to_owned())
                            }
                            None => model_match.model.metadata = None,
                        }
                        list_of_matches.push(model_match);
                    }
                }
            }
            has_more = result.page_data.current_page < result.page_data.last_page;
            page = result.page_data.current_page + 1;
        }

        Ok(ListOfModelMatches::new(Box::new(list_of_matches)))
    }

    pub fn match_model_visual(&self, uuid: &Uuid) -> Result<ListOfVisualModelMatches, ApiError> {
        trace!("Matching model visual {}...", uuid);
        let mut list_of_matches: Vec<VisuallyMatchedModel> = Vec::new();

        let mut has_more = true;
        let mut page: u32 = 1;
        let per_page: u32 = 100;
        while has_more {
            let result = self
                .client
                .get_model_visual_match_page(uuid, per_page, page)?;
            if result.page_data.total > 0 {
                let matches = result.matches;
                if !matches.is_empty() {
                    for m in matches {
                        list_of_matches.push(m.model.clone());
                    }
                }
            }
            has_more = result.page_data.current_page < result.page_data.last_page;
            page = result.page_data.current_page + 1;
        }

        // remove the reference UUID from the list of results if present
        if let Some(pos) = list_of_matches
            .iter()
            .cloned()
            .position(|x| x.uuid == uuid.to_owned())
        {
            list_of_matches.remove(pos);
        }
        list_of_matches.truncate(10);

        trace!("Result: {:?}", &list_of_matches);

        Ok(ListOfVisualModelMatches::new(Box::new(list_of_matches)))
    }

    pub fn match_scan_model(
        &self,
        uuid: &Uuid,
        threshold: f64,
        with_meta: bool,
        classification: Option<&String>,
        tag: Option<&String>,
    ) -> Result<ListOfModelMatches, ApiError> {
        trace!("Scan match model {}...", uuid);
        let mut list_of_matches: Vec<ModelMatch> = Vec::new();

        let mut has_more = true;
        let mut page: u32 = 1;
        let per_page: u32 = 50;
        while has_more {
            let result = self
                .client
                .get_model_scan_match_page(uuid, threshold, per_page, page)?;
            if result.page_data.total > 0 {
                let matches = result.matches;
                if !matches.is_empty() {
                    trace!("Reading the list of properties for model {}...", uuid);
                    let properties = match classification {
                        Some(_) => Some(self.client.get_list_of_properties()?),
                        None => None,
                    };

                    for m in matches {
                        let mut model_match = ModelMatch::from(m);
                        let model = model_match.model.clone();
                        let metadata: Option<ModelMetadata>;
                        if with_meta {
                            metadata = self.get_model_metadata(&model.uuid)?;
                        } else {
                            metadata = None;
                        }

                        match classification {
                            Some(classification) => {
                                let property =
                                    properties.as_ref().unwrap().properties.iter().find(|p| {
                                        p.name.eq_ignore_ascii_case(classification.as_str())
                                    });
                                let property = match property {
                                    Some(property) => property.clone(),
                                    None => {
                                        self.client.post_property(&String::from(classification))?
                                    }
                                };

                                let item = ModelMetadataItem::new(
                                    property.id.clone(),
                                    String::from(classification),
                                    String::from(tag.unwrap()),
                                );

                                trace!(
                                    "Setting property {} to value of {} for model {}",
                                    classification,
                                    tag.unwrap(),
                                    model.uuid
                                );
                                self.client.put_model_property(&uuid, &property.id, &item)?;
                            }
                            None => (),
                        }

                        match metadata {
                            Some(metadata) => {
                                model_match.model.metadata = Some(metadata.properties.to_owned())
                            }
                            None => model_match.model.metadata = None,
                        }
                        list_of_matches.push(model_match);
                    }
                }
            }
            has_more = result.page_data.current_page < result.page_data.last_page;
            page = result.page_data.current_page + 1;
        }

        Ok(ListOfModelMatches::new(Box::new(list_of_matches)))
    }

    pub fn set_property(&self, name: &String) -> Result<Property, ApiError> {
        Ok(self.client.post_property(name)?)
    }

    pub fn set_model_property(
        &self,
        model_uuid: &Uuid,
        id: &u64,
        item: &ModelMetadataItem,
    ) -> Result<ModelMetadataItem, ApiError> {
        Ok(self.client.put_model_property(model_uuid, id, item)?)
    }

    fn generate_graph_from_assembly_tree(
        &self,
        parent_node_index: Option<NodeIndex>,
        graph: &mut MatrixGraph<String, f64>,
        dictionary: &mut HashMap<Uuid, PartNodeDictionaryItem>,
        trees: &Vec<ModelAssemblyTree>,
    ) {
        for tree in trees {
            //let parent_uuid = Uuid::parse_str(tree.model.uuid.as_str()).unwrap();
            let node_name = tree.model.name.to_owned();
            let node_index = graph.add_node(node_name);
            let node_dictionary_item = PartNodeDictionaryItem {
                name: tree.model.name.to_owned(),
                uuid: tree.model.uuid.to_owned(),
                node: node_index.index(),
            };
            dictionary.insert(node_dictionary_item.uuid, node_dictionary_item);

            match parent_node_index {
                Some(parent_node_index) => {
                    graph.add_edge(parent_node_index, node_index, 1.0);
                }
                None => (),
            }

            let children = tree.children.to_owned();
            if tree.children.is_some() {
                self.generate_graph_from_assembly_tree(
                    Some(node_index),
                    graph,
                    dictionary,
                    &children.unwrap(),
                );
            }
        }
    }

    /// Validates list of folder names against the list of actual folders present in the tenant
    ///
    /// Parameters:
    ///
    /// existing_folders - list of existing folders
    /// desired_folder_names - list of folder names we want to check. If empty list, include all available
    pub fn validate_folders(
        &self,
        existing_folders: &ListOfFolders,
        desired_folder_names: &HashSet<String>,
    ) -> Result<ListOfFolders, ApiError> {
        let existing_folder_names: HashSet<String> = existing_folders
            .into_iter()
            .map(|f| f.name.to_owned())
            .collect();

        // generate an error if any of the desired names are not existing folder names
        let diff: HashSet<String> = desired_folder_names
            .difference(&existing_folder_names)
            .cloned()
            .collect();

        if diff.len() > 0 {
            return Err(ApiError::FolderNotFound(
                diff.into_iter().collect::<Vec<String>>().join(", "),
            ));
        }

        let validated_folders = if desired_folder_names.len() > 0 {
            // if there is a filter, include only the folders that match the names
            desired_folder_names
                .iter()
                .map(|n| existing_folders.get_folder_by_name(n.as_str()).unwrap())
                .collect()
        } else {
            // if there is no filter, include all folders
            existing_folders.clone()
        };

        Ok(validated_folders)
    }

    pub fn generate_simple_model_match_report(
        &mut self,
        uuids: Vec<Uuid>,
        threshold: &f64,
        folders: Option<HashSet<String>>,
        exclusive: bool,
        with_meta: bool,
        metadata_filter: Option<HashMap<String, String>>,
    ) -> Result<SimpleDuplicatesMatchReport, ApiError> {
        trace!("Generating simple match report...");

        let mut simple_match_report = SimpleDuplicatesMatchReport::new();

        // Read the list of folders currently existing in the tenant
        let existing_folders = self.get_list_of_folders(None)?;

        // Validate the folders against the existing folders
        let folders = match folders {
            Some(folders) => self.validate_folders(&existing_folders, &folders)?,
            None => existing_folders.clone(),
        };

        for uuid in uuids {
            let mut model = match self.get_model(&uuid, true, with_meta) {
                Ok(model) => model,
                Err(e) => {
                    warn!("Failed to query for model {}: {}", uuid, e);
                    continue;
                }
            };

            if model.state != "finished" {
                warn!(
                    "Model {} has state {}. Skipping model match!",
                    uuid, model.state
                );
                continue;
            }

            debug!("Checking for metadata filter...");
            match &metadata_filter {
                Some(filter) => {
                    debug!("Applying metadata filter...");
                    match model.get_metadata_as_properties() {
                        Some(metadata) => {
                            let all_exist = filter.iter().all(|(k, v)| match metadata.get(k) {
                                Some(value) => value == v,
                                None => false,
                            });

                            if !all_exist {
                                debug!("Failed metadata filter condition(s)");
                                continue;
                            } else {
                                debug!("Filter matches the metadata")
                            }
                        }
                        None => {
                            debug!("There is no metadata to be compared to the filter");
                            continue;
                        }
                    }
                }
                None => {
                    trace!("No metadata filter specified");
                }
            }

            let folder = existing_folders.get_folder_by_id(&model.folder_id);
            model.folder_name = match folder {
                Some(folder) => Some(folder.name.to_owned()),
                None => None,
            };

            let matches =
                match self.match_model(&uuid, threshold.clone(), with_meta, false, None, None) {
                    Ok(matches) => matches,
                    Err(e) => {
                        warn!("Failed to match model {}: {}", uuid, e);
                        continue;
                    }
                };

            let mut simple_duplicate_matches: Vec<ModelMatch> = Vec::new();

            for m in matches.inner.iter() {
                let is_exclusive_valid =
                    !exclusive || folders.get_folder_by_id(&m.model.folder_id).is_some();
                let is_name_different = model.name != m.model.name;
                let is_type_different = model.is_assembly != m.model.is_assembly;
                let is_not_duplicate = !simple_duplicate_matches.contains(&m);

                if is_exclusive_valid
                    && (is_name_different || is_type_different)
                    && is_not_duplicate
                {
                    let mut m1 = m.clone();
                    m1.comparison_url = Some(format!(
                        "https://{}.physna.com/app/compare?modelAId={}&modelBId={}",
                        self.client.tenant, uuid, m1.model.uuid
                    ));
                    m1.model.folder_name =
                        match existing_folders.get_folder_by_id(&m1.model.folder_id) {
                            Some(folder) => Some(folder.name.to_owned()),
                            None => None,
                        };

                    simple_duplicate_matches.push(m1);
                }
            }

            let folder = folders.get_folder_by_id(&model.folder_id.clone());
            let folder_name = match folder {
                Some(folder) => folder.name.to_owned(),
                None => String::default(),
            };

            if !simple_duplicate_matches.is_empty() {
                let item = ModelMatchReportItem {
                    uuid: uuid.to_string(),
                    name: model.name.clone(),
                    folder_name,
                    matches: simple_duplicate_matches,
                };
                simple_match_report.inner.insert(uuid.to_string(), item);
            }
        }

        Ok(simple_match_report)
    }

    pub fn generate_model_match_report(
        &mut self,
        uuids: Vec<Uuid>,
        threshold: f64,
        with_meta: bool,
        meta_filter: Option<HashMap<String, String>>,
    ) -> Result<ModelMatchReport, ApiError> {
        let mut flat_bom = FlatBom::empty();
        let mut roots: HashMap<Uuid, ModelAssemblyTree> = HashMap::new();
        let mut dictionary: HashMap<Uuid, PartNodeDictionaryItem> = HashMap::new();

        // Create the Assembly Tree(s)
        for uuid in uuids {
            let assembly_tree = self.get_model_assembly_tree(&uuid);
            match assembly_tree {
                Ok(assembly_tree) => {
                    roots.insert(uuid, assembly_tree.clone());
                    flat_bom.extend(&FlatBom::from(assembly_tree));
                }
                Err(e) => warn!("Error while matching {}: {}", uuid.to_string(), e),
            }
        }

        let target_uuids: Vec<Uuid> = flat_bom
            .inner
            .to_owned()
            .keys()
            .map(|uuid| Uuid::parse_str(uuid.as_str()).unwrap())
            .collect();

        let simple_match_report = self.generate_simple_model_match_report(
            target_uuids,
            &threshold,
            None,
            false,
            with_meta,
            meta_filter,
        )?;

        // Create the DAG
        let mut graph: MatrixGraph<String, f64> = MatrixGraph::new();
        self.generate_graph_from_assembly_tree(
            None,
            &mut graph,
            &mut dictionary,
            &roots.values().cloned().collect(),
        );

        //let matrix = generate_matrix_from_match_report(&simple_match_report, &dictionary);

        Ok(ModelMatchReport {
            duplicates: simple_match_report,
            dictionary,
            graph,
            //matrix: matrix,
        })
    }

    pub fn tenant_stats(
        &mut self,
        folders: HashSet<String>,
        force_fix: bool,
        ignore_assemblies: bool,
    ) -> Result<EnvironmentStatusReport, ApiError> {
        let all_folders = self.get_list_of_folders(None)?;
        let all_folders: HashMap<u32, Folder> =
            all_folders.into_iter().map(|f| (f.id, f)).collect();

        let models = self.list_all_models(Some(folders), None)?;
        let models = models.models.to_owned();
        let mut result: HashMap<u64, ModelStatusRecord> = HashMap::new();

        for model in models {
            if force_fix
                && !model.state.eq_ignore_ascii_case("FINISHED")
                && !model.state.eq_ignore_ascii_case("NO 3D DATA")
            {
                if !model.is_assembly || !ignore_assemblies {
                    let _ = self.reprocess_model(&model.uuid);
                }
            }

            let folder_id = model.folder_id;
            let folder_name = all_folders.get(&folder_id).unwrap().name.to_owned();
            let folder_name2 = folder_name.to_owned();
            let stat = ModelStatusRecord::new(
                folder_id,
                folder_name,
                model.file_type.to_uppercase(),
                model.state.to_uppercase(),
                1,
            );
            let mut s = DefaultHasher::new();
            stat.hash(&mut s);
            let h = s.finish();
            let stat_as_found = result.get(&h);
            match stat_as_found {
                Some(s) => {
                    let s2 = ModelStatusRecord::new(
                        folder_id,
                        folder_name2,
                        model.file_type.to_uppercase(),
                        model.state.to_uppercase(),
                        s.count + 1,
                    );
                    result.insert(h, s2);
                }
                None => {
                    result.insert(h, stat);
                }
            }
        }

        let result: Vec<ModelStatusRecord> = result.into_iter().map(|(_, s)| s).collect();
        let mut stats: EnvironmentStatusReport = EnvironmentStatusReport::new();
        stats.stats = result;
        Ok(stats)
    }

    pub fn upload_model(&self, folder: &str, path: &PathBuf) -> Result<Option<Model>, ApiError> {
        Ok(self.client.upload_model(folder, path)?)
    }

    pub fn download_model(&self, uuid: &Uuid) -> Result<(), ApiError> {
        Ok(self.client.download_model(uuid)?)
    }

    pub fn list_all_properties(&self) -> Result<PropertyCollection, ApiError> {
        trace!("Listing all properties...");
        Ok(self.client.get_list_of_properties()?)
    }

    pub fn upload_model_metadata(&self, input_file: &File, clean: bool) -> Result<(), ApiError> {
        // Get all properties and cache them. The Physna API V2 does not allow me to get property by name
        let properties = self.list_all_properties()?;
        let all_props = Rc::new(properties.properties.clone());
        let mut reverse_lookup: HashMap<UniCase<String>, u64> = properties
            .properties
            .iter()
            .map(|p| (UniCase::new(p.name.to_owned()), p.id))
            .collect();

        let mut uuids: Vec<Uuid> = Vec::new();

        let mut rdr = csv::Reader::from_reader(input_file);
        for record in rdr.records() {
            let (id, property) = match record {
                Ok(record) => {
                    let m: ModelMetadataItemShort = record.deserialize(None)?;

                    if clean && !uuids.contains(&m.model_uuid) {
                        trace!(
                            "Deleting all properties for model {}...",
                            m.model_uuid.to_string()
                        );

                        for property in all_props.clone().iter() {
                            let _ = self
                                .client
                                .delete_model_property(&m.model_uuid, &property.id);
                        }
                        uuids.push(m.model_uuid.clone());
                    }

                    let case_insensitive_name: UniCase<String> = UniCase::new(m.name.to_owned());
                    match reverse_lookup.get(&case_insensitive_name) {
                        Some(id) => (*id, m.to_item(*id)),
                        None => {
                            let p = self.client.post_property(&m.name)?;
                            reverse_lookup.insert(case_insensitive_name.clone(), p.id);
                            (p.id, m.to_item(p.id))
                        }
                    }
                }
                Err(e) => return Err(ApiError::FailedToRead(e.to_string())),
            };

            if property.value.is_empty() {
                self.client
                    .delete_model_property(&property.model_uuid, &id)?;
            } else {
                trace!(
                    "Set property '{}'='{}' for model {}",
                    &property.name.to_owned(),
                    &property.value.to_owned(),
                    &property.model_uuid
                );
                self.client
                    .put_model_property(&property.model_uuid, &id, &property.to_item())?;
            }
        }

        Ok(())
    }

    pub fn search_by_multiple_images(
        &self,
        paths: Vec<&PathBuf>,
        max_results: u32,
        search: Option<&String>,
        filter: Option<&String>,
    ) -> Result<ListOfModels, ApiError> {
        let mut upload_ids: Vec<String> = Vec::new();
        for path in paths {
            let path = path.as_path();
            let image_upload = self.client.get_image_upload_specs(&path)?;
            let url = Url::parse(image_upload.upload_url.as_str()).unwrap();
            let size_requirements = image_upload.file_size_requirements;
            let mime = image_upload.headers.content_type;
            let content_range = image_upload.headers.content_length_range;
            let id = image_upload.id;
            upload_ids.push(id.to_owned());

            self.client
                .upload_image_file(url, size_requirements, &path, mime, content_range)?;
        }

        let matches =
            self.client
                .get_image_search_maches(upload_ids, search, filter, max_results, 100)?;

        Ok(matches)
    }

    pub fn search_by_image(
        &self,
        path: &PathBuf,
        max_results: u32,
        search: Option<&String>,
        filter: Option<&String>,
    ) -> Result<ListOfModels, ApiError> {
        let path = path.as_path();
        let image_upload = self.client.get_image_upload_specs(&path)?;
        let url = Url::parse(image_upload.upload_url.as_str()).unwrap();
        let size_requirements = image_upload.file_size_requirements;
        let mime = image_upload.headers.content_type;
        let content_range = image_upload.headers.content_length_range;
        let id = image_upload.id;

        self.client
            .upload_image_file(url, size_requirements, &path, mime, content_range)?;

        let matches =
            self.client
                .get_image_search_maches(vec![id], search, filter, max_results, 100)?;

        Ok(matches)
    }

    pub fn label_inference(
        &mut self,
        uuid: &Uuid,
        threshold: f64, // Changed from &f64 to f64 for simplicity
        keys: &Option<Vec<String>>,
        cascade: bool,
        apply: bool,
        folders: &Option<HashSet<String>>,
    ) -> Result<ListOfMatchedMetadataItems, ApiError> {
        let matches = self.match_model(uuid, threshold, true, false, None, None)?;

        let existing_folders = self.get_list_of_folders(folders.clone())?;

        // retrieve the list of valid folders. If no filter explicitly specified, all existing folders
        let existing_folders: HashMap<u32, String> = existing_folders
            .into_iter()
            .map(|f| (f.id, f.name))
            .collect();

        // Sort matches by score in descending order (largest percentage first)
        let mut matches = matches.inner;
        matches.sort_by(|a, b| {
            b.percentage
                .partial_cmp(&a.percentage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut props: HashMap<String, MatchedMetadataItem> = HashMap::new();

        // populate with the original values, so that they do not get overrided
        match self.get_model_metadata(uuid)? {
            Some(original_metadata) => {
                for p in original_metadata.properties.iter() {
                    let name = p.to_owned().name;
                    props.insert(
                        name.to_owned(),
                        MatchedMetadataItem::new(
                            uuid.to_owned(),
                            name.to_owned(),
                            p.value.to_owned(),
                            1.0,
                        ),
                    );
                }
            }
            None => (),
        };

        for m in matches.into_iter() {
            if cascade && m.model.is_assembly {
                let uuid = m.model.uuid.to_owned();
                let tree = self.get_model_assembly_tree(&uuid)?;

                match tree.children {
                    Some(children) => {
                        for child in children.into_iter() {
                            let uuid = child.model.uuid;
                            let partial_result = self
                                .label_inference(&uuid, threshold, keys, cascade, false, folders)?;
                            let _partial_props = partial_result.items;
                        }
                    }
                    None => (),
                }
            }

            let score = m.percentage;
            let folder_id = m.model.folder_id;

            if existing_folders.get(&folder_id).is_some() {
                // the model belongs to a folder that is in the whitelist

                if let Some(ref metadata) = m.model.metadata {
                    for p in metadata {
                        let name = &p.name;
                        if keys.as_ref().map_or(true, |k| k.contains(name)) {
                            if props.get(name).is_none() {
                                let property = MatchedMetadataItem::new(
                                    uuid.to_owned(),
                                    name.clone(),
                                    p.value.clone(),
                                    score,
                                );
                                props.insert(name.clone(), property);
                            }
                        }
                    }
                }
            }
        }

        let result = ListOfMatchedMetadataItems::new(props.into_values().collect());

        if apply {
            trace!("Applying infered metadata...");

            // add the infered properties automatically
            let mut file = tempfile()?;
            let output = format_list_of_matched_properties(&result, &Format::Csv, true, None)?;
            file.write_all(output.as_bytes())?;
            file.flush()?;
            file.seek(SeekFrom::Start(0))?;

            self.upload_model_metadata(&file, false)?;
        }

        Ok(result)
    }
}
