use crate::model::{
    EnvironmentStatusReport, Folder, ListOfFolders, ListOfGeoClassifierPredictions,
    ListOfMatchedMetadataItems, ListOfModelMatches, ListOfModels, ListOfVisualModelMatches, Model,
    ModelAssemblyTree, ModelMetadata, PropertyCollection, SimpleDuplicatesMatchReport, ToCsv,
    ToHtml, ToJson,
};
use colored::*;
use ptree::print_tree;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("Unsupported format '{0}'")]
    UnsupportedFormat(String),
    #[error("JSON parsing error")]
    JsonParsingError(#[from] serde_json::Error),
    #[error("CSV parsing error")]
    CsvError(#[from] csv::Error),
    #[error("Parsing error")]
    ParsingError(#[from] crate::model::ParsingError),
    #[error("I/O error")]
    InputOutputError(#[from] std::io::Error),
}

#[derive(Debug, PartialEq)]
pub enum Format {
    Json,
    Csv,
    Tree,
    Html,
}

impl FromStr for Format {
    type Err = FormatError;
    fn from_str(input: &str) -> Result<Format, Self::Err> {
        match input {
            "JSON" => return Ok(Format::Json),
            "CSV" => return Ok(Format::Csv),
            "TREE" => return Ok(Format::Tree),
            "HTML" => return Ok(Format::Html),
            _ => Err(FormatError::UnsupportedFormat(input.to_string())),
        }
    }
}

impl ToString for Format {
    fn to_string(&self) -> String {
        match self {
            Format::Json => "JSON".to_string(),
            Format::Csv => "CSV".to_string(),
            Format::Tree => "TREE".to_string(),
            Format::Html => "HTML".to_string(),
        }
    }
}

fn color_string(message: &str, color: Option<Color>) -> colored::ColoredString {
    match color {
        Some(color) => colored::ColoredString::from(message).color(color),
        None => colored::ColoredString::from(message),
    }
}

pub fn format_list_of_folders(
    folders: ListOfFolders,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    let folders = ListOfFolders::from(folders);
    match format {
        Format::Json => Ok(color_string(folders.to_json(pretty)?.as_str(), color)),
        Format::Csv => Ok(color_string(folders.to_csv(pretty)?.as_str(), color)),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_folder(
    folder: Folder,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    let folder = Folder::from(folder);
    match format {
        Format::Json => Ok(color_string(folder.to_json(pretty)?.as_str(), color)),
        Format::Csv => Ok(color_string(folder.to_csv(pretty)?.as_str(), color)),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_model(
    model: &Model,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    match format {
        Format::Json => Ok(color_string(model.to_json(pretty)?.as_str(), color)),
        Format::Csv => Ok(color_string(model.to_csv(pretty)?.as_str(), color)),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_model_metadata(
    uuid: &Uuid,
    meta: &ModelMetadata,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    match format {
        Format::Json => Ok(color_string(meta.to_json(pretty)?.as_str(), color)),
        Format::Csv => Ok(color_string(
            meta.to_enhanced_csv(uuid, pretty)?.as_str(),
            color,
        )),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_list_of_models(
    models: &ListOfModels,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    match format {
        Format::Json => Ok(color_string(models.to_json(pretty)?.as_str(), color)),
        Format::Csv => Ok(color_string(models.to_csv(pretty)?.as_str(), color)),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_enhanced_assembly_tree(
    enhanced_assembly_tree: &ModelAssemblyTree,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    match format {
        Format::Json => Ok(color_string(
            enhanced_assembly_tree.to_json(pretty)?.as_str(),
            color,
        )),
        Format::Tree => {
            print_tree(enhanced_assembly_tree)?;
            Ok(colored::ColoredString::from(""))
        }
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_list_of_model_matches(
    list_of_model_matches: &ListOfModelMatches,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    match format {
        Format::Json => Ok(color_string(
            list_of_model_matches.to_json(pretty)?.as_str(),
            color,
        )),
        Format::Csv => Ok(color_string(
            list_of_model_matches.to_csv(pretty)?.as_str(),
            color,
        )),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_list_of_visual_model_matches(
    list_of_visual_model_matches: &ListOfVisualModelMatches,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    match format {
        Format::Json => Ok(color_string(
            list_of_visual_model_matches.to_json(pretty)?.as_str(),
            color,
        )),
        Format::Csv => Ok(color_string(
            list_of_visual_model_matches.to_csv(pretty)?.as_str(),
            color,
        )),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_list_of_geo_matches(
    list_of_model_matches: &ListOfGeoClassifierPredictions,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    match format {
        Format::Json => Ok(color_string(
            list_of_model_matches.to_json(pretty)?.as_str(),
            color,
        )),
        Format::Csv => Ok(color_string(
            list_of_model_matches.to_csv(pretty)?.as_str(),
            color,
        )),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_list_of_properties(
    properties: &PropertyCollection,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    match format {
        Format::Json => Ok(color_string(properties.to_json(pretty)?.as_str(), color)),
        Format::Csv => Ok(color_string(properties.to_csv(pretty)?.as_str(), color)),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_simple_duplicates_match_report(
    bom: &SimpleDuplicatesMatchReport,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    match format {
        Format::Json => Ok(color_string(bom.to_json(pretty)?.as_str(), color)),
        Format::Csv => Ok(color_string(bom.to_csv(pretty)?.as_str(), color)),
        Format::Html => Ok(color_string(bom.to_html()?.as_str(), color)),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_environment_status_report(
    stats: &EnvironmentStatusReport,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    match format {
        Format::Json => Ok(color_string(stats.to_json(pretty)?.as_str(), color)),
        Format::Csv => Ok(color_string(stats.to_csv(pretty)?.as_str(), color)),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}

pub fn format_list_of_matched_properties(
    props: &ListOfMatchedMetadataItems,
    format: &Format,
    pretty: bool,
    color: Option<Color>,
) -> Result<colored::ColoredString, FormatError> {
    match format {
        Format::Json => Ok(color_string(props.to_json(pretty)?.as_str(), color)),
        Format::Csv => Ok(color_string(props.to_csv(pretty)?.as_str(), color)),
        _ => Err(FormatError::UnsupportedFormat(format.to_string())),
    }
}
