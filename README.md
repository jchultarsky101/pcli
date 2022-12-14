# Physna CLI

This utility is a CLI client for the Physna's public API V2 (and in part API V1). It could be used to interact with the system
in automated ways.

## Physna Account

To be able to use this client, you will need to first request a Physna Enterprise account. Please, contact your Physna representative for details.

## Change Log

The latest version is 1.6.11

### Version 1.6.11

* Modified commands to consistently use the optional --meta flag:
    - model
    - models
    - match-folder
* Updated the client to use the new metadata API end points

### Version 1.6.10

* Added --search filder to the label-folder command

### Version 1.6.9

* Added --exclusive flag to the label-folder command
* Updated README.md

### Version 1.6.8

* Modified the upload-model-meta command to delete metadata property when the value is emty string. This is to eliminate undesirable side effect when searching
* Added label-folder command that uses KNN for automatic classification of models

### Version 1.6.7

* Added optional arguments to the match-model command allowing the user to set a metadata property for all models that match the source model. This in turn enables automated way to train the classification machine learning algorithm by specifying that tag. The user can also use that tag in the --search argument

### Version 1.6.6

* Added installation and build instructions to README.md

### Version 1.6.5

* Bug fixes
* Added content to the README.md

### Version 1.6.4

* Now displaying the list of metadata properties in the CSV output of match-folder, model-match
* Added optional argument --meta to selectively include the metadata in the output

### Version 1.6.0

* Added create-image-classifier, classifiers, classification-predictions

### Version 1.5.0

* Added create-folder command
* Modified the upload-model-meta command to use the same CSV file format as the output of model-meta. That includes now the UUID of the model as first column. Therefore, I removed the --uuid argument from the command. There are two reasons for this:
  * One can first query a model, make a small modification and then use that same file to upload
  * One could concatenate the metadata from multiple model-meta commands into a single master file. Similar file could be created by database extraction methods, etc.

### Version 1.4.5

* Documentation and help fixes

### Version 1.4.3

* Fixed a bug when handling a response from the delete command.

### Version 1.4.2

* Enabled Source ID argument when uploading files

#### Version 1.4.1

* Updated the CSV writer to terminate on CRLF instead of just CR

#### Version 1.3.5

* Added delete-model command

#### Version 1.3.5

* Fixed a bug causing output to be suppressed when using CSV format in model-match

#### Version 1.3.4

* Enhanced the upload function. The user can now provide a wild card to upload multiple files in one operation

#### Version 1.3.3

* Enhanced the upload process. Added optional flag --validate with a related optional arguument --timeout

#### Version 1.3.2

* Using the new comparison URL

#### Version 1.3.1

* Upgraded the versions of some libraries as per Renovate issue

#### Version 1.3.0

* Added reprocess command

#### Version 1.2.6

* Added exclusive flag to match-folder
* Added columns SOURCE_UUID, MATCH_UUID, SOURCE_FOLDER_ID, MATCH_FOLDER_ID to the match-folder command CSV output
* Removed the in-memory encryption - the crate is not supported for Windows yet

#### Version 1.2.5

* Implemented local token encryption

#### Version 1.2.4

* Bug fix: Removed automatic token retry. It was casung side effects

#### Version 1.2.3

* Removed unused command and argument: match-matrix, quiet

#### Version 1.2.2

* Printing the list of currently configured tenants if unknown tenant is specified
* Bug fix: Comparison URL in match-report was incomplete
* Bug fix: Thumbnail URL included unnecessary parameters

#### Version 1.1.1

* Fixed a bug when retrying to validate a token

#### Version 1.1.0

* Removed the "verbose" argument
* Added new command "properties" to list all properties that are defined for a tenant
* Added new command "model-meta" to retrieve the list of all metadata properties for a model
* Enhanced all commands that return model data to include the metadata properties for each model by default
* Added new command "upload-meta" to upload metadata for a model
* Refactored the handling of invalid/expired tokens
* Improved the logic when token is provided in the configuration

#### Version 1.0.7

* Removed special filtering logic when creating match-report
* Added model-meta command to read metadata for a specific model UUID
* Enhanced the model command to include the metadata in the response
* Optimized the match-report logic
* Added CSV output capability for all sub-commands
* Added thumbnails URLs in the CSV output for match-model and match-report
* Now using more human-readable tracing messages (including ANSI color)
* Implemented a workaround for situations where the assembly tree reports child models that no longer exist in the database
* Implemented local cache for model requests to improve performance of large data operations
* Updated the README.md

## Installation

It is a native app, which means that it is compiled to binary for each
target platform. You would need to download the correct version for your
operating system. Place the file in a directory of your choice on your system.
There is no need to run an installer application - it should be ready for use at this time.

### Pre-compiled binary

The PCLI application is a single executable binary. All that you need to do is run it in your terminal.

