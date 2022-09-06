use crate::model::{
    Folder,
    Model, 
    ModelMetadata,
    ListOfModels, 
    ListOfFolders, 
    ModelAssemblyTree, 
    SimpleDuplicatesMatchReport, 
    ListOfModelMatches, 
    EnvironmentStatusReport,
    PropertyCollection,
    ListOfImageClassifiers,
    ListOfClassificationScores,
    ToJson, 
    ToCsv
};
use anyhow::{
    anyhow, 
    Result
};
use colored::*;
use std::str::FromStr;
use ptree::print_tree;

#[derive(Debug, PartialEq)]
pub enum Format {
    Json,
    Csv,
    Tree,
}

impl FromStr for Format {
    type Err = ();
    fn from_str(input: &str) -> Result<Format, Self::Err> {
        match input {
            "JSON" => return Ok(Format::Json),
            "CSV" => return Ok(Format::Csv),
            "TREE" => return Ok(Format::Tree),
            _ => Err(()),
        }
    }
}

fn color_string(message: &str, color: Option<Color>) -> colored::ColoredString {
    match color {
        Some(color) => {
            colored::ColoredString::from(message).color(color)
        },
        None => {
            colored::ColoredString::from(message)
        }
    }
}

pub fn format_list_of_folders(folders: ListOfFolders, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    let folders = ListOfFolders::from(folders);
    match format {
        Format::Json => {
           Ok(color_string(folders.to_json(pretty)?.as_str(), color))
        },
        Format::Csv => {
            Ok(color_string(folders.to_csv(pretty)?.as_str(), color))
        },
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}

pub fn format_folder(folder: Folder, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    let folder = Folder::from(folder);
    match format {
        Format::Json => {
           Ok(color_string(folder.to_json(pretty)?.as_str(), color))
        },
        Format::Csv => {
            Ok(color_string(folder.to_csv(pretty)?.as_str(), color))
        },
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}

pub fn format_model(model: &Model, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    match format {
        Format::Json => {
           Ok(color_string(model.to_json(pretty)?.as_str(), color))
        },
        Format::Csv => {
            Ok(color_string(model.to_csv(pretty)?.as_str(), color))
         },
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}

pub fn format_model_metadata(meta: &ModelMetadata, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    match format {
        Format::Json => {
           Ok(color_string(meta.to_json(pretty)?.as_str(), color))
        },
        Format::Csv => {
            Ok(color_string(meta.to_csv(pretty)?.as_str(), color))
         },
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}

pub fn format_list_of_models(models: &ListOfModels, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    match format {
        Format::Json => {
           Ok(color_string(models.to_json(pretty)?.as_str(), color))
        },
        Format::Csv => {
            Ok(color_string(models.to_csv(pretty)?.as_str(), color))
        },
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}

pub fn format_enhanced_assembly_tree(enhanced_assembly_tree: &ModelAssemblyTree, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    match format {
        Format::Json => {
           Ok(color_string(enhanced_assembly_tree.to_json(pretty)?.as_str(), color))
        },
        Format::Tree => {
            print_tree(enhanced_assembly_tree)?;
            Ok(colored::ColoredString::from(""))
        }
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}

pub fn format_list_of_model_matches(list_of_model_matches: &ListOfModelMatches, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    match format {
        Format::Json => {
           Ok(color_string(list_of_model_matches.to_json(pretty)?.as_str(), color))
        },
        Format::Csv => {
            Ok(color_string(list_of_model_matches.to_csv(pretty)?.as_str(), color))
        },
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}

pub fn format_list_of_properties(properties: &PropertyCollection, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    match format {
        Format::Json => {
           Ok(color_string(properties.to_json(pretty)?.as_str(), color))
        },
        Format::Csv => {
            Ok(color_string(properties.to_csv(pretty)?.as_str(), color))
        },
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}

pub fn format_simple_duplicates_match_report(bom: &SimpleDuplicatesMatchReport, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    match format {
        Format::Json => {
           Ok(color_string(bom.to_json(pretty)?.as_str(), color))
        },
        Format::Csv => {
            Ok(color_string(bom.to_csv(pretty)?.as_str(), color))
        },
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}

pub fn format_environment_status_report(stats: &EnvironmentStatusReport, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    match format {
        Format::Json => {
           Ok(color_string(stats.to_json(pretty)?.as_str(), color))
        },
        Format::Csv => {
            Ok(color_string(stats.to_csv(pretty)?.as_str(), color))
        },
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}

pub fn format_list_of_classifiers(classifiers: ListOfImageClassifiers, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    match format {
        Format::Json => {
           Ok(color_string(classifiers.to_json(pretty)?.as_str(), color))
        },
        Format::Csv => {
            Ok(color_string(classifiers.to_csv(pretty)?.as_str(), color))
        },
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}

pub fn format_list_of_classification_predictions(scores: ListOfClassificationScores, format: &Format, pretty: bool, color: Option<Color>) -> anyhow::Result<colored::ColoredString> {
    match format {
        Format::Json => {
           Ok(color_string(scores.to_json(pretty)?.as_str(), color))
        },
        Format::Csv => {
            Ok(color_string(scores.to_csv(pretty)?.as_str(), color))
        },
        _ => {
            Err(anyhow!("Unsupported format {:?}", format))
        }
    }
}