# Physna CLI

This utility is a CLI client for the Physna's public API V2 (and in part API V1). It could be used to interract with the system
in automated ways.

## Physna Account

To be able to use this client, you will need to first request a Physna Enterprise account. Please, contact your Physna representative for details.

## Change Log

The latest version is 1.5.0

### Version 1.5.0

* Added create-folder command

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

* Removed special filterning logic when creating match-report
* Added model-meta command to read metadata for a specific model UUID
* Enhanced the model command to include the metadata in the response
* Optimized the match-report logic
* Added CSV output capability for all sub-commands
* Added thumbnails URLs in the CSV output for match-model and match-report
* Now using more human-readable tracing messages (including ANSI color)
* Implemented a workaround for situations where the assebly tree reports child models that no longer exist in the database
* Implemented local cache for model requests to improve performance of large data operations
* Updated the README.md

## Installation

It is a native app, which means that it is compiled to binary for each
target platform. You would need to download the correct version for your
operating system. Place the file in a directory of your choice on your system.
There is no need to run an installer application - it should be ready for use at this time.

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
pcli 1.3.0
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
    assembly-bom         Generates flat BoM of model IDs for model
    assembly-tree        Reads the model's assebly tree
    folders              Lists all available folders
    help                 Print this message or the help of the given subcommand(s)
    invalidate           Invalidates the current access token
    match-folder         Matches all models in a folder(s) to all other models
    match-model          Matches all models to the specified one
    match-report         Generates a match report for the specified models
    model                Reads data for a specific model
    model-meta           Reads the metadata (properties) for a specific model
    models               Lists all available models
    properties           Lists all available metadata propertie names and their IDs
    reprocess            Reprocesses a specific model
    status               Generates a tenant's environment status summary
    sysinfo              Prints details of the current host system
    token                Obtains security access token from the provider
    upload               Uploads a file to Physna
    upload-model-meta    Reads metadata from an input CSV file and uploads it for a model
                         specified by UUID
```

The application supports sub-commands. To get more detailed help for a
specific sub-command, enter the command name after "help" or alternatively,
first enter the command and add"--help" after as shown in the example:

```bash
$ pcli help model
```
```
pcli-model 1.3.0
Reads data for a specific model

USAGE:
    pcli model --uuid <uuid>

OPTIONS:
    -h, --help           Print help information
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
Physna uses OpenID Connect provider and uplon successful authentication will issue the user
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
first and once. You can then iterrate over a batch of command executions without concern that your session
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
```
1,Default Container
2,Crawler
3,myfolder
```

The default output format is "json". The available options are "json", "csv", "tree".

Adding "--pretty" in this case will add header row to the CSV output containing the column names.

### Listing Available Models

To obtain a list of models in your tenant environment, use the "models" sub-command. Please, note that there

is also "model" (singular) command, which is used for querying specific model. The "models" command takes
an optional argument "--folders", which is a list of folder IDs and restricts which folders are being queried. It is a best practice to always supply the folder ID(s) you are targetting to avoid querying too much data by accident. 

Example:

There is also "model" (singular) command, which is used for querying a single specific model. The "model*s*" command takes
an optional argument "--folder", which specifies the folder ID and restricts which folders are being queried. It is a best practice to always supply the folder ID(s) you are targetting to avoid querying too much data by accident.

You can list multiple folder IDs if you would like to query multiple folders at the same time. The help screen will show how to use it:

```bash
pcli help models
```
```
pcli-models 1.3.0
Lists all available models

USAGE:
    pcli models [OPTIONS] --folder <folder>

OPTIONS:
    -d, --folder <folder>    Folder ID (you can provide multiple, e.g. --folder=1 --folder=2)
    -h, --help               Print help information
    -s, --search <search>    Search clause to further filter output (optional: e.g. a model name)
    -V, --version            Print version information
```

Example for listing all available models in folders 1 and 2:

```bash
$ pcli --tenant="delta" models --folder="1" --folder="2"
```

The output from the above will include the combined list of models in both folders with ID of 1 and folder with ID of 2.

You can further filter the output of the "models" command by specifying a search term. For example, to list
all models in folder 3 whith model name containing the string "mypart", you can execute the following:

```bash
$ pcli --tenant="delta" models --folder="3" --search="mypart"
```

As with the "folder" command, you can specify CSV as the output format, use "--pretty" and colors.

### Querying for a Specific Model

The "model" command takes a unique identifier for the model we are interested in. This is done via the "--uuid"
argument. The CLI will query your tenant for that specific model regardless which folder it belongs to.

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
pcli help upload
pcli-upload 1.4.2
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
* "meta" is an optional input file containing additional metadata related to the model. The format is NAME,VALUE (both text). See "Uploading Metadata" section below, the behaviour here is identical to it
* "folder" is the Physna folder ID that will be the destination for your upload
* "units" is the unit of measure for your 3D model. It is a string. For example "mm"
* "batch" is a UUID that represents a transaction. When uploading a group of logically related models (e.g. an assembly with all of its parts), you will need to provide UUID type 4 as the transaction ID to instruct Physna that all of these files are related. If not provided, each file will be considered independent from any other and a batch UUID will be generated automatically by the client for each file
* "validate" is an optional argument that will cause the process to wait until the file upload completes. It will retrieve the model back and check the status. If the status is one of the final states, it returns the model data. If it is still pending, it will continue to wait. If no timeout is provided, it could wait forever or until error occurs.
* "timeout" only applies when "validate" is present. It specifies the maximum wait time allowed. The value is in seconds. Use that argument together with "validate" to limit the time an operation can take.
* "source" is an optional string provided by the user that represents an unique identifier for the source system. It could be helpful to link a model in Physna to an entry in a PLM system or some other database. If not specified, the original file name will be used as the default value.

