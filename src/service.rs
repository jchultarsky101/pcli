use log::{
    trace, 
    warn,
    error,
};
use anyhow::{
    anyhow, 
    Result,
};
use std::collections::HashMap;
use crate::model::{
    Folder, 
    ListOfFolders, 
    Model,
    ModelMetadata,
    ModelMetadataItem,
    ModelAssemblyTree, 
    ListOfModels, 
    ModelMatch, 
    ListOfModelMatches, 
    FlatBom, 
    ModelMatchReportItem, 
    SimpleDuplicatesMatchReport, 
    ModelStatusRecord, 
    EnvironmentStatusReport, 
    PartNodeDictionaryItem,
    ModelMatchReport,
    PropertyCollection,
    Pair,
};
use crate::client::{
    AssemblyTree,
    ApiClient,
    ClientError,
};
use petgraph::matrix_graph::NodeIndex;
use std::hash::{
    Hash, 
    Hasher,
};
use std::collections::hash_map::DefaultHasher;
use uuid::Uuid;
use petgraph::matrix_graph::MatrixGraph;
use std::io::prelude::*;
use std::{
    fs::File,
    path::Path,
};
use unicase::UniCase;

pub struct Api{
    model_cache: HashMap<Uuid, Model>,
    client: Box<ApiClient>,
}

impl Api {

    pub fn new(base_url: String, tenant: String, access_token: String) -> Api {
        Api {
            model_cache: HashMap::new(),
            client: Box::new(ApiClient::connect(&base_url.to_owned(), &tenant.to_owned(), &access_token.to_owned())),
        }
    }

    pub fn get_list_of_folders(&self) -> Result<ListOfFolders> {
        let list = self.client.get_list_of_folders();
        match list {
            Ok(list) => Ok(ListOfFolders::from(list)),
            Err(e) => {
                match e {
                    ClientError::Parsing(message) => {
                        error!("{}", message);
                        return Err(anyhow!("{}", message))
                    },
                    _ => return Err(anyhow!(e)),
                }
            },
        }
    }

    pub fn get_model_metadata(&mut self, uuid: &Uuid) -> anyhow::Result<Option<ModelMetadata>> {
        Ok(self.client.get_model_metadata(uuid)?)
    }

    pub fn get_model(&mut self, uuid: &Uuid, use_cache: bool) -> anyhow::Result<Model> {

        if use_cache {
            let model_from_cache = self.model_cache.get(uuid);
            if let Some(model) = model_from_cache {
                trace!("Model cache hit for {}", uuid.to_string());
                return Ok(model.clone())
            }
        }
        let model = self.client.get_model(uuid);
    
        match model {
            Ok(response) => {
                let mut model = Model::from(response);
                let metadata = self.get_model_metadata(uuid);
                match metadata {
                    Ok(metadata) => model.metadata = metadata.to_owned(),
                    Err(_) => (),
                }
                self.model_cache.insert(model.uuid.to_owned(), model.to_owned());
                Ok(model)
            },
            Err(e) => Err(anyhow!("Failed to read model {}, because of: {}", uuid.to_string(), e)),
        }
    }

    pub fn reprocess_model(&self, uuid: &Uuid) -> anyhow::Result<()> {
        self.client.reprocess_model(uuid)?;
        Ok(())
    }

    pub fn get_model_assembly_tree(&mut self, uuid: &Uuid)  -> anyhow::Result<ModelAssemblyTree> {

        trace!("Reading assembly tree data for {}...", uuid.to_string());
        match self.client.get_assembly_tree_for_model(uuid) {
            Ok(tree) => {
                Ok(self.enhance_assembly_tree_with_model(uuid, &tree)?)
            },
            Err(e) => Err(anyhow!("Failed to read assembly tree for model {}, because of: {}", uuid.to_string(), e)),
        }
    }

    fn enhance_assembly_tree_with_model(&mut self, uuid: &Uuid, tree: &AssemblyTree)  -> anyhow::Result<ModelAssemblyTree> {

        trace!("Enhancing model data for {}...", uuid.to_string());

        let model = self.get_model(uuid, true)?;
        let assembly_tree = match &tree.children {
            Some(tree_children) => {
                let mut assembly_children: Vec<ModelAssemblyTree> = Vec::new();
                for child in tree_children {
                    let child_uuid = Uuid::parse_str(&child.uuid.as_str())?;
                    assembly_children.push(self.enhance_assembly_tree_with_model(&child_uuid, child)?);
                }
               ModelAssemblyTree::new(model, Some(assembly_children))
            },
            None => {
                ModelAssemblyTree::new(model, None)
            },
        };
    
        Ok(assembly_tree)
    }

