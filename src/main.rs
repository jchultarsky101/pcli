use std::{env, cmp::Ordering};
use std::collections::HashMap;
use clap::{
    Arg, 
    Command
};
use model::{ModelMetadata, ModelMetadataItem, ModelExtendedMetadataItem};
use std::path::Path;
use std::fs::read_to_string;
use serde::{
    Serialize, 
    Deserialize
};
use std::str::FromStr;
use serde_yaml;
use dirs::home_dir;
use uuid::Uuid;
use anyhow::{
    anyhow, 
    Result
};
use exitcode;
use log::{
    trace,
    debug,
    warn,
    error
};
use petgraph::dot::Dot;
use std::fs;
use sysinfo::{
    System, 
    SystemExt
};
use std::{
    thread, 
    time::{
        self, 
        Instant
    }
};
use glob::glob;

mod client;
mod token;
mod model;
mod service;
mod format;

/// Returns a configuration object used for HTTP calls from the more generic configuration struct
fn from_client_configuration(configuration: &ClientConfiguration, tenant: &String) -> Result<model::Configuration> {

    let base_path = configuration.base_path.clone();
    let token = token::get_token_for_tenant(configuration, tenant);

    match token {
        Ok(token) => {
            Ok(model::Configuration {
                base_url: base_path,
                access_token: token.clone(),
            })
        },
        Err(e) => return Err(e),
    }
}

/// Reads the client configuration from a file 
fn initialize(configuration: &String) -> Result<ClientConfiguration> {

    let configuration = Path::new(configuration.as_str());
    match read_to_string(configuration) {
        Ok(configuration) => {
            Ok(serde_yaml::from_str(&configuration)?)            
        },
        Err(message) => {
           Err(anyhow!(format!("Cannot open configuration file {:?}, because of: {}", configuration, message)))
        }
    }
}

/// Represents a Physna tenant
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Tenant {
    #[serde(default)]
    client_id: String,
    #[serde(default)]
    client_secret: Option<String>,
    #[serde(default)]
    page_size: Option<u32>,
}

/// The client configuration contains the base path, URL to the identity provider and the currently selected tenant
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientConfiguration {
    base_path: String,
    identity_provider_url: String,
    tenants: HashMap<String, Tenant>,
}

fn validate_uuid_argument(uuid: Option<&str>) -> Uuid {
    let uuid = match uuid {
        Some(uuid) => uuid,
        None => {
            eprintln!("Error: {}", "The argument --uuid is mandatory");
            ::std::process::exit(exitcode::DATAERR);                        
        }
    };
    let uuid = Uuid::from_str(uuid);
    let uuid = match uuid {
        Ok(uuid) => uuid,
        Err(e) => {
            eprintln!("Error: Invalid UUID format: {}", e);
            ::std::process::exit(exitcode::DATAERR);
        },
    };
    uuid
}

fn validate_string_argument(name: &str, value: Option<&str>) -> String {
    let value = match value {
        Some(value) => value.to_string(),
        None => {
            eprintln!("Error: The argument {} is mandatory", name);
            ::std::process::exit(exitcode::DATAERR);
        }
    };
    value
}

