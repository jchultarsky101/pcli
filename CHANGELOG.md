# Change Log
All notable changes to this project will be documented in this file.
 
The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

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

