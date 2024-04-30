use std::path::PathBuf;
use std::{env, cmp::Ordering};
use std::collections::{HashSet, HashMap};
use clap::{
    Arg, 
    Command, ArgAction
};
use pcli::{service, token, format, model::{self, ModelMetadata, ModelMetadataItem, ModelExtendedMetadataItem}};
use std::str::FromStr;
use dirs::home_dir;
use uuid::Uuid;
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

const PHYSNA_WHITELIST: [&str; 17] = ["3ds", "catpart", "catproduct", "glb", "igs", "iges", "prt", "x_b", "x_t", "asm", "par", "sldasm", "sldprt", "step", "stp", "stl", "ojb"];
const BANNER: &'static str = r#"

╔═╗╔═╗╦  ╦
╠═╝║  ║  ║
╩  ╚═╝╩═╝╩
          
Physna Command Line Interface
"#;

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
        .before_long_help(BANNER)
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
                .about("Invalidates the current access token, which will cause new token to be created next execution"),
        )      
        .subcommand(
            Command::new("model")
                .about("Reads data for a specific model")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .num_args(1)
                        .help("The model UUID")
                        .required(true)
                        .value_parser(clap::value_parser!(Uuid))
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .num_args(0)
                        .help("Enhance output with model's metadata")
                        .required(false)
                ),
        )
        .subcommand(
            Command::new("reprocess")
                .about("Reprocesses a specific model")
                .alias("reprocess-model")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .num_args(1..)
                        .value_delimiter(',')
                        .action(clap::ArgAction::Append)
                        .help("The model UUID")
                        .required(true)
                        .value_parser(clap::value_parser!(Uuid))
                ),
        )
        .subcommand(
            Command::new("delete-model")
                .about("Deletes a specific model")
                .alias("delete")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .value_delimiter(',')
                        .action(clap::ArgAction::Append)
                        .num_args(1..)
                        .help("The model UUID. You can specify multiple UUIDs to be deleted")
                        .required(true)
                        .value_parser(clap::value_parser!(Uuid))
                ),
        )
        .subcommand(
            Command::new("model-meta")
                .about("Reads the metadata (properties) for a specific model")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .num_args(1)
                        .help("The model UUID")
                        .required(true)
                        .value_parser(clap::value_parser!(Uuid))                ),
        )
        .subcommand(
            Command::new("models")
                .about("Lists available models that meet the search criteria")
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .num_args(0..)
                        .value_delimiter(',')
                        .action(clap::ArgAction::Append) 
                        .help("Optional: Folder name (e.g. --folder=myfolder). You can specify this argument multiple times. If none specified, it will return all models in the tenant")
                        .required(false)
                )
                .arg(
                    Arg::new("search")
                        .short('s')
                        .long("search")
                        .num_args(1)
                        .help("Optional: Search clause to further filter output (e.g. a model name)")
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
                        .num_args(1)
                        .help("The model UUID")
                        .required(true)
                        .value_parser(clap::value_parser!(Uuid))
                ),
        )
        .subcommand(
            Command::new("match-model")
                .about("Matches all models to the specified one")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .num_args(1)
                        .help("The model UUID")
                        .required(true)
                        .value_parser(clap::value_parser!(Uuid))
                )
                .arg(
                    Arg::new("threshold")
                        .short('t')
                        .long("threshold")
                        .num_args(1)
                        .help("Match threshold percentage (e.g. '96.5')")
                        .required(true)
                        .value_parser(clap::value_parser!(f64))
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .num_args(0)
                        .help("Enhance output with model's metadata")
                        .required(false)
                )
                .arg(
                    Arg::new("classification")
                        .long("classification")
                        .num_args(1)
                        .help("The name for the classification metadata property")
                        .required(false)
                        .requires("meta")
                        .requires("tag")
                )
                .arg(
                    Arg::new("tag")
                        .long("tag")
                        .num_args(1)
                        .help("The value for the classification metadata property")   
                ),
        )
        .subcommand(
            Command::new("match-scan")
                .about("Scan-match all models to the specified one")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .num_args(1)
                        .help("The model UUID")
                        .required(true)
                        .value_parser(clap::value_parser!(Uuid))
                )
                .arg(
                    Arg::new("threshold")
                        .short('t')
                        .long("threshold")
                        .num_args(1)
                        .help("Match threshold percentage (e.g. '96.5')")
                        .required(true)
                        .value_parser(clap::value_parser!(f64))
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .num_args(0)
                        .help("Enhance output with model's metadata")
                        .required(false)
                )
                .arg(
                    Arg::new("classification")
                        .long("classification")
                        .num_args(1)
                        .help("The name for the classification metadata property")
                        .required(false)
                        .requires("meta")
                        .requires("tag")
                )
                .arg(
                    Arg::new("tag")
                        .long("tag")
                        .num_args(1)
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
                        .num_args(1)
                        .help("Match threshold percentage (e.g. '96.5'")
                        .required(true)
                        .value_parser(clap::value_parser!(f64))
                )
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .num_args(0..)
                        .value_delimiter(',')
                        .action(clap::ArgAction::Append) 
                        .help("Optional: Folder name (e.g. --folder=myfolder). You can specify this argument multiple times. If none specified, it will return all models in the tenant")
                        .required(false)
                )
                .arg(
                    Arg::new("search")
                        .short('s')
                        .long("search")
                        .num_args(1)
                        .help("Search clause to further filter output (optional: e.g. a model name)")
                        .required(false)
                )
                .arg(
                    Arg::new("exclusive")
                        .short('e')
                        .long("exclusive")
                        .num_args(0)
                        .help("If specified, the output will include only models that belong to the input folder")
                        .required(false)
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .num_args(0)
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
                        .num_args(1)
                        .help("Folder name")
                        .required(true)                  
                        .value_parser(clap::value_parser!(String))
                )
                .arg(
                    Arg::new("threshold")
                        .short('t')
                        .long("threshold")
                        .num_args(1)
                        .help("Match threshold percentage (e.g. '96.5')")
                        .required(true)
                        .value_parser(clap::value_parser!(f64))
                )
                .arg(
                    Arg::new("classification")
                        .short('c')
                        .long("classification")
                        .num_args(1)
                        .help("The name for the classification metadata property")
                        .required(true)
                )
                .arg(
                    Arg::new("search")
                        .short('s')
                        .long("search")
                        .num_args(1)
                        .help("Search clause to further filter output (optional: e.g. a model name)")
                        .required(false)
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .num_args(0)
                        .help("Enhance output with model's metadata")
                        .required(false)
                )
                .arg(
                    Arg::new("exclusive")
                        .short('e')
                        .long("exclusive")
                        .num_args(0)
                        .help("If specified, the output will include only models that belong to the input folder")
                        .required(false)
                ),
        )
        .subcommand(
            Command::new("delete-folder")
                .about("Deletes a specific folder")
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .num_args(1)
                        .help("Folder name")
                        .required(true)                  
                        .value_parser(clap::value_parser!(String))
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .num_args(0)
                        .help("If specified, all models in the folder will be deleted")
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
                        .num_args(1)
                        .help("The model UUID")
                        .required(true)
                        .value_parser(clap::value_parser!(Uuid))
                ),
        )
        .subcommand(
            Command::new("status")
                .about("Generates a tenant's environment status summary")
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .num_args(1)
                        .help("Folder name")
                        .required(true)
                        .value_parser(clap::value_parser!(String))
                )
                .arg(
                    Arg::new("repair")
                        .short('r')
                        .long("repair")
                        .num_args(0)
                        .help("Forces repair operation on any model that is not in status FINISHED")
                        .required(false)
                )
                .arg(
                    Arg::new("noasm")
                        .long("noasm")
                        .num_args(0)
                        .help("When using --repair, this flag causes assmeblies to be ignored")
                        .required(false)
                        .requires("repair")
                ),
        )
        .subcommand(
            Command::new("upload")
                .about("Uploads a file to Physna")
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .alias("model-upload")
                        .num_args(1)
                        .help("Folder name (e.g. --folder=myfolder)")
                        .required(true)
                )
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .num_args(1)
                        .help("Path to the input file")
                        .required(true)
                        .value_parser(clap::value_parser!(PathBuf))
                )
        )
        .subcommand(
            Command::new("download")
                .about("Downloads the source CAD file for the model into the default download directory")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .num_args(1)
                        .help("The model UUID")
                        .required(true)
                        .value_parser(clap::value_parser!(Uuid))
                )
        )
        .subcommand(
            Command::new("upload-many")
                .about("Performs a bulk upload of all files in a directory")
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .num_args(1)
                        .help("Folder name (e.g. --folder=myfolder)")
                        .required(true)
                )
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .num_args(1)
                        .help("Path to the input directory")
                        .required(true)
                        .value_parser(clap::value_parser!(PathBuf))
                )
        )
        .subcommand(
            Command::new("upload-model-meta")
                .about("Reads metadata from an input CSV file and uploads it for a model specified by UUID")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .num_args(1)
                        .help("Path to the input file")
                        .required(true)
                )
                .arg(
                    Arg::new("clean")
                        .long("clean")
                        .num_args(0)
                        .help("Deletes all pre-existing metadata properties")
                        .required(false)
                )
        ) 
        .subcommand(
            Command::new("match-report")
                .about("Generates a match report for the specified models")
                .arg(
                    Arg::new("uuid")
                        .short('u')
                        .long("uuid")
                        .num_args(1)
                        .help("Top-level assembly UUID (you can provide multiple)")
                        .required(true)
                        .value_parser(clap::value_parser!(Uuid))
                )
                .arg(
                    Arg::new("threshold")
                        .short('t')
                        .long("threshold")
                        .num_args(1)
                        .help("Match threshold percentage (e.g. '96.5')")
                        .required(true)
                        .value_parser(clap::value_parser!(f64))
                )
                .arg(
                    Arg::new("duplicates")
                        .short('d')
                        .long("duplicates")
                        .num_args(1)
                        .help("Output file name to store the duplicate report in CSV format")
                        .required(true)
                )
                .arg(
                    Arg::new("graph")
                        .short('g')
                        .long("graph")
                        .num_args(1)
                        .help("Output file name to store the assembly graph in DOT Graphviz format")
                        .required(true)
                )
                .arg(
                    Arg::new("dictionary")
                        .short('r')
                        .long("dictionary")
                        .num_args(1)
                        .help("Output file name to store the index-name-uuid dictionary in JSON format")
                        .required(true)
                )
                .arg(
                    Arg::new("meta")
                        .short('m')
                        .long("meta")
                        .num_args(0)
                        .help("Enhance output with model's metadata")
                        .required(false)
                ),    
        )
        .subcommand(
            Command::new("folders")
                .about("Lists all available folders")
                .arg(
                    Arg::new("folder")
                        .short('d')
                        .long("folder")
                        .num_args(0..)
                        .value_delimiter(',')
                        .action(clap::ArgAction::Append) 
                        .help("Optional: Folder name (e.g. --folder=myfolder). You can specify this argument multiple times. If none specified, it will return all models in the tenant")
                        .required(false)
                )
        )
        .subcommand(
            Command::new("create-folder")
                .about("Creates a new folder")
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .num_args(1)
                        .required(true)
                        .help("Name of the new folder")
                )
        )
        .subcommand(
            Command::new("properties")
                .about("Lists all available metadata propertie names and their IDs"),
        )
        .subcommand(
            Command::new("image-search")
                .about("Search for 3D model based on 2D image(s) (object identification)")
                .arg(
                    Arg::new("input")
                        .action(ArgAction::Append)
                        .short('i')
                        .long("input")
                        .num_args(1..=10)
                        .help("Path to the input file (up to 10 can be provided)")
                        .required(true)
                        .value_parser(clap::value_parser!(PathBuf))
                )
                .arg(
                    Arg::new("limit")
                        .short('l')
                        .long("limit")
                        .num_args(1)
                        .help("Maximum number of results to be returned (default is 20)")
                        .required(false)
                        .default_value("20")
                        .value_parser(clap::value_parser!(u32))
                )
                .arg(
                    Arg::new("search")
                        .short('s')
                        .long("search")
                        .num_args(1)
                        .help("Search clause to further filter output (optional: e.g. a model name)")
                        .required(false)
                )
                .arg(
                    Arg::new("filter")
                        .short('f')
                        .long("filter")
                        .num_args(1)
                        .help("Physna filter expression. See: https://api.physna.com/v2/docs#model-FilterExpression")
                        .required(false)
                )
,        )
        .arg(
            Arg::new("tenant")
                .short('t')
                .long("tenant")
                .num_args(1)
                .required(true)
                .env("PCLI_TENANT")
                .help("Your tenant ID (check with your Physna admin if not sure)")
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .num_args(1)
                .required(false)
                .default_value("json")
                .env("PCLI_FORMAT")
                .help("Output data format (optional: e.g. 'json', 'csv', or 'tree')")
                .value_parser(["json", "csv", "tree", "table"])
        )
        .arg(
            Arg::new("pretty")
                .short('p')
                .long("pretty")
                .num_args(0)
                .required(false)
                .help("Produces pretty output (optional: default is 'false')")
        )
        .arg(
            Arg::new("color")
                .long("color")
                .num_args(1)
                .required(false)
                .help("Adds color to the output (optional: e.g. 'black', 'red', 'green', 'yellow', 'blue', 'magenta', 'cyan', 'white')")
                .value_parser(["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"])
        )        
        .get_matches();

    let tenant = matches.get_one::<String>("tenant").unwrap();
    let format_string = matches.get_one::<String>("format").unwrap();
    let format_string = format_string.to_uppercase();
    let output_format = match format::Format::from_str(format_string.as_str()) {
        Ok(format) => format,
        Err(_) => {
            eprintln!("Cannot initialize process with the provided configuration. Invalid format \"{}\".", format_string);
            ::std::process::exit(exitcode::USAGE);
        },
    };
    let pretty = matches.get_flag("pretty");
    let color = matches.get_one::<String>("color");

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


    let configuration = pcli::configuration::initialize(&String::from(default_configuration_file_path));
    let configuration = match configuration {
        Ok(configuration) => configuration,
        Err(e) => {
            eprintln!("Cannot initialize process with the provided configuration: {}", e);
            ::std::process::exit(exitcode::CONFIG);
        },
    };

    let api_configuration = pcli::configuration::from_client_configuration(&configuration, &tenant);

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
        Some(("folders", sub_matches)) => {
            let folders: Option<HashSet<String>> = match sub_matches.get_many::<String>("folder") {
                Some(folders) => Some(folders.cloned().map(String::from).collect()),
                None => None,
            };
            trace!("List of folders: {:?}", folders);

            let folders = api.get_list_of_folders(folders);
            match folders {
                Ok(folders) => {
                    let output = format::format_list_of_folders(folders, &output_format, pretty, color);
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
                    eprintln!("Error occurred while reading folders: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("create-folder", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name");
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
                    eprintln!("Error occurred while creating a new folder: {}", e);
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
                    eprintln!("Error occurred while reading folders: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },        
        Some(("model", sub_matches)) => {
            let meta: bool = sub_matches.get_flag("meta");
            let uuid = sub_matches.get_one::<Uuid>("uuid").unwrap();
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
        },
        Some(("model-meta", sub_matches)) => {
            let uuid = sub_matches.get_one::<Uuid>("uuid").unwrap();
            match api.get_model_metadata(&uuid) {
                Ok(meta) => {
                    match meta {
                        Some(meta) => {
                            let output = format::format_model_metadata(&uuid, &meta, &output_format, pretty, color);
                            match output {
                                Ok(output) => {
                                    println!("{}", output);
                                    ::std::process::exit(exitcode::OK);
                                },
                                Err(e) => {
                                    eprintln!("Error: {}", e);
                                    ::std::process::exit(exitcode::DATAERR); 
                                }
                            }
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
        },
        Some(("upload-model-meta", sub_matches)) => {
            let input_file = sub_matches.get_one::<String>("input").unwrap();
            let clean = sub_matches.get_flag("clean");
            match api.upload_model_metadata(&input_file, clean) {
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
            let uuid = sub_matches.get_one::<Uuid>("uuid").unwrap();
            let tree = api.get_model_assembly_tree(&uuid);
            let proper_tree = model::ModelAssemblyTree::from(tree.unwrap());

            match format::format_enhanced_assembly_tree(&proper_tree, &output_format, pretty, color) {
                Ok(output) => {
                    println!("{}", output);
                    ::std::process::exit(exitcode::OK);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ::std::process::exit(exitcode::DATAERR); 
                }
            }
        },             
        Some(("models", sub_matches)) => {
            let search = sub_matches.get_one::<String>("search");
            let folders: HashSet<String> = match sub_matches.get_many::<String>("folder") {
                Some(folders) => folders.cloned().map(String::from).collect(),
                None => HashSet::new(),
            };
            trace!("List of folders: {:?}", folders);

            match api.list_all_models(folders, search) {
                Ok(physna_models) => {
                    let models = model::ListOfModels::from(physna_models);
                    match format::format_list_of_models(&models, &output_format, pretty, color) {
                        Ok(output) => {
                            println!("{}", output);
                            ::std::process::exit(exitcode::OK);
                        },
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("match-model", sub_matches)) => {
            let uuid = sub_matches.get_one::<Uuid>("uuid").unwrap();
            let threshold = sub_matches.get_one::<f64>("threshold").unwrap();
            let with_meta = sub_matches.get_flag("meta");
            let classification = sub_matches.get_one::<String>("classification");
            let tag = sub_matches.get_one::<String>("tag");
            
            let model_matches = match api.match_model(&uuid, threshold.to_owned(), with_meta, classification, tag) {
                Ok(model_matches) => {
                    trace!("We found {} match(es)!", model_matches.inner.len());
                    model_matches
                },
                Err(e) => {
                    warn!("No matches found.");
                    eprintln!("Error: {}", e);
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
                    eprintln!("Error: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                },
            }
        },
        Some(("match-scan", sub_matches)) => {
            let uuid = sub_matches.get_one::<Uuid>("uuid").unwrap();
            let threshold = sub_matches.get_one::<f64>("threshold").unwrap();
            let with_meta = sub_matches.get_flag("meta");
            let classification = sub_matches.get_one::<String>("classification");
            let tag = sub_matches.get_one::<String>("tag");
            
            let model_matches = match api.match_scan_model(&uuid, threshold.to_owned(), with_meta, classification, tag) {
                Ok(model_matches) => {
                    trace!("We found {} match(es)!", model_matches.inner.len());
                    model_matches
                },
                Err(e) => {
                    warn!("No matches found.");
                    eprintln!("Error: {}", e);
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
                    eprintln!("Error: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                },
            }
        },
        Some(("match-folder", sub_matches)) => {
            let threshold = sub_matches.get_one::<f64>("threshold").unwrap();
            let exclusive = sub_matches.get_flag("exclusive");
            let with_meta = sub_matches.get_flag("meta");
            let search = sub_matches.get_one::<String>("search");
            let folders: HashSet<String> = sub_matches.get_many::<String>("folder").unwrap().cloned().collect();

            match api.list_all_models(folders.clone(), search) {
                Ok(physna_models) => {
                    let models = model::ListOfModels::from(physna_models);
                    let uuids: Vec<Uuid> = models.models.into_iter().map(|model| Uuid::from_str(model.uuid.to_string().as_str()).unwrap()).collect();
                    match api.generate_simple_model_match_report(uuids, threshold, folders, exclusive, with_meta) {
                        Ok(report) => {
                            let output = format::format_simple_duplicates_match_report(&report, &output_format, pretty, color); 
                            match output {
                                Ok(output) => {
                                    println!("{}", output);
                                    ::std::process::exit(exitcode::OK);
                                },
                                Err(e) => {
                                    eprintln!("Error: {}", e);
                                    ::std::process::exit(exitcode::DATAERR);
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("delete-folder", sub_matches)) => {
            let force = sub_matches.get_flag("force");
            let folders: HashSet<String> = sub_matches.get_many::<String>("folder").unwrap().cloned().collect();

            // delete all models in the folders if forced
            if force {
                match api.list_all_models(folders.clone(), None) {
                    Ok(physna_models) => {
                        let models = model::ListOfModels::from(physna_models);
                        let uuids: Vec<Uuid> = models.models.into_iter().map(|model| Uuid::from_str(model.uuid.to_string().as_str()).unwrap()).collect();
                        for uuid in uuids {
                            match api.delete_model(&uuid) {
                                Ok(()) => (),
                                Err(e) => {
                                    eprintln!("Error: {}", e);
                                    ::std::process::exit(exitcode::DATAERR);
                                }
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ::std::process::exit(exitcode::DATAERR);
                    }
                }
            }

            // attempt to delete the folder itself
            match api.delete_folder(folders) {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                },
            }
        },
        Some(("label-folder", sub_matches)) => {
            let threshold = sub_matches.get_one::<f64>("threshold").unwrap();
            let folders: HashSet<String> = sub_matches.get_many::<String>("folder").unwrap().cloned().collect();
            let classification = sub_matches.get_one::<String>("classification").unwrap();
            let exclusive = sub_matches.get_flag("exclusive");
            let search = sub_matches.get_one::<String>("search");
            let mut model_meta_cache: HashMap<Uuid, ModelMetadata> = HashMap::new();

            match api.list_all_models(folders.clone(), search) {
                Ok(physna_models) => {
                    let models = model::ListOfModels::from(physna_models);
                    let uuids: Vec<Uuid> = models.models.into_iter().map(|model| Uuid::from_str(model.uuid.to_string().as_str()).unwrap()).collect();
                    
                    debug!("Generating simple match report...");
                    
                    match api.generate_simple_model_match_report(uuids, threshold, folders.clone(), false, true) {
                        Ok(report) => {

                            let existing_folders = match api.get_list_of_folders(None) {
                                Ok(folders) => folders,
                                Err(e) => {
                                    eprintln!("Failed to retrieve the list of folders: {}", e);
                                    ::std::process::exit(exitcode::DATAERR);
                                }
                            };
                            
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
                                        let matched_model_folder_name = existing_folders.get_folder_by_id(&&matched_model.model.folder_id).unwrap().name.to_owned();
                                        if !exclusive || (exclusive && folders.contains(&matched_model_folder_name)) {
                                            let model = matched_model.model;
                                            let meta = match model_meta_cache.get(&model.uuid) {
                                                Some(meta) => meta.clone(),
                                                None => {
                                                    match api.get_model_metadata(&model.uuid).unwrap() {
                                                        Some(meta) => {    
                                                            model_meta_cache.insert(model.uuid, meta.clone());
                                                            meta
                                                        },
                                                        None => {
                                                            let meta = ModelMetadata::new(Vec::new());
                                                            model_meta_cache.insert(model.uuid, meta.clone());
                                                            meta
                                                        }
                                                    }
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
                            eprintln!("Error: {}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
            
            
        },
        Some(("reprocess", sub_matches)) => {
            let uuids: Vec<Uuid> = sub_matches.get_many::<Uuid>("uuid").unwrap().copied().collect();
            trace!("Reprocess arguments: {:?}", uuids);
            for uuid in uuids {
                match api.reprocess_model(&uuid) {
                    Ok(()) => {
                        println!();
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ::std::process::exit(exitcode::DATAERR); 
                    }
                };
            }
        },
        Some(("delete-model", sub_matches)) => {
            let uuids: Vec<Uuid> = sub_matches.get_many::<Uuid>("uuid").unwrap().copied().collect();
            for uuid in uuids {
                match api.delete_model(&uuid) {
                    Ok(()) => {
                        println!();
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ::std::process::exit(exitcode::DATAERR); 
                    }
                };
            }
        },
        Some(("status", sub_matches)) => {
            let folders: HashSet<String> = sub_matches.get_many::<String>("folder").unwrap().cloned().collect();
            let repair = sub_matches.get_flag("repair");
            let noasm = sub_matches.get_flag("noasm");
            let result = api.tenant_stats(folders, repair, noasm);
            match result {
                Ok(result) => {
                    let output = format::format_environment_status_report(&result, &output_format, pretty, color);
                    match output {
                        Ok(output) => {
                            println!("{}", output);
                            ::std::process::exit(exitcode::OK);
                        }
                        Err(e) => {
                            eprintln!("Error occurred while reading environment status: {}", e);
                            ::std::process::exit(exitcode::DATAERR);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error occurred while reading environment status: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("upload", sub_matches)) => {

            let folder = sub_matches.get_one::<String>("folder").unwrap();
            let path = sub_matches.get_one::<PathBuf>("input").unwrap();

            let mut list_of_models: Vec<model::Model> = Vec::new();

            trace!("Uploading file {}...", String::from(path.clone().into_os_string().to_string_lossy()));
            let result = api.upload_model(&folder.to_owned(), &path);
            match result {
                Ok(model) => {
                    match model {
                        Some(model) => list_of_models.push(model.clone()),
                        None => (),
                    }
                },
                Err(e) => {
                    eprintln!("Error occurred while uploading: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }

            let output = format::format_list_of_models(&model::ListOfModels::from(list_of_models), &output_format, pretty, color);
            match output {
                Ok(output) => {
                    println!("{}", output);
                    ::std::process::exit(exitcode::OK);
                }
                Err(e) => {
                    eprintln!("Error occurred while reading environment status: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("download", sub_matches)) => {
            let uuids: Vec<Uuid> = sub_matches.get_many::<Uuid>("uuid").unwrap().copied().collect();
            for uuid in uuids {
                match api.download_model(&uuid) {
                    Ok(()) => {
                        println!();
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ::std::process::exit(exitcode::DATAERR); 
                    }
                };
            }
        },
        Some(("upload-many", sub_matches)) => {

            let folder = sub_matches.get_one::<String>("folder").unwrap();
            let path = sub_matches.get_one::<PathBuf>("input").unwrap();
            let mut list_of_models: Vec<model::Model> = Vec::new();

            
            if path.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(file_name) = path.file_name() {
                                    let parts: Vec<&str> = file_name.to_str().unwrap().split('.').collect();
                                    let extension = if parts.len() > 1 {
                                        parts[1]
                                    } else {
                                        ""
                                    };
                                    trace!("File extension detected: {}", &extension);

                                    let extension = extension.to_lowercase();

                                    trace!("Uploading data file with extension: {}", &extension);
                                    
                                    if PHYSNA_WHITELIST.contains(&extension.as_str()) {
                                        if let Ok(metadata) = fs::metadata(&path) {
                                            if metadata.len() > 0 {
                                                trace!("Uploading file {}...", String::from(path.clone().into_os_string().to_string_lossy()));
                                                let result = api.upload_model(&folder.to_owned(), &path);
                                                match result {
                                                    Ok(model) => {
                                                        match model {
                                                            Some(model) => list_of_models.push(model.clone()),
                                                            None => (),
                                                        }
                                                    },
                                                    Err(e) => {
                                                        eprintln!("Error occurred while uploading: {}", e);
                                                        ::std::process::exit(exitcode::DATAERR);
                                                    }
                                                }                                             
                                            } else {
                                                trace!("Ignored file {}. It has zero size.", path.into_os_string().to_string_lossy());
                                            }
                                        }
                                    } else {
                                        trace!("Ingnored file {}. It is not an approved type.", path.into_os_string().to_string_lossy());
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                eprint!("Error: Input path is not a directory.");
                ::std::process::exit(exitcode::NOINPUT);
            }

            let output = format::format_list_of_models(&model::ListOfModels::from(list_of_models), &output_format, pretty, color);
            match output {
                Ok(output) => {
                    println!("{}", output);
                    ::std::process::exit(exitcode::OK);
                }
                Err(e) => {
                    eprintln!("Error occurred while reading environment status: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        Some(("match-report", sub_matches)) => {
            let uuids: Vec<Uuid> = sub_matches.get_many::<Uuid>("uuid").unwrap().copied().collect();
            let duplicates_file_name = sub_matches.get_one::<String>("duplicates").unwrap();
            let graph_file_name = sub_matches.get_one::<String>("graph").unwrap();
            let dictionary_file_name = sub_matches.get_one::<String>("dictionary").unwrap();

            trace!("Source UUIDs: {:?}", uuids);

            let threshold = sub_matches.get_one::<f64>("threshold").unwrap().to_owned();
            let with_meta = sub_matches.get_flag("meta");

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
                    eprintln!("Error: Failed to generate assembly graph: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },  
        Some(("image-search", sub_matches)) => {
            let file: Vec<&PathBuf> =  sub_matches.get_many::<PathBuf>("input").unwrap().collect();
            let max_results = sub_matches.get_one::<u32>("limit").unwrap();
            let search = sub_matches.get_one::<String>("search");
            let filter = sub_matches.get_one::<String>("filter");
            let scores = api.search_by_multiple_images(file, max_results.to_owned(), search, filter);
            match scores {
                Ok(scores) => {
                    let output = format::format_list_of_models(&scores, &output_format, pretty, color);
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
                    eprintln!("Error occurred while searching by image: {}", e);
                    ::std::process::exit(exitcode::DATAERR);
                }
            }
        },
        _ => unreachable!("Error: Invalid command. See help for details"),
    }

    ::std::process::exit(exitcode::OK);
}

