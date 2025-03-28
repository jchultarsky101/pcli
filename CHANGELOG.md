# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/).

## v1.10.0
* Enhanced the match-model command with --method to enable part-in-part search
* Changed the output of matches to include forward and reverse match percentage

## v1.9.16
* Removed the assembly-bom command as it is no longer supported
* Introduced the folder-tree command
* Introfuced the create-folder command
* Enhanced the delete-folder command to enable recursive deletion of sub-folders and their data

## v1.9.15
* Implemented automatic token renewal

## v1.9.14
* Fixed bug related to upload-many when file names contain multiple dots in the name

## v1.9.13
* Updated assembly tree operations

## v1.9.12
* Fixed a bug in match-report where the reference model does not have metadata and the --reference-meta flag is specified
>>>>>>> main

## v1.9.11
* Added --ignore-errors to match-report

## v1.9.10
* Added --threshold argument to the **visual-match** command
* Updated the README.md file

## v1.9.9
* Standardized column names to resolve inconsistencies in CSV format

## v1.9.8
* Bug fix related to folder names in "models" command

## v1.9.7
* Added "users" command
* Fixed bug where the owner ID was not shown in CSV format
* Fixed bug where the folder name was showing as NULL in the output of "models" command

## v1.9.6
* Enhanced the "status" command. The argument (folder) is not optional. When none provided, it will generate status for all folders in the tenant
* Added "match-all-models" command. It will match all models in all folders at a specified threshold

## v1.9.5
* Added visual-match command

## v1.9.4
* Enhanced the match-report output to show the folder names instead of folder IDs

## v1.9.3
* Enhanced the 'match-model' command by adding 'reference-meta' flag to include all metadata from the reference model in the output in addition to the candidate model metadata

## v1.9.2
* Enhanced 'image-search' command to enable new multiple image method

## v1.9.1
* Enhanced the 'label-inference' command to include 'folder' filter

## v1.9.0
* Added command 'label-inference'
* Added JT to the list of supported file formats

## v1.8.26
* Fixed a bug in image-search

## v1.8.25
* Fixed a bug in match-folder

## v1.8.24
* Enhanced the 'match-folder' command. The --folder argument is now optional. If not folder names are provided, it will match all models in all folders in the tenant

## v1.8.23
* Enhanced the 'match-folder' command by adding --meta-filter argument

## v1.8.22
* Excluded models with "No 3D Data" status from automatic reprocessing

## v1.8.21
* Added "upgrade" command to automatically update the executable when new version is available

## v1.8.20
* Fixed a bug in the match report which was causing model matches not to be included if an assembly and a model have the same name

## v1.8.19
* Bug fixes

## v1.8.18
* Bug fixes

## v1.8.17
* Enhanced the "upload-many" command to add --show-stats and --on-error arguments
* Updated the documentation

## v1.8.16
* Added optional folder name filters to the "folders" command

## v1.8.15
* Improved error messages
* Updated documentation
* Bug fixes

## v1.8.14
* Fixed a bug preventing files with special extensions to be uploaded

## v1.8.13
* Implemented model-download command
* Updated the documentation

## v1.8.12
* Improved error messages
* Implemeneted search filters using folder names instead of folder IDs
* Implemented CI/CD

## v1.8.11
* Added upload-many command for uploading the contents of an entire directory

## v1.8.10
* Now using the new model upload API

## v1.8.9
* Fixed a bug related to missing model metadata in the label-folder command

## v1.8.8
* Fixed a bug related to the --meta argument of the label-folder command