1. Go to the GitHub [repository](https://github.com/jchultarsky101/pcli) for this project
2. Under "Releases" click on the latest release
3. Download the ZIP file for your platform: "pc-windows-gnu" is for Windows; "x86_64-apple-darwin" is for Intel-based MacOS
4. Place the ZIP file in a directory of your choice
5. Unzip the ZIP file
6. If necessary, grant execution permissions to the executable file
7. Optionally, ad the path to your PATH environment variable, create shortcut, symbolic link, etc.

### Building from source

The application is written in Rust. You will need to (install Rust)[https://www.rust-lang.org/tools/install] on your computer in order to compile it. 

1. Clone the GitHub repository
2. Execute:
```
cargo build --release
```
3. This will produce the binary for your platform in your project's *target* directory

## A Word about Tenants

Physna is a multi-tenant system. As such, each client organization receives their own environment. 
For example, if your company is MyCompany, LLC, you may receive an environment and your
customer-facing web site may be at https://mycompany.physna.com. Here the identifier "mycompany" is
your tenant ID.

The tenant ID is a critical bit of information, which will be required from you almost every time you execute
the CLI. If you are unsure what your tenant ID is, please contact your Physna representative.

NOTE: If you specify a tenant that is unknown (not in the configuration file), the error message will list all currently configured tenants

## Configuration

The tool uses a single configuration file. The default location for this file is your home directory
and the default file name is ~/.pcli.conf. However, if you so desire, the tool does provide a command line
argument to allow you to specify alternative configuration at another location.

Here is an example contents of .pcli.conf:

~~~
base_path: https://api.physna.com
identity_provider_url: https://physna.okta.com/oauth2/default/v1/token
tenants:
    mytenant:
      client_id: 0000000000000000
      client_secret: 
~~~

In the example above, "mytenant" is your tenant ID as explained above. Make sure to 
edit this sample to reflect that correctly.

The only other piece of data you need to provide is the client_id value. This can be obtained
by your Physna administrator. They would need to login via the web UI, click on their account avatar
in the upper-right corner of the page, click on their username, and finally click on "PUBLIC API".
Both the client ID and the client secret will be shown there. The Physna admin is most likelly someone
in your company that is responsible for your tenant. If unsure, reach out to your Physna representative.

NOTE: In the new version of the Physna's web UI, click on "Manage" under your avatar.

In most cases, you should leave the client secret field unpopulated. Read further to understand how
it could be useful sometimes.

## Basic Use

This is a command line utility. You can use it within a terminal session.
To see what functions are supported, you can type "help" as the first argument.

For example:

```bash
$ pcli help
```
```
pcli 1.6.11
Julian Chultarsky <jchultarsky@physna.com>
CLI client utility to the Physna public API/V2

USAGE:
    pcli [OPTIONS] --tenant <tenant> <SUBCOMMAND>

OPTIONS:
    -c, --config <config>    Configuration file [default: /Users/julian/.pcli.conf]
        --color <color>      Adds color to the output (optional: e.g. 'black', 'red', 'green',
                             'yellow', 'blue', 'magenta', 'cyan', 'white')
    -f, --format <format>    Output data format (optional: e.g. 'json', 'csv', or 'tree') [default:
                             json]
    -h, --help               Print help information
    -p, --pretty             Produces pretty output (optional: default is 'false')
    -t, --tenant <tenant>    Your tenant ID (check with your Physna admin if not sure)
    -V, --version            Print version information

SUBCOMMANDS:
    assembly-bom                  Generates flat BoM of model IDs for model
    assembly-tree                 Reads the model's assembly tree
    classification-predictions    Read the list of classification predictions for an image by
                                      given classifier
    classifiers                   Lists all available image classifiers
    create-folder                 Creates a new folder
    create-image-classifier       Creates a new image classifier
    delete-model                  Deletes a specific model
    folders                       Lists all available folders
    help                          Print this message or the help of the given subcommand(s)
    invalidate                    Invalidates the current access token
    label-folder                  Labels models in a folder based on KNN algorithm and geometric
                                      match score as distance
    match-folder                  Matches all models in a folder to other models
    match-model                   Matches all models to the specified one
    match-report                  Generates a match report for the specified models
    model                         Reads data for a specific model
    model-meta                    Reads the metadata (properties) for a specific model
    models                        Lists all available models in a folder
    properties                    Lists all available metadata propertie names and their IDs
    reprocess                     Reprocesses a specific model
    status                        Generates a tenant's environment status summary
    sysinfo                       Prints details of the current host system
    token                         Obtains security access token from the provider
    upload                        Uploads a file to Physna
    upload-model-meta             Reads metadata from an input CSV file and uploads it for a
                                      model specified by UUID
```

The application supports sub-commands. To get more detailed help for a
specific sub-command, enter the command name after "help" or alternatively,
first enter the command and add"--help" after as shown in the example:

```bash
$ pcli help model
```

Produces the following output:

```
pcli-model 1.6.11
Reads data for a specific model

USAGE:
    pcli --tenant <tenant> model [OPTIONS] --uuid <uuid>

OPTIONS:
    -h, --help           Print help information
    -m, --meta           Enhance output with model's metadata
    -u, --uuid <uuid>    The model UUID
    -V, --version        Print version information
```

As you can see, here the "model" sub-command takes "--uuid" as an argument
to specify the UUID of the model we are trying to read. To read a specific model
in this case, we need to provide the following (just an example):

```bash
$ pcli --tenant="beta" model --uuid="82cb38ce-c3e4-4a07-b605-5177602a6d8f"
```

The command flags support both short and long names. For example, for tenant we can
specify the full name "--tenant" or the short name "-t". Those two are be equivalent. Not all arguments
have a short name. In those cases, the long name is the only option. We reserve those for cases where the argument is
rarely used or it is important to make sure we avoid mistakes.

### Order of Arguments

There are two types of arguments:

* General use
* Specific to a sub-command

When composing your command line, you need to provide the general arguments first. If using a sub-command, type that next and
follow that with any sub-command specific arguments. Both the general and specific arguments could be optional. 

This is the general idea:

```bash
$ pcli [general arguments] sub-command [command-specific arguments]
```

Here are few examples:

```bash
$ pcli help models
```

In this case, "help" is the subcommand. There are zero generic arguments we need to provide to it. We provide one argument "models" to identify the command
for which we need specific help.

That was a very simple example. Here is a more complicated one:

```bash
$ pcli --tenant="mycompany" --format="cvs" --pretty models --folder="1" --search="part_name"
```

In this case, "tenant", "format", and "pretty" are all generic arguments that may apply accross all sub-commands in a similar way. On another hand
"folder" and "search" are arguments that are specific to the "models" command. Of course "models" is the sub-command itself.

###### How do I know which arguments are generic and which are specific?

That is easy! If you display the help without specifying a command name, you will see the info about the generic arguments. If you do provide
a command name in the help request, you will see the command-specific arguments only.

Remember, provide the generic arguments (if any) first, then the command name followed with any specific command arguments.

###### Equal sign, quotes, oh mine!

There is some free play when it comes of how you provide values to an argument. Strictly speaking, those rules are not implemented by PCLI,
but the operating system you are using and the terminal program you are using. However, it may be useful to go over few things here.

For example these two ways of executing the "folders" command are both valid and equivalent:

```bash
$ pcli --tenant mycompany folders
```

and

```bash
$ pcli --tenant="mycompany" folders
```

In other words, you can provide either the equal character ('=') or a space as the separator between argument and its value. The recommended way is to use
the equal character (example #2) without any spaces between the '=' on either side.

If a value does not contain spaces or any other characters that my cause confusion, you can type it without surrounding it with quotes. The best practice is
to always wrap it in double quotes.

In general, it is considered O.K. to not surround numbers with double quotes. It is a bit more typing, but I recommend to be consistent and wrap all values
as a matter of good habit.

###### Arguments with multiple values

Few sub-commands can take multiple values for an argument. Such arguments are clearly indicated in the help. 
For example, in an example you will see further down again:

```bash
$ pcli --tenant="beta" --format="csv" --pretty match-folder --folder="4" --folder="6" --threshold="0.99"
```

This means that the "match-folder" command will search simultaneously in two folders (i.e. 4 and 5) and combine the output 
for both. This way, you can widen the search in one command.

###### Arguments without a value

The argument "pretty" does not take a value. This is because it is a boolean argument. The mere presence of it indicates
that you want to use it. In this case:

```bash
$ pcli --tenant="mycompany" --pretty folders
```

means that I would like to print a more human-readable JSON output than the default compact version meant for post-processing tasks. 
If I ommit "pretty" it is the same as setting a value of false for it, which is the default.

### Working with Tokens

It is important to understand how the authentication and authorization work.
Physna uses OpenID Connect provider and upon successful authentication will issue the user
an access token, which will be valid for the duration of the session (several hours). As long
as your token is valid, you do not need to authenticate every time you run the CLI utility.

The current token is stored by the CLI tool in a hidden file in your home directory. The file name 
is ".pcli.<tenant_id>.token". For example if your tenant is "beta", the file name for that
environment would be ~/.pcli.beta.token.

You can delete the token file at any time. If you do, the CLI tool will prompt you to authenticate again
and create a new one in its place. There is an easier way however. The CLI supports dedicated sub-commands
to deal with token generation:

#### Invalidating Your Token

The sub-command "invalidate" will delete the current token for a given tenant. It will leave
unmodified any other tokens to other tenants you may have.

```bash
$ pcli --tenant="beta" invalidate
```

This will invalidate your current token and delete the token file from your system.

#### Displaying Your Token

Your token data is encrypted. It does not reveal anything about yourself or your system.
In the vast majority of cases, you would not care what it is. However, there may be some
special use cases where knowing your token is handy. One such case is if you are trying to
access the Physna's API in other ways than via this CLI. For example, this may be with cURL,
Postman, or some other client. 

The following command will print the current token for your tenant (here is "beta") to the terminal:

```bash
$ pcli --tenant="beta" token
```

#### Best Practices for Handling Tokens

First of all, make sure your home directory is properly secured with the correct file permissions. 
It is by default on all popular OS-es, but this is your responsibility. If not secure, there is 
much to worry about, not just the access token by Physna.

If you are automating your operations via shell scripts and you plan to invoke the CLI multiple times,
it is always a good idea to start with a fresh session. In your BASH script, call the "invalidate" command
first and once. You can then iterate over a batch of command executions without concern that your session
may expire in the middle of your work. This is handy especially when you have unattended executions (e.g. triggered by a cron job, etc.).

Also in the case of unattended executions, you could enter your Client Secret in your configuration file. This is
a less secure option, because it will not prompt you to enter it in your terminal every time you authenticate,
but if there is no human to enter it, it is difficult to automate. If you choose this path, make sure that 
your configuration file has the proper file permission to secure it against other people that may share your
computer.

### Listing Available Folders

The command "folders" will print the full list of folders currently available for your tenant.
Physna organizes data into logical storage units named folders. This is not unlike many other
systems you are familiar with. Each folder has a name, but most importantly, it has a numeric identifier.
The folder ID is optional in some of the sub-commands you may want to use, but it is always a good idea
to provide it.

```bash
$ pcli --tenant="mycompany" folders
```
```
[{"id":1,"name":"Default Container"},{"id":2,"name":"Crawler"},{"id":3,"name":"myfolder"}]
```

The output of this command by default is formatted as compact JSON. This to allow you to chain the output
of this utility with other commands you may have after.

To make it a bit more human-readable, you can use the option "--pretty". This will pretty-print, or format
the output with some structure to make it easy to view.

```bash
$ pcli --tenant="mycompany" --pretty folders
```
```
[
  {
    "id": 1,
    "name": "Default Container"
  },
  {
    "id": 2,
    "name": "Crawler"
  },
  {
    "id": 3,
    "name": "myfolder"
  }
```

You can add the option "--color" to make the output colorful if you wish:

```bash
$ pcli --tenant="mycompany" --pretty --color="green" folders
```

Some commands support other types of format. For example, you can receive the same information as CSV for
parsing later:

```bash
$ pcli --tenant="mycompany" --format="csv" folders
```

The output is:

```
1,Default Container
2,Crawler
3,myfolder
```

The default output format is "json". The available options are "json", "csv", "tree".

Adding "--pretty" in this case will add header row to the CSV output containing the column names.

### Listing Available Models

To obtain a list of models in your tenant environment, use the "models" sub-command. Please, note that there is also "model" (singular) command, which is used for querying a single model. The "models" command takes a mandatory argument "--folder", which is the folder ID of interest and limits the search.

Example:

```bash
pcli help models
```

```
pcli-models 1.6.11
Lists all available models in a folder

USAGE:
    pcli --tenant <tenant> models [OPTIONS] --folder <folder>

OPTIONS:
    -d, --folder <folder>    Folder ID (e.g. --folder=1)
    -h, --help               Print help information
    -m, --meta               Enhance output with model's metadata
    -s, --search <search>    Search clause to further filter output (optional: e.g. a model name)
    -V, --version            Print version information
```

Example for listing all available models in folder 1 (the Default folder):

```bash
$ pcli --tenant="delta" models --folder="1"
```

The output from the above will include the list of models in folder with ID of 1.

You can further filter the output of the "models" command by specifying an optional search term. For example, to list
all models in folder 3 with model name containing the string "mypart", you can execute the following:

```bash
$ pcli --tenant="delta" models --folder="3" --search="mypart"
```

As with the "folder" command, you can specify CSV as the output format, use "--pretty" and colors.

The --meta flag is optional. When provided, it will also query for and add the metadata to the output.

### Querying for a Specific Model

The "model" command takes as mandatory argument the unique identifier (the UUID) of the model we are interested in. This is done via the "--uuid"
argument. The CLI will query your tenant for that specific model regardless which folder it belongs to.

```
pcli-model 1.6.11
Reads data for a specific model

USAGE:
    pcli --tenant <tenant> model [OPTIONS] --uuid <uuid>

OPTIONS:
    -h, --help           Print help information
    -m, --meta           Enhance output with model's metadata
    -u, --uuid <uuid>    The model UUID
    -V, --version        Print version information
```

* uuid - is the models UUID
* meta - is an optional flag. When provided, it will query and add the metadata of the model to the output
    
```bash
pcli --tenant="delta" model --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx"
```

As explained before you can use different output formats, pretty-print, color.

### Uploading a Model

The "upload" command assists you with uploading new 3D models to Physna. It takes the following arguments:

```bash
$ pcli help upload

```
```
pcli-upload 1.6.9
Uploads a file to Physna

USAGE:
    pcli --tenant <tenant> upload [OPTIONS] --folder <folder> --input <input> --units <units>

OPTIONS:
    -b, --batch <batch>        Batch UUID (Optional, if not provided new one will be generated)
    -d, --folder <folder>      Folder ID (e.g. --folder=1)
    -h, --help                 Print help information
    -i, --input <input>        Path to the input file
    -m, --meta <meta>          Input CSV file name containing additional metadata associated with
                               this model
        --source <source>      Specifies the Source ID to be used
        --timeout <timeout>    When validating, specifies the timeout in seconds
        --units <units>        The unit of measure for the model (e.g. 'inch', 'mm', etc.)
    -V, --version              Print version information
        --validate             Blocks until the model is in its final state
```

* "input" is the path to the file you would like to upload in your local file system
* "folder" is the Physna folder ID that will be the destination for your upload
* "units" is the unit of measure for your 3D model. It is a string. For example "mm"
* "batch" is a UUID that represents a transaction. When uploading a group of logically related models (e.g. an assembly with all of its parts), you will need to provide UUID type 4 as the transaction ID to instruct Physna that all of these files are related. If not provided, each file will be considered independent from any other and a batch UUID will be generated automatically by the client for each file
* "validate" is an optional argument that will cause the process to wait until the file upload completes. It will retrieve the model back and check the status. If the status is one of the final states, it returns the model data. If it is still pending, it will continue to wait. If no timeout is provided, it could wait forever or until error occurs.
* "timeout" only applies when "validate" is present. It specifies the maximum wait time allowed. The value is in seconds. Use that argument together with "validate" to limit the time an operation can take.
* "source" is an optional string provided by the user that represents an unique identifier for the source system. It could be helpful to link a model in Physna to an entry in a PLM system or some other database. If not specified, the original file name will be used as the default value.
* "meta" is optional. It is the file path to additional file containing metadata associated with this model. Once the model is uploaded, we will attempt to upload the metadata for it as well.
* "validate" is optional. Normally, the upload operation is asynchronous. This means that as soon as the content of the file is uploaded to Physna, the control will be returned back to the user. However, Physna will continue to process the model in order to index it in the background. That takes additional time. In some cases, you may want to block and wait until the model is fully ready for use before proceeding to the next operation. In such cases, you can add this flag.

Here is an example of how all this comes together:

```bash
$ pcli --tenant="mycompany" upload --folder="5" --input="/path/to/my/file" --units="mm"
```

If successful, we will upload the model in the file named "file".

Example of uploading a model and waiting until it is ready for use (or error):

```bash
$ pcli --tenant="mycompany" upload --folder="5" --input="/path/to/my/file" --units="mm" --validate --timeout=60
```

This operation will block until the model is fully indexed or 60 seconds have elapsed.

You can upload multiple files in one operation. To do that, you can specify the input path to contain a wild card or simply be a directory path. If wild card is used, you need to surround the path with double quotes.

**NOTE:** Be careful with uploading all files in a directory. It may contain files that are not 3D models. 

For example:

```bash
$ pcli --tenant="mycompany" upload --folder="5" --input="/path/*.stl" --units="mm"
```

The above command will upload all files with STL extension in directory "/path".

### Reprocessing a Model

The "reprocess" command is useful to recover from situations when a model has been uploaded, but for some reason its indexing
in Physna has not completed normally. It only takes a single mandatory parameter: the UUID of the model we want to reprocess.

NOTE: You can combine this operation with a loop in your own script to reprocess any models that show status other than "finished" in a specific folder if you wish.

```bash
$ pcli help reprocess
```

Example:

```bash
pcli --tenant="delta" reprocess --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx"
```

This will cause the status of the model to be reset to "reprocessing" and the model will progress through the normal steps of processing and indexing as when uploading a new file.

The command produces no output.

### Deleting a model

This command will delete a model and all related metadata from the Physna database. Please, be careful when using this function.

Example:

```bash
pcli --tenant="delta" delete-model --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx"
```

### Reading Metadata

In addition to the 3D geometry data, additional metadata can be associated with the model. The metadata is in the form of name/value pairs. Both the name and the value are UTF-8 strings. The metadata is returned as part of the model data when using the commands "model" or "models". However, PCLI offers an additional specialized command to only retrieve the metadata and not the rest of the model data. This is useful when scripting more sophisticated solutions. 

The command is:

```
pcli-model-meta 1.6.9
Reads the metadata (properties) for a specific model

USAGE:
    pcli --tenant <tenant> model-meta --uuid <uuid>

OPTIONS:
    -h, --help           Print help information
    -u, --uuid <uuid>    The model UUID
    -V, --version        Print version information
```

It takes one mandatory argument - the model's UUID. The output by default is JSON, but when we specify CSV, the output contains the following columns:

* MODEL_UUID - the UUID of the model
* NAME - the property name
* VALUE - the property value

**NOTE:** The property names are case insensitive and are capitalized.

Here is an example:

```
pcli --tenant=mycompany --format=csv --pretty model-meta --uuid=97377547-9062-4149-90f7-16daf400148a
MODEL_UUID,NAME,VALUE
97377547-9063-4149-90f7-16daf400148a,DESCRIPTION,Test description
97377547-9063-4149-90f7-16daf400148a,SKU,Test
```

In this example, the model has two properties ("DESCRIPTION" and "SKU") with their corresponding names.

The reason for the UUID of the model to be included as the first column is simple. You can concatenate the output of many executions of this command into one single file. That larger file will contain metadata for many models. You will see how that becomes helpful in the next section.

### Uploading Metadata

In some cases, we need to associate additional metadata with the geometry of a model. The command "upload-model-meta" serves this purpose.

```bash
pcli help upload-model-meta
```
```
pcli-upload-model-meta 1.6.9
Reads metadata from an input CSV file and uploads it. 

USAGE:
    pcli upload-model-meta --input <input>

OPTIONS:
    -h, --help             Print help information
    -i, --input <input>    Path to the input file
    -V, --version          Print version information
```

The file format is the same as the output produced by the command "model-meta". It is the same format used by the 'upload' command earlier. The columns are: MODEL_UUID,NAME,VALUE. One use case is to first read the metadata for some models, edit it externally (for example, with a text editor). This may include modifying values for existing properties or adding new properties and their values.

The required argument is "input" - the name of the CSV formatted input file.

If a property with this name already exists for the model, its value will be overridden with the new value provided. If the property does not exist, a new property with the provided (but capitalized) name will be created.

**NOTE:** If the metadata property value is an empty string, this command will delete the property for the model. In other words, if you want to delete a property, upload the same with value of an empty string in the input CSV file.

### Reading the Assembly Structure

The command "assembly-tree" will query for a specific model and return as result the assembly structure.
Obviously, this is mostly useful when working with assemblies. The assembly tree could be recursive with 
assemblies having sub-assemblies, and so forth.

The "assembly-tree" command supports the unique output format of "tree".

### Matching Models to Other Models

Physna's core expertise is in finding geometric matches for models. The sub-command "--match-model" does
part-to-part match. This means that a model is matched as a unit to all other models in the system. 
With this function, we do not cascade from top-level assembly into all of its sub-assemblies, nor we 
try to determine if this model may be a component of another assembly. 

```
pcli-match-model 1.6.9
Matches all models to the specified one

USAGE:
    pcli --tenant <tenant> match-model [OPTIONS] --uuid <uuid> --threshold <threshold>

OPTIONS:
        --classification <classification>    The name for the classification metadata property
    -h, --help                               Print help information
    -m, --meta                               Enhance output with model's metadata
    -t, --threshold <threshold>              Match threshold percentage (e.g. '96.5')
        --tag <tag>                          The value for the classification metadata property
    -u, --uuid <uuid>                        The model UUID
    -V, --version                            Print version information
```

* "uuid" is the UUID of the model we are trying to match.
* "threshold" is the match level. This is a value between [0..1]. For example, 80% match would be 0.8.
* "meta" is an optional flag. When specified, we will query for additional metadata and if present we will add that to the output.
* "classification" is an optional argument and requires that he "meta" is present. It is the name of a metadata property that will be set for each matching model. This way the user can permanently tag models.
* "tag" requirest that the classification argument is present. It is the value to be associated with the "classification" property.

Example:

```bash
pcli --tenant="delta" match-model --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx" --threshold="97.5"
```

The output contains the list of models that matched the criteria and a value between zero and one indicating the fit. A value of 1 means 100% match.

As with the other commands, you can choose to output in JSON or CSV format.

**NOTE:** If you are using pipes to send the output to another process, please make sure you have obtained your
token correctly prior to executing the operation. If not, the process will stop and wait for you to enter the client secret on the command line
and your output will not be valid.

The --classification argument requires further clarification.
It is an optional argument, but if it is to be used, the --meta argument must be present. 
The purpose of this is to allow the user to permanently "tag" models that match the source model with a metadata name/value pair. In other words, for each model that 
matches, we will create a new medata property with name provided as --classification and value provided as --tag. 
If --classification is not provided, no new metadata will be created. 
This is useful in cases where we want to mark models that have similar geometry as some arbitrary class. Later, you can use this when you search for models and provide the value as the --search argument.
You can also use the metadata to automate the ML learning for data classification.

### Matching Entire Folders of Models

Sometimes, we need to execute match models in bulk. With the commands already provided so far, you could create a driver script to
achieve the effect, but we provide a convenience method for this purpose. In other words, this command will query for the list of models in your folder and for each it will execute "match-model". It will combine the responses into a single output. The input arguments are the same as the previous command.

```
pcli-match-folder 1.6.9
Matches all models in a folder to other models

USAGE:
    pcli --tenant <tenant> match-folder [OPTIONS] --threshold <threshold> --folder <folder>

OPTIONS:
    -d, --folder <folder>          Folder ID (e.g. --folder=1)
    -e, --exclusive                If specified, the output will include only models that belong to
                                   the input folder
    -h, --help                     Print help information
    -m, --meta                     Enhance output with model's metadata
    -s, --search <search>          Search clause to further filter output (optional: e.g. a model
                                   name)
    -t, --threshold <threshold>    Match threshold percentage (e.g. '96.5'
    -V, --version                  Print version information
```

Example:

The following command will execute individual matches for all models found in the folder with ID of 4 (--folder=4) at match threshold of 99% (--threshold=0.99).
It will output the result in CSV format (--format=csv) and add header line with column names to it (--pretty).

```bash
$ pcli --tenant="beta" --format="csv" --pretty match-folder --folder="4" --threshold="0.99"
```

You can also specify a search term to further narrow down the filter. Finally, the "--meta" flag will cause any associated metadata to be added to the output.

### Generating Match Report

The "match-report" command combines multiple operations. It is used to generate comprehensive match report that
could be used as input for further post processing. For example, machine learning algorithms. It produces multiple
outputs and therefore it requires the user to specify file names for each output.

```bash
$ pcli --tenant="whirlpool" match-report \
--duplicates="./my_duplicates.csv" \
--graph="./my_graph.graphviz" \
--dictionary="./my_dictionary.json" \
--threshold="0.95" \
--uuid="<my_master_assembly_uuid>"
```

* "duplicates" is an output file name. A new file will be created (overridden if it exists) with a CSV content listing each model match and its percentage. The percentage is not lower than the threshold specified
* "graph" is an output file. It will contain special GraphViz format that represents the graph of the master assembly. It can be converted to an image and viewed, or used as input to the next process
* "dictionary" is an output file in JSON format. It will map the UUIDs for each model in Physna to the graph node IDs
* "threshold" is the minimum match level
* "uuid" is the UUID for the master assembly in Physna

Hint: You can find the UUID for any model by name by using the "models" command and a search clause.

Hint: You can install and use the [Graphviz CLI](https://graphviz.org/doc/info/command.html) to convert the graphviz format to an image that you can view. 
You will have to install that utility separately. For example, to convert the file we created earlier:

```bash
$ cat ./my_graph.graphviz | dot -Tsvg > my_graph.svg
```

This will produce a SVG file, which you can view by opening it in your browser or another graphics viewer.

### Tenant Environment Status

We provide a convenience command to check on the status of folders in your environment.

The following command would output details about the number of models in the specified folder
per type of file and status.

```bash
$ pcli --tenant="mytenant" --format="csv" --pretty status --folder="1"
```

### Classifiers

In addition to geometric search and comparison functions, Physna offers AI technology capable of automatic classification of images into 3D models. If you have uploaded sufficient number of models in your environment and have created and trained an image classifier, you could upload a 2D image of a part (e.g. digital picture taken with your phone) and Physna can recognize the object and retrieve the 3D model that corresponds to it.

At this time, the training of new classifiers within Physna is only available via the web UI since user input is required during the training process. The PCLI offers commands to query the classifier(s) and retrieve a list of predictions from the AI.

The ML algorithm used in this case is [Logistic Reressoin](https://en.wikipedia.org/wiki/Logistic_regression).

#### List the available classifiers

To obtain the list of currently available classifiers in your environment, use the "classifiers" command. It takes no arguments:

```
pcli --tenant=mycompany --format=csv --pretty classifiers
ID,NAME,STATUS
8db6569f-65e6-45ca-8fc2-3603c9967437,MyClassifier,deployed
```

You will need the classifier UUID in order to be able to execute the next command.

#### Classification predictions

As I mentioned above, you can take a 2D picture of a part and upload that image as input. Physna's AI will analyze the image and produce a list of predictions to what 3D models this image belongs. The command to do this is:

```
pcli help classification-predictions
pcli-classification-predictions 1.6.8
Read the list of classification predictions for an image by given classifier

USAGE:
    pcli --tenant <tenant> classification-predictions --uuid <uuid> --input <input>

OPTIONS:
    -h, --help             Print help information
    -i, --input <input>    Path to the input file
    -u, --uuid <uuid>      Classifier UUID
    -V, --version          Print version information
```

The required arguments are:

* "uuid" - the UUID of the classifier to use. See the previous section for details on how to obtain that UUID
* "input" - the path to the 2D file in your local file system

Here is an example:

```
pcli --tenant=delta --format=csv --pretty classification-predictions --uuid=1d896c02-9fba-4807-b9ec-07c96a7db946 --input=./mypart.png
ID,NAME,STATUS
96a2bb55-dcef-4a05-a43f-82bf696f872d,part1,0.9844062328338624
9575ad2b-0548-4b48-9cec-7ed183bff064,part2,0.0000015775714246046846
```

In this case, we have one model with score of 0.98, which is very close match.

### Model Labeling

While the classification altorithm we described in the previous section is a component of Physna's backend logic, 
the PCLI client also provides its own (much simpler) mechanism for label propagation. 
This is implemented in the *label-folder" command. 
It is based entirely on geometric match scores provided by Physna. 

The user provides 3 mandatory and one optional input arguments:

* "folder" - the target folder identifier in your tenant
* "classification" - the name of a metadata property that will be used for classification
* "threshold" - the confidence threshold value
* "exclusive" - this is a flag and does not take a value. If present, only the models found in the source fodler will be considered for matching. The default is to consider all models in the tenenat regardless of their parent folder.

When executed, PLCI will read the contents of the folder and for each of the models in it, 
it will perform part-to-part match as in the "match-model" command.
The match will be done with the specified threshold. 
It will then rank the matches by their scores and starting from the highest to the lowest will check if
the matching models contain a value for the metadata property specified. 
If they do, the model will also be assigned that same metadata property and value.

The assumption is that if model "A" has metadata property of "classification" with value of "apple" and model "B" is 98.5% geometrically the same as model "A", we can say with
98.5% confidence that model "B" is also an "apple". 
We indicate that by setting model "A"'s metadata property "classification" to have value of "apple"

```
pcli-label-folder 1.6.10
Labels models in a folder based on KNN algorithm and geometric match score as distance

USAGE:
    pcli --tenant <tenant> label-folder [OPTIONS] --folder <folder> --threshold <threshold> --classification <classification>

OPTIONS:
    -c, --classification <classification>
            The name for the classification metadata property

    -d, --folder <folder>
            Folder ID (e.g. --folder=1)

    -e, --exclusive
            If specified, the output will include only models that belong to the input folder

    -h, --help
            Print help information

    -s, --search <search>
            Search clause to further filter output (optional: e.g. a model name)

    -t, --threshold <threshold>
            Match threshold percentage (e.g. '96.5')

    -V, --version
            Print version information
```

Example:

```bash
pcli --tenant=mycompany label-folder --folder=1 --threshold=0.9 --classification="classification"
```

The optional '--search' argument may be used to further refine the target list of models. Only models that match the search
criteria will be labeled and all others ignored. The --search option works the same as for the "models" command.

The command does not have any visible output, except returning success or error status. Once completed, your models should be labeled accordingly.

**NOTE:** Because the logic depeneds on at least some models being labeled apriori and because the command implements a single pass through the folder
you may need to run this command multiple times for best results.

**NOTE:** If a model does not have any geometric matches that contain the required metadata property, its own property by that name will be deleted. 
Be careful what property name you use for classification. On another hand, you can run this command by specifying different property names if so desired.

Although the user may target models in specific folder for labeling, PCLI will match any model in any folder
unless the user adds the --exclusive command line argument. It that is specified, only the model in the target
folder will be used for matching. 
In other words, the label values may come from any folder in your tenant unless you specify --exclusive.

For the initial labeling of models, you can use the "upload-model-meta" command.

## Advanced Use

The real power of this CLI tool comes when you use it in conjunction with other tools. For example,
you can filter down the list of models further by piping the output (formatted as JSON) to [JQ](https://stedolan.github.io/jq/):

```bash
$ pcli -t="beta" models --folders="1" | jq '.[] | select(.id=="96049555-b55a-45b1-bdcb-2555cb0012fe")'
```

JQ has many useful features that could help you manipulate the output as needed.

You can pipe the output to a file on your disk for post-processing of the output:

```bash
$ pcli -t="beta" --format="csv" models --folders="5" > myfile.csv
```

Be aware that "--pretty" adds more to the output. For example, if your output format is CSV, it will add
a header record. If your post-processor counts the number of records in the CSV to tally the number of
models found (as example), you will have to ignore the first record. In this case it is probably better 
not to include the "--pretty" flag. This argument is binary and does not take a value. If it is present, it
means that it is active; if not, it is effectively set to false.

## Support

If you have any questions, please e-mail to [jchultarsky@physna.com](mailto:jchultarsky@physna.com).