/// The main application entry point
fn main() {

    //env_logger::init();
    let _log_init_result = pretty_env_logger::try_init_timed();

    let home_directory = home_dir();
    let home_directory = match home_directory {
        Some(dir) => dir,
        None => {
            eprintln!("Error: Failed to determine the home directory");
            ::std::process::exit(exitcode::DATAERR);
        }
    };
    let home_directory = String::from(home_directory.to_str().unwrap());
    let mut default_configuration_file_path = home_directory;
    default_configuration_file_path.push_str("/.pcli.conf");

    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("sysinfo")
                .about("Prints details of the current host system"),
        )
        .subcommand(
            Command::new("token")
                .about("Obtains security access token from the provider"),
        )
        .subcommand(
            Command::new("invalidate")
                .about("Invalidates the current access token"),
        )      
        .subcommand(
            Command::new("model")
                .about("Reads data for a specific model")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .takes_value(true)
                        .help("The model UUID")
                        .required(true)
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .takes_value(false)
                        .help("Enhance output with model's metadata")
                        .required(false)
                ),
        )
        .subcommand(
            Command::new("reprocess")
                .about("Reprocesses a specific model")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .takes_value(true)
                        .help("The model UUID")
                        .required(true)
                ),
        )
        .subcommand(
            Command::new("delete-model")
                .about("Deletes a specific model")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .takes_value(true)
                        .help("The model UUID")
                        .required(true)
                ),
        )
        .subcommand(
            Command::new("model-meta")
                .about("Reads the metadata (properties) for a specific model")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .takes_value(true)
                        .help("The model UUID")
                        .required(true)
                ),
        )
        .subcommand(
            Command::new("models")
                .about("Lists all available models in a folder")
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .takes_value(true)
                        .multiple_occurrences(false)
                        .help("Folder ID (e.g. --folder=1)")
                        .required(true)
                )
                .arg(
                    Arg::new("search")
                        .short('s')
                        .long("search")
                        .takes_value(true)
                        .help("Search clause to further filter output (optional: e.g. a model name)")
                        .required(false)
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .takes_value(false)
                        .help("Enhance output with model's metadata")
                        .required(false)
                ),
        )
        .subcommand(
            Command::new("assembly-tree")
                .about("Reads the model's assembly tree")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .takes_value(true)
                        .help("The model UUID")
                        .required(true)
                ),
        )
        .subcommand(
            Command::new("match-model")
                .about("Matches all models to the specified one")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .takes_value(true)
                        .help("The model UUID")
                        .required(true)
                )
                .arg(
                    Arg::new("threshold")
                        .short('t')
                        .long("threshold")
                        .takes_value(true)
                        .help("Match threshold percentage (e.g. '96.5')")
                        .required(true)
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .takes_value(false)
                        .help("Enhance output with model's metadata")
                        .required(false)
                )
                .arg(
                    Arg::new("classification")
                        .long("classification")
                        .takes_value(true)
                        .help("The name for the classification metadata property")
                        .required(false)
                        .requires("meta")
                        .requires("tag")
                )
                .arg(
                    Arg::new("tag")
                        .long("tag")
                        .takes_value(true)
                        .help("The value for the classification metadata property")   
                ),
        )
        .subcommand(
            Command::new("match-folder")
                .about("Matches all models in a folder to other models")
                .arg(
                    Arg::new("threshold")
                        .short('t')
                        .long("threshold")
                        .takes_value(true)
                        .help("Match threshold percentage (e.g. '96.5'")
                        .required(true)
                )
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .takes_value(true)
                        .multiple_occurrences(false)
                        .help("Folder ID (e.g. --folder=1)")
                        .required(true)
                )
                .arg(
                    Arg::new("search")
                        .short('s')
                        .long("search")
                        .takes_value(true)
                        .help("Search clause to further filter output (optional: e.g. a model name)")
                        .required(false)
                )
                .arg(
                    Arg::new("exclusive")
                        .short('e')
                        .long("exclusive")
                        .takes_value(false)
                        .help("If specified, the output will include only models that belong to the input folder")
                        .required(false)
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .takes_value(false)
                        .help("Enhance output with model's metadata")
                        .required(false)
                ),
        )        
        .subcommand(
            Command::new("label-folder")
                .about("Labels models in a folder based on KNN algorithm and geometric match score as distance")
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .takes_value(true)
                        .help("Folder ID (e.g. --folder=1)")
                        .required(true) 
                )
                .arg(
                    Arg::new("threshold")
                        .short('t')
                        .long("threshold")
                        .takes_value(true)
                        .help("Match threshold percentage (e.g. '96.5')")
                        .required(true)
                )
                .arg(
                    Arg::new("classification")
                        .short('c')
                        .long("classification")
                        .takes_value(true)
                        .help("The name for the classification metadata property")
                        .required(true)
                )
                .arg(
                    Arg::new("search")
                        .short('s')
                        .long("search")
                        .takes_value(true)
                        .help("Search clause to further filter output (optional: e.g. a model name)")
                        .required(false)
                )
                .arg(
                    Arg::new("exclusive")
                        .short('e')
                        .long("exclusive")
                        .takes_value(false)
                        .help("If specified, the output will include only models that belong to the input folder")
                        .required(false)
                ),
        )
        .subcommand(
            Command::new("assembly-bom")
                .about("Generates flat BoM of model IDs for model")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .takes_value(true)
                        .help("The model UUID")
                        .required(true)
                ),
        )
        .subcommand(
            Command::new("status")
                .about("Generates a tenant's environment status summary")
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .takes_value(true)
                        .multiple_occurrences(false)
                        .help("Folder ID (e.g. --folder=1)")
                        .required(true)
                ),
        )
        .subcommand(
            Command::new("upload")
                .about("Uploads a file to Physna")
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .takes_value(true)
                        .multiple_occurrences(false)
                        .help("Folder ID (e.g. --folder=1)")
                        .required(true)
                )
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .takes_value(true)
                        .multiple_occurrences(false)
                        .help("Path to the input file")
                        .required(true)
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .takes_value(true)
                        .multiple_occurrences(false)
                        .help("Input CSV file name containing additional metadata associated with this model")
                        .required(false)
                )
                .arg(
                    Arg::new("batch")
                        .short('b')
                        .long("batch")
                        .takes_value(true)
                        .multiple_occurrences(false)
                        .help("Batch UUID (Optional, if not provided new one will be generated)")
                        .required(false)
                )
                .arg(
                    Arg::new("units")
                        .long("units")
                        .takes_value(true)
                        .multiple_occurrences(false)
                        .help("The unit of measure for the model (e.g. 'inch', 'mm', etc.)")
                        .required(true)
                )
                .arg(
                    Arg::new("validate")
                        .long("validate")
                        .takes_value(false)
                        .help("Blocks until the model is in its final state")
                        .required(false)
                )
                .arg(
                    Arg::new("timeout")
                        .long("timeout")
                        .takes_value(true)
                        .requires("validate")
                        .help("When validating, specifies the timeout in seconds")
                        .required(false)
                )
                .arg(
                    Arg::new("source")
                        .long("source")
                        .takes_value(true)
                        .help("Specifies the Source ID to be used")
                        .required(false)
                )

        )
        .subcommand(
            Command::new("upload-model-meta")
                .about("Reads metadata from an input CSV file and uploads it for a model specified by UUID")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .takes_value(true)
                        .multiple_occurrences(false)
                        .help("Path to the input file")
                        .required(true)
                )
        ) 
        .subcommand(
            Command::new("match-report")
                .about("Generates a match report for the specified models")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .takes_value(true)
                        .multiple_occurrences(true)
                        .help("Top-level assembly UUID (you can provide multiple)")
                        .required(true)
                )
                .arg(
                    Arg::new("threshold")
                        .short('t')
                        .long("threshold")
                        .takes_value(true)
                        .help("Match threshold percentage (e.g. '96.5')")
                        .required(true)
                )
                .arg(
                    Arg::new("duplicates")
                        .short('d')
                        .long("duplicates")
                        .takes_value(true)
                        .help("Output file name to store the duplicate report in CSV format")
                        .required(true)
                )
                .arg(
                    Arg::new("graph")
                        .short('g')
                        .long("graph")
                        .takes_value(true)
                        .help("Output file name to store the assembly graph in DOT Graphviz format")
                        .required(true)
                )
                .arg(
                    Arg::new("dictionary")
                        .short('r')
                        .long("dictionary")
                        .takes_value(true)
                        .help("Output file name to store the index-name-uuid dictionary in JSON format")
                        .required(true)
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .takes_value(false)
                        .help("Enhance output with model's metadata")
                        .required(false)
                ),    
        )
        .subcommand(
            Command::new("folders")
                .about("Lists all available folders"),
        )
        .subcommand(
            Command::new("create-folder")
                .about("Creates a new folder")
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .takes_value(true)
                        .required(true)
                        .help("Name of the new folder")
                )
        )
        .subcommand(
            Command::new("properties")
                .about("Lists all available metadata propertie names and their IDs"),
        )
        .subcommand(
            Command::new("create-image-classifier")
                .about("Creates a new image classifier")
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .takes_value(true)
                        .required(true)
                        .help("Name of the new image classifier")
                )
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .takes_value(true)
                        .multiple_occurrences(true)
                        .help("Folder ID (you can provide multiple, e.g. --folder=1 --folder=2)")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("classifiers")
                .about("Lists all available image classifiers"),
        )
        .subcommand(
            Command::new("classification-predictions")
                .about("Read the list of classification predictions for an image by given classifier")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .takes_value(true)
                        .multiple_occurrences(true)
                        .help("Classifier UUID")
                        .required(true)
                )
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .takes_value(true)
                        .multiple_occurrences(false)
                        .help("Path to the input file")
                        .required(true)
                ),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .takes_value(true)
                .required(false)
                .help("Configuration file")
                .default_value(default_configuration_file_path.as_str())
        )
        .arg(
            Arg::new("tenant")
                .short('t')
                .long("tenant")
                .takes_value(true)
                .required(true)
                .help("Your tenant ID (check with your Physna admin if not sure)")
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .takes_value(true)
                .required(false)
                .default_value("json")
                .help("Output data format (optional: e.g. 'json', 'csv', or 'tree')")
        )
        .arg(
            Arg::new("pretty")
                .short('p')
                .long("pretty")
                .takes_value(false)
                .required(false)
                .help("Produces pretty output (optional: default is 'false')")
        )
        .arg(
            Arg::new("color")
                .long("color")
                .takes_value(true)
                .required(false)
                .help("Adds color to the output (optional: e.g. 'black', 'red', 'green', 'yellow', 'blue', 'magenta', 'cyan', 'white')")
        )        
        .get_matches();

    let tenant = validate_string_argument("tenant", matches.value_of("tenant"));
    let format_string = validate_string_argument("format", matches.value_of("format"));
    let format_string = format_string.to_uppercase();
    let output_format = match format::Format::from_str(format_string.as_str()) {
        Ok(format) => format,
        Err(_) => {
            eprintln!("Cannot initialize process with the provided configuration. Invalid format \"{}\".", format_string);
            ::std::process::exit(exitcode::USAGE);
        },
    };
    let pretty = matches.is_present("pretty");
    let color = matches.value_of("color");

    let color = match color {
        Some(color) => {
            let color = colored::Color::from_str(color);
            match color {
                Ok(color) => Some(color),
                Err(_) => None,
            }
        },
        None => None,
    };

    let configuration = matches.value_of("config");
    let configuration = match configuration {
        Some(configuration) => {
            trace!("Reading client configuration from {}...", configuration);
            let configuration = initialize(&String::from(configuration));
            match configuration {
                Ok(configuration) => configuration,
                Err(e) => {
                    eprintln!("Cannot initialize process with the provided configuration: {}", e);
                    ::std::process::exit(exitcode::CONFIG);
                },
            }
        },
        None => {
            eprintln!("No valid configuration available");
            ::std::process::exit(exitcode::CONFIG);
        },
    };

    let api_configuration = from_client_configuration(&configuration, &tenant);

    let mut api: service::Api;
    match api_configuration {
        Ok(api_configuration) => {
            api = service::Api::new(api_configuration.base_url, tenant.to_owned(), api_configuration.access_token);
        },
        Err(e) => {
            eprintln!("Invalid configuration: {}", e);
            eprintln!("Currently configured tenants:");
            for (k,_) in configuration.tenants.iter() {
                eprintln!("{}", k);
            }

            ::std::process::exit(exitcode::CONFIG);
        }
    }
    
    match matches.subcommand() {
        Some(("sysinfo", _sub_matches)) => {
            let mut sys = System::new_all();
            sys.refresh_all();

            // Display system information:
            println!("System name:             {:?}", sys.name().unwrap_or("unknown".to_string()));
            println!("System kernel version:   {:?}", sys.kernel_version().unwrap_or("unknown".to_string()));
            println!("System OS version:       {:?}", sys.os_version().unwrap_or("unknown".to_string()));
            println!("NB CPUs: {}", sys.cpus().len());
        },
        Some(("token", _sub_matches)) => {
            let token = token::get_token_for_tenant(&configuration, &tenant);
            match token {
                Ok(token) => {
                    println!("{}", token);
                    ::std::process::exit(exitcode::OK);
                },
                Err(e) => {
                    eprintln!("Failed to obtain token: {}", e);
                    ::std::process::exit(exitcode::NOPERM);
                }
            }
        },
        Some(("invalidate", _sub_matches)) => {
            match token::invalidate_token(&tenant) {
                Ok(_) => {
                    ::std::process::exit(exitcode::OK);
                },
                Err(e) => {
                    eprintln!("Error while invalidating current token: {}", e);
                    ::std::process::exit(exitcode::NOPERM);
                }
            }
        },
        Some(("folders", _sub_matches)) => {
            let folders = api.get_list_of_folders();
            match folders {
                Ok(folders) => {
                    let list_of_folders = folders.folders;
                    let list_of_folders = model::ListOfFolders::from(list_of_folders);
                    let output = format::format_list_of_folders(list_of_folders, &output_format, pretty, color);
                    match output {
                        Ok(output) => {
                            println!("{}", output);
                            ::std::process::exit(exitcode::OK);
                        },
                        Err(e) => {
                            eprintln!("Error while invalidating current token: {}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        },
                    }
                },
                Err(e) => {
                    eprintln!("Error occurred while reading folders: {}. Try invalidating the token.", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("create-folder", sub_matches)) => {
            let name = sub_matches.value_of("name");
            let name = match name {
                Some(name) => name,
                None => {
                    eprintln!("Error: The folder name argument is mandatory");
                    ::std::process::exit(exitcode::DATAERR);
                },
            };
            let folder = api.create_folder(&name.to_string());
            match folder {
                Ok(folder) => {
                    let output = format::format_folder(folder, &output_format, pretty, color);
                    match output {
                        Ok(output) => {
                            println!("{}", output);
                            ::std::process::exit(exitcode::OK);
                        },
                        Err(e) => {
                            eprintln!("Error while invalidating current token: {}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        },
                    }
                },
                Err(e) => {
                    eprintln!("Error occurred while creating a new folder: {}. Try invalidating the token.", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("properties", _sub_matches)) => {
            let properties = api.list_all_properties();
            match properties {
                Ok(properties) => {
                    let output = format::format_list_of_properties(&properties, &output_format, pretty, color);
                    match output {
                        Ok(output) => {
                            println!("{}", output);
                            ::std::process::exit(exitcode::OK);
                        },
                        Err(e) => {
                            eprintln!("Error while invalidating current token: {}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        },
                    }
                },
                Err(e) => {
                    eprintln!("Error occurred while reading folders: {}. Try invalidating the token.", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },        
        Some(("model", sub_matches)) => {
            let meta: bool = sub_matches.is_present("meta");
            if sub_matches.is_present("uuid") {
                let uuid = validate_uuid_argument(sub_matches.value_of("uuid"));
                match api.get_model(&uuid, false, meta) {
                    Ok(model) => {
                        let output = format::format_model(&model, &output_format, pretty, color).unwrap();
                        println!("{}", output);
                        ::std::process::exit(exitcode::OK);
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ::std::process::exit(exitcode::DATAERR); 
                    }
                };
            } else {
                eprintln!("Model ID not specified!");
                ::std::process::exit(exitcode::USAGE);
            }
        },
        Some(("model-meta", sub_matches)) => {
            if sub_matches.is_present("uuid") {
                let uuid = validate_uuid_argument(sub_matches.value_of("uuid"));
                match api.get_model_metadata(&uuid) {
                    Ok(meta) => {
                        match meta {
                            Some(meta) => {
                                let output = format::format_model_metadata(&uuid, &meta, &output_format, pretty, color).unwrap();
                                println!("{}", output);
                                ::std::process::exit(exitcode::OK);
                            },
                            None => {
                                println!("");
                                ::std::process::exit(exitcode::OK);
                            },
                        }

                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ::std::process::exit(exitcode::DATAERR); 
                    }
                };
            } else {
                eprintln!("Model ID not specified!");
                ::std::process::exit(exitcode::USAGE);
            }
        },
        Some(("upload-model-meta", sub_matches)) => {
            let input_file = validate_string_argument("input", sub_matches.value_of("input"));
            match api.upload_model_metadata(&input_file) {
                Ok(_) => {
                    ::std::process::exit(exitcode::OK);
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ::std::process::exit(exitcode::DATAERR); 
                }
            };
        }, 
        Some(("assembly-tree", sub_matches)) => {
            if sub_matches.is_present("uuid") {
                let uuid = validate_uuid_argument(sub_matches.value_of("uuid"));
                let tree = api.get_model_assembly_tree(&uuid);
                let proper_tree = model::ModelAssemblyTree::from(tree.unwrap());

                let output = format::format_enhanced_assembly_tree(&proper_tree, &output_format, pretty, color);
                println!("{}", output.unwrap());
                ::std::process::exit(exitcode::OK);
            } else {
                eprintln!("Model ID not specified!");
                ::std::process::exit(exitcode::USAGE);
            }
        },             
        Some(("models", sub_matches)) => {
            let search: Option<String>;
            if sub_matches.is_present("search") {
                search = Some(String::from(sub_matches.value_of("search").unwrap()));
            } else {
                search = None;
            }

            let folders: Option<Vec<u32>>;
            if sub_matches.is_present("folder") {
                let folder_id_strings = Some(String::from(sub_matches.value_of("folder").unwrap()));
                folders = Some(folder_id_strings.into_iter().map(|x| x.parse::<u32>().unwrap()).collect());
            } else {
                folders = None;
            }

            let meta: bool = sub_matches.is_present("meta");

            match api.list_all_models(folders, search, meta) {
                Ok(physna_models) => {
                    let models = model::ListOfModels::from(physna_models);
                    let output = format::format_list_of_models(&models, &output_format, pretty, color);
                    println!("{}", output.unwrap());
                    ::std::process::exit(exitcode::OK);
                },
                Err(e) => {
                    eprintln!("{}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("match-model", sub_matches)) => {
            if sub_matches.is_present("uuid") {
                let uuid = validate_uuid_argument(sub_matches.value_of("uuid"));

                let threshold = validate_string_argument("threshold", sub_matches.value_of("threshold"));
                let threshold: f64 = match threshold.parse() {
                    Ok(threshold) => threshold,
                    Err(e) => {
                        eprintln!("Error: Failed to parse the threshold value: {}", e);
                        ::std::process::exit(exitcode::DATAERR);
                    }
                };

                if (threshold > 1.0) || (threshold < 0.0) {
                    eprintln!("Error: The threshold value must be between 0 and 1");
                    ::std::process::exit(exitcode::DATAERR);
                }
                
                let with_meta = sub_matches.is_present("meta");

                let classification = sub_matches.value_of("classification");
                let tag = sub_matches.value_of("tag");
                
                let model_matches = match api.match_model(&uuid, threshold, with_meta, classification, tag) {
                    Ok(model_matches) => {
                        trace!("We found {} match(es)!", model_matches.inner.len());
                        model_matches
                    },
                    Err(e) => {
                        warn!("No matches found.");
                        eprintln!("{}", e);
                        ::std::process::exit(exitcode::DATAERR);
                    },
                };

                let output = format::format_list_of_model_matches(&model_matches, &output_format, pretty, color);
                match output {
                    Ok(output) => {
                        println!("{}", output);
                        ::std::process::exit(exitcode::OK);
                    },
                    Err(e) => {
                        eprintln!("{}", e);
                        ::std::process::exit(exitcode::DATAERR);
                    },
                };
            }
        },
        Some(("match-folder", sub_matches)) => {
            let threshold = validate_string_argument("threshold", sub_matches.value_of("threshold"));
            let threshold: f64 = match threshold.parse() {
                Ok(threshold) => threshold,
                Err(e) => {
                    eprintln!("Error: Failed to parse the threshold value: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            };
            
            if (threshold > 1.0) || (threshold < 0.0) {
                eprintln!("Error: The threshold value must be between 0 and 1");
                ::std::process::exit(exitcode::DATAERR);
            }

            let exclusive: bool = sub_matches.is_present("exclusive");
            let with_meta: bool = sub_matches.is_present("meta");

            let search: Option<String>;
            if sub_matches.is_present("search") {
                search = Some(String::from(sub_matches.value_of("search").unwrap()));
            } else {
                search = None;
            }

            let folder_id = validate_string_argument("folder", sub_matches.value_of("folder"));
            let folder_id = match folder_id.parse::<u32>() {
                Ok(folder_id) => folder_id,
                Err(e) => {
                    eprintln!("Error: Failed to parse the value for folder ID: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            };
            let mut folders_list: Vec<u32> = Vec::new();
            folders_list.push(folder_id);
            let folders_list = Some(folders_list);

            let meta: bool = sub_matches.is_present("meta");

            match api.list_all_models(folders_list.clone(), search, meta) {
                Ok(physna_models) => {
                    let models = model::ListOfModels::from(physna_models);
                    let uuids: Vec<Uuid> = models.models.into_iter().map(|model| Uuid::from_str(model.uuid.to_string().as_str()).unwrap()).collect();
                    match api.generate_simple_model_match_report(uuids, threshold, folders_list, exclusive, with_meta) {
                        Ok(report) => {
                            let output = format::format_simple_duplicates_match_report(&report, &output_format, pretty, color);
                            println!("{}", output.unwrap());
                            ::std::process::exit(exitcode::OK);
                        },
                        Err(e) => {
                            eprintln!("{}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("{}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("label-folder", sub_matches)) => {
            let threshold = validate_string_argument("threshold", sub_matches.value_of("threshold"));
            let threshold: f64 = match threshold.parse() {
                Ok(threshold) => threshold,
                Err(e) => {
                    eprintln!("Error: Failed to parse the threshold value: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            };
            
            if (threshold > 1.0) || (threshold < 0.0) {
                eprintln!("Error: The threshold value must be between 0 and 1");
                ::std::process::exit(exitcode::DATAERR);
            }
            
            let folder_id = validate_string_argument("folder", sub_matches.value_of("folder"));
            let folder_id = match folder_id.parse::<u32>() {
                Ok(folder_id) => folder_id,
                Err(e) => {
                    eprintln!("Error: Failed to parse the value for folder ID: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            };
                        
            let classification = validate_string_argument("classification", sub_matches.value_of("classification"));

            let exclusive = sub_matches.contains_id("exclusive");
            
            let mut folders_list: Vec<u32> = Vec::new();
            folders_list.push(folder_id);
            let folders_list = Some(folders_list);

            let search: Option<String> = match sub_matches.get_one("search") {
                Some(search) => {
                    let search: &String = &*search;
                    Some(search.to_owned())
                },
                None => None,
            };
            
            let mut model_meta_cache: HashMap<Uuid, ModelMetadata> = HashMap::new();

            let meta: bool = sub_matches.is_present("meta");

            debug!("Running NKK labeling for folder {}...", folder_id);
            
            match api.list_all_models(folders_list.clone(), search, meta) {
                Ok(physna_models) => {
                    let models = model::ListOfModels::from(physna_models);
                    let uuids: Vec<Uuid> = models.models.into_iter().map(|model| Uuid::from_str(model.uuid.to_string().as_str()).unwrap()).collect();
                    
                    debug!("Generating simple match report...");
                    
                    match api.generate_simple_model_match_report(uuids, threshold, folders_list, false, true) {
                        Ok(report) => {
                            
                            // ensure that the classification property is available
                            debug!("Reading master property list...");
                            let properties = api.list_all_properties();
                            let property =
                                properties.as_ref().unwrap().properties.iter().find(
                                    |p| p.name.eq_ignore_ascii_case(classification.as_str()),
                                );
                            let property = match property {
                                Some(property) => property.clone(),
                                None => api.set_property(&String::from(classification.clone())).unwrap(),
                            };
                                       
                            for (master_model_uuid, mut item) in report.inner {
                                let master_model_uuid = Uuid::from_str(master_model_uuid.as_str()).unwrap();

                                debug!("Analyzing model {}...", master_model_uuid);   
                                
                                if !item.matches.is_empty() {

                                    debug!("Found matches with threshold of {}.", threshold);
                                    
                                    // sort the list of matches by the mach score
                                    item.matches.sort_by(|a, b| {
                                        if a.percentage < b.percentage {
                                            return Ordering::Less;
                                        } else if a.percentage > b.percentage {
                                            return Ordering::Greater;
                                        }
                                        return Ordering::Equal;
                                    });
                                
                                    // reverse the sort order. Wee need the best fit on top:
                                    item.matches.reverse();

                                    debug!("Found matches for model {}, Checking for classification labels {}...", master_model_uuid, classification);
                                    
                                    for matched_model in item.matches {
                                        if !exclusive || (exclusive && matched_model.model.folder_id.eq(&folder_id)) {
                                            let model = matched_model.model;
                                            let meta = match model_meta_cache.get(&model.uuid) {
                                                Some(meta) => meta.clone(),
                                                None => {
                                                    let meta = api.get_model_metadata(&model.uuid).unwrap().unwrap();
                                                    model_meta_cache.insert(model.uuid, meta.clone());
                                                    meta
                                                },
                                            };
                                            let meta: HashMap<String, ModelMetadataItem> = meta.properties.iter().map(|p| (p.name.clone(), p.clone())).collect();
                                    
                                            let classification_value = meta.get(&classification.clone());
                                            match classification_value {
                                                Some(classification_value) => {
                                                    // set the classification value for the master model and exit the loop
                                                    //let value = classification_value.value.clone();

                                                    debug!("Matching model {} has {}={:?}", model.uuid, classification, classification_value);

                                                    if !classification_value.value.eq_ignore_ascii_case("unclassified") {
                                                        let meta_item = ModelExtendedMetadataItem::new(
                                                            master_model_uuid.clone(),
                                                            classification_value.key_id.clone(),
                                                            String::from(classification.clone()),
                                                            String::from(classification_value.value.clone()),
                                                        );

                                                        debug!("Assigning {}={:?} for model {}...", classification, classification_value, master_model_uuid);
                                                        api.set_model_property(&meta_item.model_uuid, &property.id, &meta_item.to_item()).unwrap();
                                                        break;
                                                    } else {
                                                        debug!("Ignoring the matching model's classification value.");
                                                    }
                                                },
                                                None => {
                                                    ();
                                                },
                                            }
                                        }
                                    }
                                } else {
                                    debug!("There are no matches for this model. Deleting the classification metadata...");
                                    // Did not find any matches for this model. If there was an old classification value, it needs to be deleted
                                    let _ = api.delete_model_metadata_property(&master_model_uuid, &property.id);
                                }
                            }                            
                            
                            ::std::process::exit(exitcode::OK);
                        },
                        Err(e) => {
                            eprintln!("{}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("{}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
            
            
        },
        Some(("reprocess", sub_matches)) => {
            if sub_matches.is_present("uuid") {
                let uuid = validate_uuid_argument(sub_matches.value_of("uuid"));
                match api.reprocess_model(&uuid) {
                    Ok(()) => {
                        println!();
                        ::std::process::exit(exitcode::OK);
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ::std::process::exit(exitcode::DATAERR); 
                    }
                };
            } else {
                eprintln!("Model ID not specified!");
                ::std::process::exit(exitcode::USAGE);
            }
        },
        Some(("delete-model", sub_matches)) => {
            if sub_matches.is_present("uuid") {
                let uuid = validate_uuid_argument(sub_matches.value_of("uuid"));
                match api.delete_model(&uuid) {
                    Ok(()) => {
                        println!();
                        ::std::process::exit(exitcode::OK);
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ::std::process::exit(exitcode::DATAERR); 
                    }
                };
            } else {
                eprintln!("Model ID not specified!");
                ::std::process::exit(exitcode::USAGE);
            }
        },
        Some(("status", sub_matches)) => {
            let folders: Option<Vec<u32>>;
            if sub_matches.is_present("folder") {
                let folder_id_strings = Some(String::from(sub_matches.value_of("folder").unwrap()));
                folders = Some(folder_id_strings.into_iter().map(|x| x.parse::<u32>().unwrap()).collect());
            } else {
                folders = None;
            }

            let result = api.tenant_stats(folders);
            match result {
                Ok(result) => {
                    let output = format::format_environment_status_report(&result, &output_format, pretty, color);
                    println!("{}", output.unwrap());
                    ::std::process::exit(exitcode::OK);
                },
                Err(e) => {
                    eprintln!("Error occurred while reading environment status: {}. Try invalidating the token.", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("upload", sub_matches)) => {

            let folder_id = sub_matches.value_of("folder");
            let folder_id = match folder_id {
                Some(folder_id) => {
                    match u32::from_str(&folder_id) {
                        Ok(folder_id) => folder_id,
                        Err(e) => {
                            eprintln!("Error: Invalid number format for the folder ID: {}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        }
                    }
                },
                None => {
                    eprintln!("Error: The folder ID argument is mandatory");
                    ::std::process::exit(exitcode::DATAERR);
                }
            };

            let file =  String::from(sub_matches.value_of("input").unwrap());
            let metadata_file = sub_matches.value_of("meta");
            let batch_uuid = match sub_matches.value_of("batch") {
                Some(batch_uuid) => Uuid::from_str(&batch_uuid).unwrap(),
                None => Uuid::new_v4(),
            };
            let units =  String::from(sub_matches.value_of("units").unwrap());
            let validate = sub_matches.is_present("validate");
            let timeout: Option<u64> = match sub_matches.value_of("timeout") {
                Some(duration) => Some(duration.parse::<u64>().unwrap()),
                None => None,
            };
            let source_id: Option<String> = match sub_matches.value_of("source") {
                Some(id) => Some(id.to_string()),
                None => None,
            };

            let glob = glob(file.as_str());
            match glob {
                Ok(glob) => {
                    let mut list_of_models: Vec<model::Model> = Vec::new();
                    for path in glob {
                        let file = path.unwrap().into_os_string().into_string().unwrap();
                        let result = api.upload_file(folder_id, &file, batch_uuid, &units, source_id.clone());
                        match result {
                            Ok(model) => {
                                match model {
                                    Some(model) => {
                                        let m: model::Model = match metadata_file {
                                                        Some(metadata_file) => {
                                                            let _meta_response = api.upload_model_metadata(metadata_file);
                                                            let m2 = api.get_model(&model.uuid, false, false);
                                                            m2.unwrap()
                                                        },
                                                        None => model.clone(),
                                        };
            
                                        if validate {
                                            let two_seconds = time::Duration::from_millis(2000);
                                            let start_time = Instant::now();
                                            let mut state = m.state.clone();
                                            while state.ne("finished") &&
                                                  state.ne("failed") &&
                                                  state.ne("missing-parts") {
            
                                                let duration = start_time.elapsed().as_secs();
                                                if timeout.is_some() && (duration >= timeout.unwrap()) {
                                                    ::std::process::exit(exitcode::TEMPFAIL);
                                                }
            
                                                match api.get_model(&m.uuid, false, false) {
                                                    Ok(verified_model) => {
                                                        state = verified_model.state.clone();
                                                    },
                                                    Err(_) => break,
                                                }
                                                thread::sleep(two_seconds);
                                            }
                                        }

                                        list_of_models.push(m.clone());
                                    },
                                    None => (),
                                }
                            },
                            Err(e) => {
                                eprintln!("Error occurred while uploading: {}. Try invalidating the token.", e);
                                ::std::process::exit(exitcode::DATAERR);
                            }
                        }
                    }

                    let output = format::format_list_of_models(&model::ListOfModels::from(list_of_models), &output_format, pretty, color);
                    println!("{}", output.unwrap());
                },
                Err(_) => {
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("match-report", sub_matches)) => {
            let uuids = sub_matches.values_of("uuid");
            let duplicates_file_name = sub_matches.value_of("duplicates").unwrap();
            let graph_file_name = sub_matches.value_of("graph").unwrap();
            let dictionary_file_name = sub_matches.value_of("dictionary").unwrap();

            match uuids {
                Some(uuids) => {
                    let uuids: Vec<_> = uuids.collect();
                    let uuids: Vec<Uuid> = uuids.into_iter().map(|u| Uuid::from_str(u).unwrap()).collect();
                    trace!("Source UUIDs: {:?}", uuids);

                    let threshold = sub_matches.value_of("threshold").unwrap();
                    let threshold: f64 = threshold.parse().unwrap();

                    let with_meta: bool = sub_matches.is_present("meta");
        
                    match api.generate_model_match_report(uuids, threshold, with_meta) {
                        Ok(report) => {

                            let output = format::format_simple_duplicates_match_report(&report.duplicates, &format::Format::from_str("CSV").unwrap(), false, None);
                            match fs::write(duplicates_file_name, format!("{}", &output.unwrap().to_string())) {
                                Ok(()) => (),
                                Err(e) => {
                                    error!("Failed to write duplicates report as {}, because of: {}", duplicates_file_name, e);
                                    ::std::process::exit(exitcode::DATAERR);
                                }
                            }
        
                            match fs::write(graph_file_name, format!("{}", Dot::with_config(&report.graph, &[]))) {
                                Ok(()) => (),
                                Err(e) => {
                                    error!("Failed to write graph as {}, because of: {}", graph_file_name, e);
                                    ::std::process::exit(exitcode::DATAERR);
                                }
                            }

                            match fs::write(dictionary_file_name, format!("{}", serde_json::to_string_pretty(&report.dictionary).unwrap())) {
                                Ok(()) => (),
                                Err(e) => {
                                    error!("Failed to write dictionary as {}, because of: {}", dictionary_file_name, e);
                                    ::std::process::exit(exitcode::DATAERR);
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to generate assembly graph: {}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        }
                    }
                },
                None => {
                    trace!("No list of UUIDs specified.");
                },
            }
        },  
        Some(("create-image-classifier", sub_matches)) => {
            let name = sub_matches.value_of("name").unwrap();

            let folders: Vec<u32>;
            let folder_id_strings = Some(String::from(sub_matches.value_of("folder").unwrap()));
            folders = folder_id_strings.into_iter().map(|x| x.parse::<u32>().unwrap()).collect();

            let uuid = api.create_image_classifier(name.to_string(), folders);
            match uuid {
                Ok(uuid) => {
                    println!("{}", uuid);
                    ::std::process::exit(exitcode::OK);
                },
                Err(e) => {
                    eprintln!("Error occurred while creating a new image classifier: {}. Try invalidating the token.", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("classifiers", _sub_matches)) => {
            let classifiers = api.get_image_classifiers();
            match classifiers {
                Ok(classifiers) => {
                    let output = format::format_list_of_classifiers(classifiers, &output_format, pretty, color);
                    match output {
                        Ok(output) => {
                            println!("{}", output);
                            ::std::process::exit(exitcode::OK);
                        },
                        Err(e) => {
                            eprintln!("Error while invalidating current token: {}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        },
                    }
                },
                Err(e) => {
                    eprintln!("Error occurred while reading classifiers: {}. Try invalidating the token.", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("classification-predictions", sub_matches)) => {

            let uuid = validate_uuid_argument(sub_matches.value_of("uuid"));
            let file =  String::from(sub_matches.value_of("input").unwrap());
            let scores = api.get_classification_predictions(uuid, file.as_str());
            match scores {
                Ok(scores) => {
                    let output = format::format_list_of_classification_predictions(scores, &output_format, pretty, color);
                    match output {
                        Ok(output) => {
                            println!("{}", output);
                            ::std::process::exit(exitcode::OK);
                        },
                        Err(e) => {
                            eprintln!("Error while invalidating current token: {}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        },
                    }
                },
                Err(e) => {
                    eprintln!("Error occurred while reading classification predictions: {}. Try invalidating the token.", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        _ => unreachable!("Invalid command"),
    }

    ::std::process::exit(exitcode::OK);
}