    pub fn list_all_models(&mut self, folders: Option<Vec<u32>>, search: Option<String>) -> Result<ListOfModels> {

        trace!("Listing all models for folders {:?}...", folders);
        let mut list_of_models: Vec<Model> = Vec::new();
    
        let mut has_more = true;
        let mut page: u32 = 1;
        let per_page: u32 = 10;
        while has_more {
            match self.client.get_list_of_models_page(folders.to_owned(), search.to_owned(), per_page, page) {
                Ok(result) => {
                    if result.page_data.total > 0 {
                        let models = result.models;
                        if !models.is_empty() {
                            for m in models {
                                let mut normalized_model = Model::from(m);
                                let metadata = self.get_model_metadata(&normalized_model.uuid)?;
                                normalized_model.metadata = metadata;
                                list_of_models.push(normalized_model);
                            }
                        }
                    }
                    has_more = result.page_data.current_page < result.page_data.last_page;
                    page = result.page_data.current_page + 1;
                },
                Err(e) => {
                    return Err(anyhow!("{}", e))
                }
            };
        }
        
        let result = ListOfModels::from(list_of_models);
        //trace!("List of Models: {:?}", result);
        Ok(result)
    }

    pub fn match_model(&self, uuid: &Uuid, threshold: f64) -> anyhow::Result<ListOfModelMatches> {

        trace!("Matching model {}...", uuid);    
        let mut list_of_matches: Vec<ModelMatch> = Vec::new();
    
        let mut has_more = true;
        let mut page: u32 = 1;
        let per_page: u32 = 30;
        while has_more {
            match self.client.get_model_match_page(uuid, threshold, per_page, page) {
                Ok(result) => {
                    if result.page_data.total > 0 {
                        let matches = result.matches;
                        if !matches.is_empty() {
                            for m in matches {
                                let model_match = ModelMatch::from(m);
                                list_of_matches.push(model_match);
                            }
                        }
                    }
                    has_more = result.page_data.current_page < result.page_data.last_page;
                    page = result.page_data.current_page + 1;
                },
                Err(e) => {
                    return Err(anyhow!("{}", e))
                }
            };
        }
            
        Ok(ListOfModelMatches::new(Box::new(list_of_matches)))
    }

    fn generate_graph_from_assembly_tree(&self, parent_node_index: Option<NodeIndex>, graph: &mut MatrixGraph<String, f64>, dictionary: &mut HashMap<Uuid, PartNodeDictionaryItem>, trees: &Vec<ModelAssemblyTree>) {

        for tree in trees {
            //let parent_uuid = Uuid::parse_str(tree.model.uuid.as_str()).unwrap();
            let node_name = tree.model.name.to_owned();
            let node_index = graph.add_node(node_name);
            let node_dictionary_item = PartNodeDictionaryItem{
                                    name: tree.model.name.to_owned(),
                                    uuid: tree.model.uuid.to_owned(),
                                    node: node_index.index(),
                                };
            dictionary.insert(node_dictionary_item.uuid, node_dictionary_item);
    
            match parent_node_index {
                Some(parent_node_index) => {
                    graph.add_edge(parent_node_index, node_index, 1.0);
                },
                None => (),
            }
            
            let children = tree.children.to_owned();
            if tree.children.is_some() {
                self.generate_graph_from_assembly_tree(Some(node_index), graph, dictionary, &children.unwrap());
            }
        }
    }

    pub fn generate_simple_model_match_report(&mut self, uuids: Vec<Uuid>, threshold: f64, folders: Option<Vec<u32>>, exclusive: bool) -> anyhow::Result<SimpleDuplicatesMatchReport> {

        let mut simple_match_report = SimpleDuplicatesMatchReport::new();

        for uuid in uuids {
            let model = self.get_model(&uuid, true)?;
            if model.state.eq("finished") {
                let matches = self.match_model(&uuid, threshold)?;
    
                let mut simple_duplicate_matches: Vec<ModelMatch> = Vec::new();
                for m in *matches.inner {
                    if !exclusive || folders.is_none() || (exclusive && folders.clone().is_some() && folders.clone().unwrap().contains(&m.model.folder_id)) &&
                       (!model.name.eq(&m.model.name) && !simple_duplicate_matches.contains(&m)) {
                        let mut m1 = m.clone();
                        let comparison_url: String = format!("https://{}.physna.com/app/compare?modelAId={}&modelBId={}", self.client.tenant, uuid.to_string(), m1.model.uuid.to_string());
                        
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
                warn!("Model {} has state of {}. Skipping model match!", uuid, model.state);
            }
        }

        Ok(simple_match_report)
    }

    pub fn generate_model_match_report(&mut self, uuids: Vec<Uuid>, threshold: f64) -> anyhow::Result<ModelMatchReport> {

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
                },
                Err(e) => warn!("Error while matching {}: {}", uuid.to_string(), e),
            }
        }

        let target_uuids: Vec<Uuid> = flat_bom.inner.to_owned().keys().map(|uuid| Uuid::parse_str(uuid.as_str()).unwrap()).collect();
        let simple_match_report = self.generate_simple_model_match_report(target_uuids, threshold, None, false)?;
        
