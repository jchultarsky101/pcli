use crate::client::{ApiClient, AssemblyTree, ClientError};
use crate::model::{
    EnvironmentStatusReport, FlatBom, Folder, ListOfFolders, ListOfModelMatches, ListOfModels,
    Model, ModelAssemblyTree, ModelMatch, ModelMatchReport, ModelMatchReportItem, ModelMetadata,
    ModelMetadataItem, ModelMetadataItemShort, ModelStatusRecord, PartNodeDictionaryItem, Property,
    PropertyCollection, SimpleDuplicatesMatchReport,
};
use anyhow::{anyhow, Result};
use itertools::Itertools;
use log::debug;
use log::{error, trace, warn};
use petgraph::matrix_graph::MatrixGraph;
use petgraph::matrix_graph::NodeIndex;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::rc::Rc;
use unicase::UniCase;
use url::Url;
use uuid::Uuid;

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

    pub fn get_list_of_folders(&self) -> Result<ListOfFolders> {
        let list = self.client.get_list_of_folders();
        match list {
            Ok(list) => Ok(ListOfFolders::from(list)),
            Err(e) => match e {
                ClientError::Parsing(message) => {
                    error!("{}", message);
                    return Err(anyhow!("{}", message));
                }
                _ => return Err(anyhow!(e)),
            },
        }
    }

    pub fn create_folder(&self, name: &String) -> Result<Folder> {
        let folder = self.client.create_folder(name);
        match folder {
            Ok(folder) => Ok(Folder::from(folder)),
            Err(e) => match e {
                ClientError::Parsing(message) => {
                    error!("{}", message);
                    return Err(anyhow!("{}", message));
                }
                _ => return Err(anyhow!(e)),
            },
        }
    }

    pub fn delete_folder(&self, folders: HashSet<String>) -> anyhow::Result<()> {
        self.client.delete_folder(&folders)?;
        Ok(())
    }

    pub fn get_model_metadata(&self, uuid: &Uuid) -> anyhow::Result<Option<ModelMetadata>> {
        log::trace!("Reading model metadata for {}...", uuid.to_string());
        Ok(self.client.get_model_metadata(uuid)?)
    }

    pub fn delete_model_metadata_property(&self, uuid: &Uuid, id: &u64) -> anyhow::Result<()> {
        self.client.delete_model_property(uuid, id)?;
        Ok(())
    }

    pub fn get_model(&mut self, uuid: &Uuid, use_cache: bool, meta: bool) -> anyhow::Result<Model> {
        if use_cache {
            let model_from_cache = self.model_cache.get(uuid);
            if let Some(model) = model_from_cache {
                trace!("Model cache hit for {}", uuid.to_string());
                return Ok(model.clone());
            }
        }
        let model = self.client.get_model(uuid);

        match model {
            Ok(response) => {
                let mut model = Model::from(response);

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
            Err(e) => Err(anyhow!(
                "Failed to read model {}, because of: {}",
                uuid.to_string(),
                e
            )),
        }
    }

    pub fn reprocess_model(&self, uuid: &Uuid) -> anyhow::Result<()> {
        trace!("Reprocessing {}...", uuid.to_string());
        self.client.reprocess_model(uuid)?;
        Ok(())
    }

    pub fn delete_model(&self, uuid: &Uuid) -> anyhow::Result<()> {
        self.client.delete_model(uuid)?;
        Ok(())
    }

    pub fn get_model_assembly_tree(&mut self, uuid: &Uuid) -> anyhow::Result<ModelAssemblyTree> {
        trace!("Reading assembly tree data for {}...", uuid.to_string());
        match self.client.get_assembly_tree_for_model(uuid) {
            Ok(tree) => Ok(self.enhance_assembly_tree_with_model(uuid, &tree)?),
            Err(e) => Err(anyhow!(
                "Failed to read assembly tree for model {}, because of: {}",
                uuid.to_string(),
                e
            )),
        }
    }

    fn enhance_assembly_tree_with_model(
        &mut self,
        uuid: &Uuid,
        tree: &AssemblyTree,
    ) -> anyhow::Result<ModelAssemblyTree> {
        trace!("Enhancing model data for {}...", uuid.to_string());

        let model = self.get_model(uuid, true, false)?;
        let assembly_tree = match &tree.children {
            Some(tree_children) => {
                let mut assembly_children: Vec<ModelAssemblyTree> = Vec::new();
                for child in tree_children {
                    let child_uuid = Uuid::parse_str(&child.uuid.as_str())?;
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
        folders: HashSet<String>,
        search: Option<&String>,
    ) -> Result<ListOfModels> {
        trace!("Listing all models for folders {:?}...", folders);

        let folder_ids: HashSet<u32> = if folders.len() > 0 {
            let existing_folders = self.get_list_of_folders()?;

            let folders = self.validate_folders(&existing_folders, &folders)?;

            let folder_ids: HashSet<u32> = folders.into_iter().map(|f| f.id).collect();
            folder_ids
        } else {
            HashSet::new()
        };

        let mut list_of_models: Vec<Model> = Vec::new();

        let mut has_more = true;
        let mut page: u32 = 1;
        let per_page: u32 = 50;
        while has_more {
            match self.client.get_list_of_models_page(
                folder_ids.clone(),
                search.to_owned(),
                per_page,
                page,
            ) {
                Ok(result) => {
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
                Err(e) => return Err(anyhow!("{}", e)),
            };
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
        classification: Option<&String>,
        tag: Option<&String>,
    ) -> anyhow::Result<ListOfModelMatches> {
        trace!("Matching model {}...", uuid);
        let mut list_of_matches: Vec<ModelMatch> = Vec::new();

        let mut has_more = true;
        let mut page: u32 = 1;
        let per_page: u32 = 50;
        while has_more {
            match self
                .client
                .get_model_match_page(uuid, threshold, per_page, page)
            {
                Ok(result) => {
                    if result.page_data.total > 0 {
                        let matches = result.matches;
                        if !matches.is_empty() {
                            debug!("Reading the list of properties for model {}...", uuid);
                            let properties = match classification {
                                Some(_) => Some(self.client.get_list_of_properties()?),
                                None => None,
                            };

                            for m in matches {
                                let mut model_match = ModelMatch::from(m);
                                let model = model_match.model.clone();
                                let metadata: Option<ModelMetadata> = if with_meta {
                                    self.get_model_metadata(&model.uuid)?
                                } else {
                                    None
                                };

                                log::trace!("Model metadata: {:?}", &metadata);

                                match classification {
                                    Some(classification) => {
                                        let property = properties
                                            .as_ref()
                                            .unwrap()
                                            .properties
                                            .iter()
                                            .find(|p| {
                                                p.name.eq_ignore_ascii_case(classification.as_str())
                                            });
                                        let property = match property {
                                            Some(property) => property.clone(),
                                            None => self
                                                .client
                                                .post_property(&String::from(classification))?,
                                        };

                                        let item = ModelMetadataItem::new(
                                            property.id.clone(),
                                            String::from(classification),
                                            String::from(tag.unwrap()),
                                        );

                                        debug!(
                                            "Setting property {} to value of {} for model {}",
                                            classification,
                                            tag.unwrap(),
                                            model.uuid
                                        );
                                        self.client.put_model_property(
                                            &uuid,
                                            &property.id,
                                            &item,
                                        )?;
                                    }
                                    None => (),
                                }

                                match metadata {
                                    Some(metadata) => {
                                        model_match.model.metadata =
                                            Some(metadata.properties.to_owned())
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
                Err(e) => return Err(anyhow!("{}", e)),
            };
        }

        Ok(ListOfModelMatches::new(Box::new(list_of_matches)))
    }

    pub fn match_scan_model(
        &self,
        uuid: &Uuid,
        threshold: f64,
        with_meta: bool,
        classification: Option<&String>,
        tag: Option<&String>,
    ) -> anyhow::Result<ListOfModelMatches> {
        trace!("Scan match model {}...", uuid);
        let mut list_of_matches: Vec<ModelMatch> = Vec::new();

        let mut has_more = true;
        let mut page: u32 = 1;
        let per_page: u32 = 50;
        while has_more {
            match self
                .client
                .get_model_scan_match_page(uuid, threshold, per_page, page)
            {
                Ok(result) => {
                    if result.page_data.total > 0 {
                        let matches = result.matches;
                        if !matches.is_empty() {
                            debug!("Reading the list of properties for model {}...", uuid);
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
                                        let property = properties
                                            .as_ref()
                                            .unwrap()
                                            .properties
                                            .iter()
                                            .find(|p| {
                                                p.name.eq_ignore_ascii_case(classification.as_str())
                                            });
                                        let property = match property {
                                            Some(property) => property.clone(),
                                            None => self
                                                .client
                                                .post_property(&String::from(classification))?,
                                        };

                                        let item = ModelMetadataItem::new(
                                            property.id.clone(),
                                            String::from(classification),
                                            String::from(tag.unwrap()),
                                        );

                                        debug!(
                                            "Setting property {} to value of {} for model {}",
                                            classification,
                                            tag.unwrap(),
                                            model.uuid
                                        );
                                        self.client.put_model_property(
                                            &uuid,
                                            &property.id,
                                            &item,
                                        )?;
                                    }
                                    None => (),
                                }

                                match metadata {
                                    Some(metadata) => {
                                        model_match.model.metadata =
                                            Some(metadata.properties.to_owned())
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
                Err(e) => return Err(anyhow!("{}", e)),
            };
        }

        Ok(ListOfModelMatches::new(Box::new(list_of_matches)))
    }

    pub fn set_property(&self, name: &String) -> Result<Property> {
        self.client.post_property(name)
    }

    pub fn set_model_property(
        &self,
        model_uuid: &Uuid,
        id: &u64,
        item: &ModelMetadataItem,
    ) -> Result<ModelMetadataItem> {
        self.client.put_model_property(model_uuid, id, item)
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
    ) -> Result<ListOfFolders> {
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
            return Err(anyhow!(format!(
                "Folder not found: {}",
                diff.iter().join(", ")
            )));
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
        folders: HashSet<String>,
        exclusive: bool,
        with_meta: bool,
    ) -> anyhow::Result<SimpleDuplicatesMatchReport> {
        let mut simple_match_report = SimpleDuplicatesMatchReport::new();

        // read the list of folders currently existing in the tenant
        let existing_folders = self.get_list_of_folders()?;

        // create a sublist only from folders that a validated to be found in the tenant
        let folders = self.validate_folders(&existing_folders, &folders)?;

        for uuid in uuids {
            let model = self.get_model(&uuid, true, with_meta);
            match model {
                Ok(model) => {
                    if model.state.eq("finished") {
                        let matches =
                            self.match_model(&uuid, threshold.clone(), with_meta, None, None)?;

                        let mut simple_duplicate_matches: Vec<ModelMatch> = Vec::new();

                        for m in *matches.inner {
                            if !exclusive
                                || (exclusive
                                    && folders.get_folder_by_id(&m.model.folder_id).is_some())
                                    && (!model.name.eq(&m.model.name)
                                        && !simple_duplicate_matches.contains(&m))
                            {
                                let mut m1 = m.clone();
                                let comparison_url: String = format!(
                                    "https://{}.physna.com/app/compare?modelAId={}&modelBId={}",
                                    self.client.tenant,
                                    uuid.to_string(),
                                    m1.model.uuid.to_string()
                                );

                                m1.comparison_url = Some(comparison_url);
                                m1.model_one_thumbnail = m.model_one_thumbnail.to_owned();
                                m1.model_two_thumbnail = m.model_two_thumbnail.to_owned();
                                simple_duplicate_matches.push(m1.to_owned());
                            }
                        }

                        if !simple_duplicate_matches.is_empty() {
                            let item = ModelMatchReportItem {
                                uuid: uuid.to_string(),
                                name: model.name.to_owned(),
                                folder_id: model.folder_id.to_owned(),
                                matches: simple_duplicate_matches,
                            };

                            simple_match_report.inner.insert(uuid.to_string(), item);
                        }
                    } else {
                        warn!(
                            "Model {} has state of {}. Skipping model match!",
                            uuid, model.state
                        );
                    }
                }
                Err(e) => warn!("Failed to query for model {}, because of: {}", uuid, e),
            }
        }

        Ok(simple_match_report)
    }

    pub fn generate_model_match_report(
        &mut self,
        uuids: Vec<Uuid>,
        threshold: f64,
        with_meta: bool,
    ) -> anyhow::Result<ModelMatchReport> {
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
            HashSet::new(),
            false,
            with_meta,
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
    ) -> anyhow::Result<EnvironmentStatusReport> {
        let all_folders = self.get_list_of_folders()?;
        let all_folders: HashMap<u32, Folder> =
            all_folders.into_iter().map(|f| (f.id, f)).collect();

        let models = self.list_all_models(folders, None)?;
        let models = models.models.to_owned();
        let mut result: HashMap<u64, ModelStatusRecord> = HashMap::new();

        for model in models {
            if force_fix && !model.state.eq_ignore_ascii_case("FINISHED") {
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

    pub fn upload_model(&self, folder: &str, path: &PathBuf) -> Result<Option<Model>> {
        self.client.upload_model(folder, path)
    }

    pub fn list_all_properties(&self) -> Result<PropertyCollection> {
        trace!("Listing all properties...");
        let response = self.client.get_list_of_properties();
        match response {
            Ok(properties) => Ok(properties),
            Err(e) => return Err(anyhow!("Failed to read properties, because of: {}", e)),
        }
    }

    pub fn upload_model_metadata(&self, input_file: &str, clean: bool) -> Result<()> {
        // Get all properties and cache them. The Physna API V2 does not allow me to get property by name
        let properties = self.list_all_properties()?;
        let all_props = Rc::new(properties.properties.clone());
        let mut reverse_lookup: HashMap<UniCase<String>, u64> = properties
            .properties
            .iter()
            .map(|p| (UniCase::new(p.name.to_owned()), p.id))
            .collect();

        let mut uuids: Vec<Uuid> = Vec::new();

        let mut rdr = csv::Reader::from_reader(File::open(input_file)?);
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
                Err(e) => return Err(anyhow!("Failed to read input: {}", e)),
            };

            if property.value.is_empty() {
                self.client
                    .delete_model_property(&property.model_uuid, &id)?;
            } else {
                self.client
                    .put_model_property(&property.model_uuid, &id, &property.to_item())?;
            }
        }

        Ok(())
    }

    fn average_image_search_results(
        &self,
        max_results: u32,
        results: Vec<ListOfModels>,
    ) -> ListOfModels {
        let mut rank_map: HashMap<Uuid, Vec<usize>> = HashMap::new();
        let mut model_map: HashMap<Uuid, Model> = HashMap::new();
        let mut seen_uuids: Vec<Uuid> = Vec::new();
        let number_of_lists = results.len();

        for models in results {
            let models = models.models.to_owned();
            for (rank, model) in models.iter().enumerate() {
                model_map.insert(model.uuid, model.clone());
                rank_map
                    .entry(model.uuid)
                    .or_insert_with(Vec::new)
                    .push(rank);

                seen_uuids.push(model.uuid);
            }

            for (&uuid, ranks) in &mut rank_map {
                if !seen_uuids.contains(&uuid) {
                    ranks.push(max_results as usize)
                }
            }
        }

        // Calculating average ranks
        let mut average_ranks: HashMap<Uuid, f64> = HashMap::new();
        for (uuid, ranks) in rank_map {
            let sum: usize = ranks.iter().sum();
            let avg = sum as f64 / number_of_lists as f64;
            average_ranks.insert(uuid, avg);
        }

        // sort by average rank in assending order
        let mut sorted_average_ranks: Vec<(Uuid, f64)> = average_ranks.into_iter().collect();
        sorted_average_ranks.sort_by(|&(_, v1), &(_, v2)| {
            v1.partial_cmp(&v2).unwrap_or(std::cmp::Ordering::Greater)
        });

        let mut result = ListOfModels::default();
        for (uuid, rank) in sorted_average_ranks {
            log::trace!("UUID={}, rank={}", uuid, rank);
            result.models.push(model_map.get(&uuid).unwrap().to_owned());
        }

        result
    }

    pub fn search_by_multiple_images(
        &self,
        paths: Vec<&PathBuf>,
        max_results: u32,
        search: Option<&String>,
        filter: Option<&String>,
    ) -> Result<ListOfModels> {
        let mut results: Vec<ListOfModels> = Vec::new();

        if paths.len() == 1 {
            // optimization for a single image file, which would be most often
            self.search_by_image(&paths[0], max_results, search, filter)
        } else {
            // using miltiple image files
            for path in paths {
                let result = self.search_by_image(&path, max_results, search, filter)?;
                results.push(result);
            }

            // average the results into a single result
            let result = self.average_image_search_results(max_results, results);

            Ok(result)
        }
    }

    pub fn search_by_image(
        &self,
        path: &PathBuf,
        max_results: u32,
        search: Option<&String>,
        filter: Option<&String>,
    ) -> Result<ListOfModels> {
        let path = path.as_path();
        let image_upload = self.client.get_image_upload_specs(&path)?;
        let url = Url::parse(image_upload.upload_url.as_str()).unwrap();
        let size_requirements = image_upload.file_size_requirements;
        let mime = image_upload.headers.content_type;
        let content_range = image_upload.headers.content_length_range;
        let id = image_upload.id;

        self.client
            .upload_image_file(url, size_requirements, &path, mime, content_range)?;

        let matches = self
            .client
            .get_image_search_maches(id, search, filter, max_results, 100)?;

        Ok(matches)
    }
}