Here is an example of how all this comes together:

```bash
$ pcli --tenant="mycompany" upload --folder="5" --input="/path/to/my/file" --meta="/path/to/my/metadata.csv" --units="mm"
```

If successful, we will upload the model in "file" and once that is completed, we will automatically also upload the optional metadata for it found in "metadata.csv" file.

Uploading of metadata here is optional and as convenience feature since often both are uploaded at the same time. If the "meta" argiment is omitted, only the geometry will be uploaded.

Example of uploading a model and waiting until it is ready for use (or error):

```bash
$ pcli --tenant="mycompany" upload --folder="5" --input="/path/to/my/file" --units="mm" --validate --timeout=60
```

This operation will block until the model is fully indexed or 60 seconds have elapsed.

You can upload multiple files in one operation. To do that, you can specify the input path to contain a wild card or simply be a directory path. If wild card is used, you need to surround the path with dowble quotes.

**NOTE:** Be careful with uploading all files in a directory. It may contain files that are not 3D models. 

For example:

```bash
$ pcli --tenant="mycompany" upload --folder="5" --input="/path/*.stl" --units="mm"
```

The above command will upload all files with STL extension in direcotry "/path".

### Reprocessing a Model

The "reprocess" command is useful to recover from situations when a model has been uploaded, but for some reason its indexing
in Physna has not completed normally. It only takes a single mandatary parameter: the UUID of the model we want to reprocess.

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

### Uploading Metadata

In some cases, we need to associate additional metadata with the geometry of a model. The command "upload-model-meta" serves this purpose.

```bash
pcli help upload-model-meta
```
```
pcli-upload-model-meta 1.3.0
Reads metadata from an input CSV file and uploads it for a model specified by UUID

USAGE:
    pcli upload-model-meta --uuid <uuid> --input <input>

OPTIONS:
    -h, --help             Print help information
    -i, --input <input>    Path to the input file
    -u, --uuid <uuid>      The model UUID
    -V, --version          Print version information
```

The required arguments are:

* "uuid" - the model's UUID
* "input" - CSV formatted input file. It must contain a header record with the following column names: "NAME","VALUE"

Here is an example of a metadata file:

```
NAME,VALUE
MyFirstTagName,Value1
MySecondTagName,Value2
```

If the operation is successful, your model will show two new properties with the provided names ("MyFirstTagName" and "MySecondTagName") with their corresponding values.

If a property with this name already exists for the model, its value will be overriden with the new value provided. If the property does not exist, a new property with the provided name will be created.

This command does not delete any existing properties. Deletion would need to be performed manually via the web UI.

### Reading the Assembly Structure

The command "assembly-tree" will query for a specific model and return as result the assembly structure.
Obviously, this is mostly useful when working with assemblies. The assembly tree could be recursive with 
assemblies having sub-assemblies, and so forth.

The "assembly-tree" command supports the unique output format of "tree".

### Matching Models to Other Models

Physna's core expertise is in finding geometric matches for models. The sub-command "--match_model" does
part-to-part match. This means that a model is matched as a unit to all other models in the system. 
With this function, we do not cascate from top-level assembly into all of its sub-assemblies, nor we 
try to determine if this model may be a component of another assembly. 

```bash
pcli --tenant="delta" match-model --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx" --threshold="97.5"
```

The UUID is the model's identifier. The "--threshold" is the minimum percentage to match.

The output contains the list of models that matched the criteria and a value between zero and one
indicating the fit. A value of 1 means 100% match.

As with the other commands, you can choose to output in JSON or CSV format.

To output a graph representation of your assembly tree:

*NOTE:* If you are using pipes to send the output to another process, please make sure you have obtained your
token correctly prior to executing the operation. If not, the process will stop and wait for you to enter the client secret on the command line
and your output will not be valid.

### Matching Entire Folders of Models

Sometimes, we need to execute match models in bulk. With the commands already provided so far, you could create a driver script to
achieve the effect, but we provide a convenience method for this pupose.

The following command will execute individual matches for all models found in the folder with ID of 4 (--folder=4) at match threshold of 99% (--threshold=0.99).
It will output the result in CSV format (--format=csv) and add header line with column names to it (--pretty).

```bash
$ pcli --tenant="beta" --format="csv" --pretty match-folder --folder="4" --threshold="0.99"
```

You can specify multiple folders here. For example:

```bash
$ pcli --tenant="beta" --format="csv" --pretty match-folder --folder="4" --folder="6" --threshold="0.99"
```

You can also specify a search term to further narrow down the filter.

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
means that it is active; if not, it is effectivelly set to false.

## Support

If you have any questions, please e-mail to [jchultarsky@physna.com](mailto:jchultarsky@physna.com).