        // Create the DAG
        let mut graph: MatrixGraph<String, f64> = MatrixGraph::new();
        self.generate_graph_from_assembly_tree(None, &mut graph, &mut dictionary, &roots.values().cloned().collect());
    
        //let matrix = generate_matrix_from_match_report(&simple_match_report, &dictionary);
    
        Ok(ModelMatchReport{
            duplicates: simple_match_report,
            dictionary: dictionary,
            graph: graph,
            //matrix: matrix,
        })
    }

    pub fn tenant_stats(&mut self, folders: Option<Vec<u32>>) -> anyhow::Result<EnvironmentStatusReport> {

        let all_folders = self.get_list_of_folders()?;
        let all_folders: HashMap<u32, Folder> = all_folders.folders.into_iter().map(|f| (f.id, f)).collect();
    
        let models = self.list_all_models(folders.to_owned(), None)?;
        let models = models.models.to_owned();
        let mut result: HashMap<u64, ModelStatusRecord> = HashMap::new();
    
        for model in models {
            let folder_id = model.folder_id;
            let folder_name = all_folders.get(&folder_id).unwrap().name.to_owned();
            let folder_name2 = folder_name.to_owned();
            let stat = ModelStatusRecord::new(folder_id, folder_name, model.file_type.to_uppercase(), model.state.to_uppercase(), 1);
            let mut s = DefaultHasher::new();
            stat.hash(&mut s);
            let h = s.finish();
            let stat_as_found = result.get(&h);
            match stat_as_found {
                Some(s) => {
                    let s2 = ModelStatusRecord::new(folder_id, folder_name2, model.file_type.to_uppercase(), model.state.to_uppercase(), s.count + 1);
                    result.insert(h, s2);
                },
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

    pub fn upload_file(&self, folder_id: u32, file: &str, batch_uuid: Uuid, units: &str) -> Result<Option<Model>> {
        const CAPACITY: usize = 1000000;
        let mut f = File::open(file)?;
        let mut total_size: u64 = 0;
        let file_size = f.metadata().unwrap().len();
        let mut start_index = 0;
        let mut result: Result<Option<Model>> = Err(anyhow!("Failed to upload file"));

        let file_name = Path::new(&file.to_owned()).file_name().unwrap().to_os_string().into_string().unwrap();
        let mut source_id_resolved = String::new();
        source_id_resolved.push_str("/");
        source_id_resolved.push_str(Uuid::new_v4().to_string().as_str());
        source_id_resolved.push_str("/");
        source_id_resolved.push_str(file_name.as_str());
    
        trace!("Uploading file {} with size {} byte(s)...", file.to_owned(), file_size.to_owned());
        while total_size < file_size {
            let buffer = &mut [0 as u8; CAPACITY];
            let chunk_size: usize = f.read(buffer)?;
    
            total_size += chunk_size as u64;
            let end_index = start_index + chunk_size as u64 - 1;
            result = self.client.upload_file_chunk(folder_id, file, source_id_resolved.to_owned().as_str(), batch_uuid, units, start_index, end_index, file_size, Box::new(buffer[0..chunk_size].to_vec()));

            match result {
                Ok(_) => start_index = end_index + 1,
                Err(e) => return Err(e),
            }
        };
    
        result
    }

    pub fn list_all_properties(&self) -> Result<PropertyCollection> {

        trace!("Listing all properties...");
        let response = self.client.get_list_of_properties();
        match response {
            Ok(properties) => Ok(properties),
            Err(e) => return Err(anyhow!("Failed to read properties, because of: {}", e)),
        }
    }

    pub fn upload_model_metadata(&self, uuid: &Uuid, input_file: &str) -> Result<()> {

        // Get all properties and cache them. The Physna API V2 does not allow me to get property by name
        let properties = self.list_all_properties()?;
        let reverse_lookup: HashMap<UniCase<String>, u64> = properties.properties.into_iter().map(|p| (UniCase::new(p.name.to_owned()), p.id)).collect();

        let mut rdr = csv::Reader::from_reader(File::open(input_file)?);
        for record in rdr.records() {
            let property = match record {
                Ok(record) => {
                    let pair: Pair = record.deserialize(None)?;
                    let case_insensitive_name: UniCase<String> = UniCase::new(pair.name.to_owned());
                    let property = match reverse_lookup.get(&case_insensitive_name) {
                        Some(id) => ModelMetadataItem::new(uuid.to_owned(), *id, pair.name, pair.value),
                        None => {
                            let p = self.client.post_property(&pair.name)?;
                            ModelMetadataItem::new(uuid.to_owned(), p.id, p.name.to_owned(), pair.value.to_owned())
                        },
                    };

                    property
                },
                Err(e) => return Err(anyhow!("Failed to read input: {}", e)),
            };

            self.client.put_model_property(&property)?;
        }        

        Ok(())
    }
}