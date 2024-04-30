<a name="readme-top"></a>

<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]


<!-- PROJECT LOGO -->
<br />
<div align="center">

<!--
  <a href="https://github.com/jchultarsky101/pcli">
    <img src="images/logo.webb" alt="Logo" width="240" height="240">
  </a>
-->

  <h3 align="center">PCLI</h3>

  <p align="center">
    Command Line Interface client for the Physna API
    <br />
    <a href="https://jchultarsky101.github.io/pcli/"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/jchultarsky101/pcli/tree/main/images">View Demo</a>
    ·
    <a href="https://github.com/jchultarsky101/pcli/issues">Report Bug</a>
    ·
    <a href="https://github.com/jchultarsky101/pcli/issues">Request Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#about-the-project">About the project</a></li>
    <li><a href="#built-with">Built with</a></li>
    <li>
      <a href="#getting-started">Getting started</a>
      <ol>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li>
          <a href="#installation">Installation</a>
          <ol>
            <li><a href="#installers">Installers</a></li>
            <li><a href="#downloading-binary">Downloading a pre-compiled binary</a></li>
            <li><a href="#compilation">Compilation from source</a></li>
          </ol>
        </li>
        <li><a href="#about-tenants">About tenants</a></li>
      </ol>
    </li>
    <li><a href="#configuration">Configurations</a></li>
    <li>
      <a href="#basic-use">Basic use</a>
      <ol>
        <li>
          <a href="#command-line-arguments">Command line arguments</a>
          <ol>
            <li><a href="#order-of-arguments">Order of arguments</a></li>
            <li><a href="#general-vs-specific-args">How do I know which arguments are general and which are specific</a></li>
            <li><a href="#signs-n-quotes">Equal sign, quotes, oh mine</a></li>
            <li><a href="#milti-value-args">Arguments with multiple values</a></li>
          </ol>
        </li>
        <li>
          <a href="#tokens">Working with tokens</a>
          <ol>
            <li><a href="#invalidate-token">Invalidating your token</a></li>
            <li><a href="#print-token">Displaying your token</a></li>
            <li><a href="#token-best-practices">Best practices for handling tokens</a></li>
          </ol>
        </li>
        <li><a href="#list-folders">Listing folders</a></li>
        <li><a href="#list-models">Listing models</a></li>
        <li><a href="#query-model">Querying for a specific model</a></li>
        <li><a href="#upload-model">Uploading a model</a></li>
        <li><a href="#upload-many-models">Uploading multiple models in one step</a></li>
        <li><a href="#download-model">Downloading model file</a></li>
        <li><a href="#reprocess-model">Reprocessing a model</a></li>
        <li><a href="#delete-model">Delete a model</a></li>
        <li><a href="#read-meta">Reading metadata</a></li>
        <li><a href="#upload-meta">Uploading metadata</a></li>
        <li><a href="#read-asm">Reading the assembly structure</a></li>
        <li><a href="#match-model">Matching models to other models</a></li>
        <li><a href="#match-folders">Matching entire folders of models</a></li>
        <li><a href="#match-scan">Matching scanned model</a></li>
        <li><a href="#match-report">Generating a match report</a></li>
        <li><a href="#environment-status">Tenant environment status</a></li>
        <li><a href="#2D-to-3D">Searching for 3D models by 2D image</a></li>
        <li><a href="#model-label">Model labeling</a></li>
      </ol>
    </li>
    <li><a href="#errors">Handling errors</a></li>
    <li>
      <a href="#advanced-use">Advanced use</a>
      <ol>
        <li><a href="#pipes">Using pipes</a></li>
        <li><a href="#nushell">Using NuShell</a></li>
      </ol>
    </li>
    <li><a href="#support">Support</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>

<p/>

# <a id="about-the-project"></a>About the project

This utility is a CLI client for the Physna's public API V2. It could be used to interact with the system
in automated ways.

:warning: **Disclaimer**: This software is an open-source project and is not an officially supported product of Physna, Inc. Its primary purpose is to serve as a reference implementation and provide examples for utilizing the APIs. It has not been assessed for SOC2 compliance. Please make sure to read the license.

# <a id="built-with"></a>Built with

This project is built with the wonderful programming language [Rust](https://www.rust-lang.org).

[![Rust][Rust-logo]][Rust-url]

<!-- GETTING STARTED -->
# <a id="getting-started"></a>Getting started

## <a id="prerequisites"></a>Prerequisites

This is a command line interface (CLI) program that runs in the terminal. You need to have some familiarity of how to execute commands. 

It is a client to the Physna services. To be able to use it, you will need to first request a Physna Enterprise account. Please, contact your Physna representative for details.

## <a id="installation"></a>Installation

### <a id="installers"></a>Installers

You can use the installation script for your platform as shown on the [documentation site](https://jchultarsky101.github.io/pcli).

### <a id="downloading-binary"></a>Downloading a pre-compiled binary

You can download a pre-compiled binary for your platform from the [documentation site](https://jchultarsky101.github.io/pcli). You will have uncompress it and copy the file to location of your choice.

### <a id="compilation"></a>Compilation from source

You would need to have Rust installed on your computer to use this method. Clone this repository on your computer, navigate to the project root and compile with Rust:

````bash
cargo build release
````

This will produce an executable for your operating system.


## <a id="about-tenants"></a>A word about tenants

Physna is a multi-tenant system. As such, each client organization receives their own environment. 
For example, if your company is MyCompany, LLC, you may receive an environment and your
customer-facing web site may be at https://mycompany.physna.com. Here the identifier "mycompany" is
your tenant ID.

The tenant ID is a critical bit of information, which will be required from you almost every time you execute
the CLI. If you are unsure what your tenant ID is, please contact your Physna representative.

NOTE: If you specify a tenant that is unknown (not in the configuration file), the error message will list all currently configured tenants

# <a id="configuration"></a>Configuration

The tool uses a single configuration file. The location for this file is your home directory
and the file name is ~/.pcli.conf.

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

The only other piece of data you need to provide is the **client_id** value. This can be obtained
by your Physna administrator. In most cases, you should leave the client secret field unpopulated. Read further to understand how
it could be useful sometimes.

# <a id="basic-use"></a>Basic use

This is a command line utility. You can use it within a terminal session.
To see what functions are supported, you can type "help" as the first argument.

Example:

```bash
pcli help
```
```
╔═╗╔═╗╦  ╦
╠═╝║  ║  ║
╩  ╚═╝╩═╝╩

Physna Command Line Interface


CLI client utility to the Physna public API/V2

Usage: pcli [OPTIONS] --tenant <tenant> <COMMAND>

Commands:
  sysinfo
          Prints details of the current host system
  token
          Obtains security access token from the provider
  invalidate
          Invalidates the current access token, which will cause new token to be created next execution
  model
          Reads data for a specific model
  reprocess
          Reprocesses a specific model
  delete-model
          Deletes a specific model
  model-meta
          Reads the metadata (properties) for a specific model
  models
          Lists available models that meet the search criteria
  assembly-tree
          Reads the model's assembly tree
  match-model
          Matches all models to the specified one
  match-scan
          Scan-match all models to the specified one
  match-folder
          Matches all models in a folder to other models
  label-folder
          Labels models in a folder based on KNN algorithm and geometric match score as distance
  delete-folder
          Deletes a specific folder
  assembly-bom
          Generates flat BoM of model IDs for model
  status
          Generates a tenant's environment status summary
  upload
          Uploads a file to Physna
  download
          Downloads the source CAD file for the model into the default download directory
  upload-many
          Performs a bulk upload of all files in a directory
  upload-model-meta
          Reads metadata from an input CSV file and uploads it for a model specified by UUID
  match-report
          Generates a match report for the specified models
  folders
          Lists all available folders
  create-folder
          Creates a new folder
  properties
          Lists all available metadata propertie names and their IDs
  image-search
          Search for 3D model based on 2D image(s) (object identification)
  help
          Print this message or the help of the given subcommand(s)

Options:
  -t, --tenant <tenant>
          Your tenant ID (check with your Physna admin if not sure)

          [env: PCLI_TENANT=pre-prod-enterprise]

  -f, --format <format>
          Output data format (optional: e.g. 'json', 'csv', or 'tree')

          [env: PCLI_FORMAT=]
          [default: json]
          [possible values: json, csv, tree, table]

  -p, --pretty
          Produces pretty output (optional: default is 'false')

      --color <color>
          Adds color to the output (optional: e.g. 'black', 'red', 'green', 'yellow', 'blue', 'magenta', 'cyan', 'white')

          [possible values: black, red, green, yellow, blue, magenta, cyan, white]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

The application supports sub-commands. To get more detailed help for a
specific sub-command, enter the command name after "help" or alternatively,
first enter the command and add "--help" after as shown in the example:

```bash
pcli help model
```

Produces the following output:

```
Reads data for a specific model

Usage: pcli --tenant <tenant> model [OPTIONS] --uuid <uuid>

Options:
  -u, --uuid <uuid>  The model UUID
  -m, --meta         Enhance output with model's metadata
  -h, --help         Print help
  -V, --version      Print version
```

As you can see, here the **model** sub-command takes "--uuid" as an argument
to specify the UUID of the model we are trying to read.

Example):

```bash
pcli --tenant="mytenant" model --uuid="82cb38ce-c3e4-4a07-b605-5177602a6xxx"
```

Most, but not all command line arguments support both short and long names. For example, for tenant we can
specify the full name "--tenant" or the short name "-t". Those two are be equivalent. Not all arguments
have a short name. In those cases, the long name is the only option. We reserve those for cases where the argument is
rarely used or it is important to make sure we avoid mistakes.

## <a id="command-line-arguments"></a>Command line arguments

### <a id="order-of-arguments"></a>Order of arguments

There are two types of arguments:

* General use
* Specific to a sub-command

When composing your command line, you need to provide the general arguments first. If using a sub-command, type that next and
follow that with any sub-command specific arguments. Both the general and specific arguments could be optional. 

This is the idea:

```bash
pcli [general arguments] sub-command [command-specific arguments]
```

Here are few examples:

```bash
pcli help models
```

In this case, "help" is the subcommand. There are zero general arguments we need to provide for it.
We provide one more subcommand **models** to identify the exact sub-command for which we need specific help.

That was a very simple example. Here is a more complicated one:

```bash
pcli --tenant="mytenant" --format="cvs" --pretty models --folder="default" --search="part_name"
```

In this case, "tenant", "format", and "pretty" are all general arguments that apply accross many sub-commands.
On another hand "folder" and "search" are arguments that are specific to the **models** command.
Of course "models" is the sub-command itself.

### <a id="general-vs-specific-args"></a>How do I know which arguments are general and which are specific?

The general arguments are:

* --tenant
* --format
* --pretty

If you display the help without specifying a command name, you will see the info about the general arguments. If you do provide
a command name in the help request, you will see the command-specific arguments only.

Remember, provide the general arguments (if any) first, then the command name followed with any specific command arguments.

### <a id="signs-n-quites"></a>Equal sign, quotes, oh mine!

There is some free play when it comes of how you provide values to an argument. Strictly speaking, those rules are not implemented by PCLI,
but the operating system you are using and the terminal program you are using. However, it may be useful to go over few things here.

For example these two ways of executing the **folders** command are both valid and equivalent:

```bash
pcli --tenant mytenant folders
```

and

```bash
pcli --tenant="mytenant" folders
```

In other words, you can provide either the equal character ('=') or a space as the separator between argument and its value. 
The recommended way is to use the equal character (example #2) without any spaces between the '=' on either side.

If a value does not contain spaces or any other characters that my cause confusion, you can type it without surrounding it with quotes.
The best practice is to always wrap it in double quotes.

In general, it is considered O.K. to not surround numbers with double quotes. 
It is a bit more typing, but I recommend to be consistent and wrap all values as a matter of good habit.

### <A id="multi-value-args"></a>Arguments with multiple values

Few sub-commands can take multiple values for an argument. Such arguments are clearly indicated in the help. 
In an example you will see further down again:

```bash
pcli --tenant="mytenant" --format="csv" --pretty match-folder --folder="myfolder1" --folder="myfolder2" --threshold="0.99"
```

This means that the **match-folder** command will search simultaneously in two folders (i.e. foler with name "myfolder1" and a second folder with name "myfolder2") and combine the output 
for both. This way, you can widen the search in one command.

For connvenience, we provide an alternative method of specifying multiple values. 
You can use a single command line argument name and a comma-separated list of values.
The following is equivalent to the example above:

```bash
pcli --tenant="mytenant" --format="csv" --pretty match-folder --folder="myfolder1,myfolder2" --threshold="0.99"
```

That can come handy when using PCLI in conjusction with a custom business logic, which prefers to use comma-separated values.

### Arguments without a value

The argument "pretty" does not take a value. This is because it is a boolean argument or sometimes also called a flag. The mere presence of it indicates
that you want to use it. In this case:

```bash
pcli --tenant="mytenant" --pretty folders
```

it means that I would like to print a more human-readable JSON output than the default compact version meant for post-processing tasks. 
If I ommit "pretty" it is the same as setting a value of false for it, which is the default.

### Default values for general arguments via environment variables

If you only have one tenant it would be annoying to have to type it every time you use PCLI. Instead, you can setup an environment variable in your command line
shell. PCLI will use the value to automaically populate the tenant ID for you. How to create an environment variable and how to make that persistent for your shell
is beyond the scope of this document. It differs between operating systems. Consult online resources if you are not familiar.

Example of using an environment variable to set the tenant ID in Linux:

```bash
export PCLI_TENANT="mytenant"
```

Here we set the environment variable "PCLI_TENANT" to have value of "mytenant". When calling PCLI later, we do not need to provide --tenant as argument anymore. 
The value from the PCLI_TENANT variable is used. If you do specify the argument, it will override the one from the environment. In other words, the environment variable 
provides a default value for the argument if it is not explicitly specified.


```bash
pcli folders
```

There are two general arguments that can be configured as environment variables:

* --tenant - as "PCLI_TENANT"
* --format - as "PCLI_FORMAT"

## <a id="tokens"></a>Working with tokens

It is important to understand how the authentication and authorization work.
Physna uses OpenID Connect provider and upon successful authentication will issue the user
an access token, which will be valid for the duration of the session (several hours). As long
as your token is valid, you do not need to authenticate every time you run the PCLI utility.

The current token is stored by PCLI in a hidden file in your home directory. The file name 
is ".pcli.<tenant_id>.token". For example if your tenant is "beta", the file name for that
environment would be ~/.pcli.beta.token.

You can delete the token file at any time. If you do, PCLI will prompt you to authenticate again
and create a new one in its place. There is an easier way however. The CLI supports dedicated sub-commands
to deal with token generation:

### <a id="invalidate-token"></a>Invalidating your token

The sub-command **invalidate** will delete the current token for a given tenant. It will leave
unmodified any other tokens belonging to other tenants you may have.

```bash
pcli --tenant="mytenant" invalidate
```

This operation causes PCLI to start a new session next time it is executed by requesting a new token.

### <a id="print-token"></a>Displaying your token

Your token data is encrypted. It does not reveal anything about yourself or your system.
In the vast majority of cases, you would not care what it is. However, there may be some
special use cases where knowing your token is handy. One such case is if you are trying to
access the Physna's API in other ways than via this CLI. For example, this may be with cURL,
Postman, or some other client.

You do not need to execute this command just to get a new token. Every command you execute will attempt to obtain one if
none currently exist.

The following command will print the current token for your tenant to the terminal:

```bash
pcli --tenant="mytenant" token
```

### <a id="token-best-practices"></a>Best practices for handling tokens

First of all, make sure your home directory is properly secured with the correct file permissions. 
It is by default on all popular OS-es, but this is your responsibility. If not secure, there is 
much to worry about, not just the access token by Physna.

If you are automating your operations via shell scripts and you plan to invoke the CLI multiple times,
it is always a good idea to start with a fresh session. In your BASH script, call the "invalidate" command
first and once. You can then iterate over a batch of command executions without concern that your session
may expire in the middle of your work. This is handy especially when you have unattended executions (e.g. triggered by a cron job, etc.).

Also in the case of unattended executions, you could provide a value for your **client_secret** in your configuration file. This is
a less secure option, because it will not prompt you to enter it in your terminal every time you authenticate,
but if there is no human to enter it, it is difficult to automate. If you choose this path, make sure that 
your configuration file has the proper file permission to secure it against others that may share your
computer.

## <a id="list-folders"></a>Listing folders

The command **folders** will print the full list of folders currently available for your tenant.
Physna organizes data into logical storage units named folders. This is not unlike many other
systems you are familiar with. Each folder has a name. It also has a numeric identifier.
The folder ID is may be returned as part of the data Physna provides. For example, for 3D models,
it indicates which folder they belong to. The folder name is useful as a fiter for some operations (read further).

If you ask for the help screen of this command, you will see the following:

```bash
pcli help folders
```
```
Lists all available folders

Usage: pcli --tenant <tenant> folders [OPTIONS]

Options:
  -d, --folder [<folder>...]  Optional: Folder name (e.g. --folder=myfolder). You can specify this argument multiple times. If none specified, it will return all models in the tenant
  -h, --help                  Print help
  -V, --version               Print version
```

Example:

```bash
pcli --tenant="mytenant" folders
```
```
[{"id":1,"name":"Default Container"},{"id":2,"name":"Crawler"},{"id":3,"name":"myfolder"}]
```

The output of this command by default is formatted as compact JSON. This to allow you to chain the output
of this utility with other commands you may have after.

To make it a bit more human-readable, you can use the option "--pretty". This will pretty-print, or format
the output with some structure to make it easy to view.

```bash
pcli --tenant="mytenant" --pretty folders
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

The **folders** command has the optional argiment "--folder". This allows you to narrow down your result. For example, if you are only interested in folder "myfolders",
you could add the following filter:

```bash
pcli --tenant="mytenant" --pretty folders --folder myfolder
```
```
[
  {
    "id": 3,
    "name": "myfolder"
  }
]
```

You can provide more than one "--folder" value or none at all. If you have none, it will list all available folders. If you have some, it will provide only the folders that have matching names in the list you specified.

You can also add the option "--color" to make the output colorful if you wish:

```bash
$ pcli --tenant="mytenant" --pretty --color="green" folders
```

Some commands support other types of format. For example, you can receive the same information as CSV for
parsing later:

```bash
pcli --tenant="mytenant" --format="csv" folders
```

The output is:

```
1,Default Container
2,Crawler
3,myfolder
```

The default output format is "json". The available options are "json", "csv", "tree".

Adding "--pretty" in this case will add header row to the CSV output containing the column names.

## <a id="list-models"></a>Listing models

To obtain a list of models currently present in your tenant environment, use the **models** sub-command.
Please, note that there is also **model** (singular) command, which is used for querying a single model.

Example:

```bash
pcli help models
```

```
Lists available models that meet the search criteria

Usage: pcli --tenant <tenant> models [OPTIONS]

Options:
  -d, --folder [<folder>...]  Optional: Folder name (e.g. --folder=default). You can specify this argument multiple times. If none specified, it will return all models in the tenant
  -s, --search <search>       Optional: Search clause to further filter output (e.g. a model name)
  -h, --help                  Print help
  -V, --version               Print version
```

The **models** command takes a an optional argument "--folder", which is the folder name of interest and limits the search. If you use a folder name filter, but such does not exist, you will see an error message to that effect. 

If do not specify any folder names as a filter, it will return data from all available folders in your tenant.

Example for listing all available models in folder "myfolder":

```bash
pcli --tenant="mytenant" models --folder="myfolder"
```

The output from the above will include the list of models in folder with name "myfolder".

You can further filter the output of the "models" command by specifying an optional search term. For example, to list
all models in folder "myfolder" with model name containing the string "mypart", you can execute the following:

```bash
pcli --tenant="mytenant" models --folder="myfolder" --search="mypart"
```


As with the **folders** command, you can specify CSV as the output format, use "--pretty" and "--color".

The **models** command accepts multiple values for the folder name. You can query for the combined list of models from several folders in one command.
In the following example, you will get all models belonging in either folder "myfolder1" or folder "myfolder2":

```bash
pcli --tenant="mytenant" models --folder="myfolder1" --folder="myfolder2"
```

You can use the alternative notation with comma-separated values to achieve the same result:

```bash
pcli --tenant="mytenant" models --folder="myfolder1,myfolder2"
```

If one is provided, the search argument applies across the folders.


## <a id="query-model"></a>Querying for a specific model

The **model** command takes as mandatory argument the unique identifier (the UUID) of the model we are interested in. This is done via the "--uuid"
argument. The CLI will query your tenant for that specific model regardless which folder it belongs to.

```
Reads data for a specific model

Usage: pcli --tenant <tenant> model [OPTIONS] --uuid <uuid>

Options:
  -u, --uuid <uuid>  The model UUID
  -m, --meta         Enhance output with model's metadata
  -h, --help         Print help
  -V, --version      Print version
```

* uuid - is the models UUID
* meta - is an optional flag. When provided, it will query and add the metadata of the model to the output
    
```bash
pcli --tenant="mytenant" model --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx"
```

As explained before you can use different output formats, pretty-print, color.

## <a id="upload-model"></a>Uploading a model

The **upload** command assists you with uploading new 3D models to Physna. It takes the following arguments:

```bash
pcli help upload
```
```
Uploads a file to Physna

Usage: pcli --tenant <tenant> upload --folder <folder> --input <input>

Options:
  -d, --folder <folder>  Folder name (e.g. --folder=default)
  -i, --input <input>    Path to the input file
  -h, --help             Print help
  -V, --version          Print version
```

* "input" is the path to the file you would like to upload in your local file system
* "folder" is the Physna folder name that will be the destination for your upload

Here is an example of how all this comes together:

```bash
pcli --tenant="mycompany" upload --folder="myfolder" --input="/path/to/my/file" --units="mm"
```

If successful, it will upload the model in the file named "file".

Be aware of the following restrictions:

* You can upload the following file types: .3ds, .asm, .catpart, .catproduct, .glb, .iges, .igs, .obj, .par, .prt, .sldasm, .sldprt, .stl, .step, .stp, .x_b, .x_t
* Please be aware that files may take several minutes to process after uploading.
* Scans with missing facets or filesizes over 150MB may impact performance.
* Each part file should not exceed 1GB
* Each assembly file should not have more than 3,000 parts
* All part files should be uploaded with their assembly file(s)

## <a href="upload-many-models"></a>Uploading multiple models in one step

You can upload multiple models in one step if they are located in the same directory on your computer. In this case,
you can use the **upload-many** command. It is esentially the same as **upload**, but in this case the "--input" argument
is not a file, but the path to the directory on your computer where the files are staged.

```bash
pcli help upload-many
````
````
Performs a bulk upload of all files in a directory

Usage: pcli --tenant <tenant> upload-many --folder <folder> --input <input>

Options:
  -d, --folder <folder>  Folder name (e.g. --folder=default)
  -i, --input <input>    Path to the input directory
  -h, --help             Print help
  -V, --version          Print version
````

Alternativelly, you can write a script to call the **upload** command for each file you want to upload.

## <a id="download-model"></a>Downloading model file

The **download** command will download the original source file of the model into your default download directory.
The exact location of the download directory depends on your operating system, but it should be a sub-directory of your home directory.

```bash
pcli help download
```
```
Downloads the source CAD file for the model into the default download directory

Usage: pcli --tenant <tenant> download --uuid <uuid>

Options:
  -u, --uuid <uuid>  The model UUID
  -h, --help         Print help
  -V, --version      Print version
```

This command does not return any data. The result is a new file saved in your download directory. If a file with the same name already existed there, it will be overriden.

Example:

Suppose you previously uploaded a 3D model file named "myfile.stl" and it is now available in your tenant and has been assigned UUID of 511e65e7-d217-4873-af8d-2e3a438bxxxx.
If you execute the following command:

```bash
pcli --tenant="mytenant" download --uuid 511e65e7-d217-4873-af8d-2e3a438bxxxx
```

if successful, you should see a file name "myfile.stl" in your default download directory.

## <a id="reprocess-model"></a>Reprocessing a model

The **reprocess** command is useful to recover from situations when a model has been uploaded, but for some reason its indexing
in Physna has not completed normally. It takes mandatory parameter: the UUID of the model we want to reprocess.

```bash
pcli help reprocess
```
```
Reprocesses a specific model

Usage: pcli --tenant <tenant> reprocess --uuid <uuid>...

Options:
  -u, --uuid <uuid>...  The model UUID
  -h, --help            Print help
  -V, --version         Print version
```

Example:

```bash
pcli --tenant="mytenant" reprocess --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx"
```

This will cause the status of the model to be reset to "reprocessing" and the model will progress through the normal steps of processing and indexing as when uploading a new file.

The command produces no output.

The **reprocess** command has an alias **reprocess-model**. The following is equivalent to the above:

```bash
pcli --tenant="mytenant" reprocess-model --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx"
```

The **reprocess** command takes multiple values for the parameter --uuid. Therefore, you can reproces multiple models in one operation:


```bash
pcli --tenant="mytenant" reprocess --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx" --uuid="95ac73f8-c086-4bec-a8f6-de6ceazzzzzz"
```

Alternativelly, you can use a comma-separated values for the UUID: --uuid="98797abc-bb3d-4898-9262-3b82827f43adxxxxxxx, 98797abc-bb3d-4898-9262-3b82827f43adyyyyyyy"

## <a id="delete-model"></a>Deleting a model

This command will delete a model and all related metadata from the Physna database.

Example:

```bash
pcli --tenant="mytenant" delete-model --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx"
```

The same command has an alias "delete". The same operation can be performed this way:

```bash
pcli --tenant="mytenant" delete --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx"
```

As with the **reprocess** command, it takes multiple values for --uuid to allow you to delete many models in one execution.

**NOTE:** Please, be extra careful when running bulk delete operations. Once deleted, a model cannot be recovered by Physna.
You would have to upload it again.

## <a id="read-meta"></a>Reading metadata

In addition to the 3D geometry data, additional metadata can be associated with the model.
The metadata is in the form of name/value pairs. Both the name and the value are UTF-8 strings.
The metadata is returned as part of the model data when using the commands **model** or **models**.
However, PCLI offers an additional specialized command to only retrieve the metadata and not the rest of the model data.
This is useful when scripting more sophisticated solutions. 

The command is:

```bash
pcli help model-meta
````
```
Reads the metadata (properties) for a specific model

Usage: pcli --tenant <tenant> model-meta --uuid <uuid>

Options:
  -u, --uuid <uuid>  The model UUID
  -h, --help         Print help
  -V, --version      Print version
```

It takes one mandatory argument - the model's UUID.
The output by default is JSON, but when we specify CSV, the output contains the following columns:

* MODEL_UUID - the UUID of the model
* NAME - the property name
* VALUE - the property value

**NOTE:** The property names are case insensitive and are capitalized.

Example:

```bash
pcli --tenant="mytenant" --format="csv" --pretty model-meta --uuid="97377547-9062-4149-90f7-16daf400148x"
```
```
MODEL_UUID,NAME,VALUE
97377547-9063-4149-90f7-16daf400148x,DESCRIPTION,Test description
97377547-9063-4149-90f7-16daf400148x,SKU,Test
```

In this example, the model has two properties ("DESCRIPTION" and "SKU") with their corresponding names.

The reason for the UUID of the model to be included as the first column is simple. You can concatenate the output of many executions of this command into one single file. That larger file will contain metadata for many models. You will see how that becomes helpful in the next section.

## <a id="upload-meta"></a>Uploading metadata

In some cases, we need to associate additional metadata with the geometry of a model. The command **upload-model-meta** serves this purpose.

```bash
pcli help upload-model-meta
```
```
Reads metadata from an input CSV file and uploads it for a model specified by UUID

Usage: pcli --tenant <tenant> upload-model-meta --input <input>

Options:
  -i, --input <input>  Path to the input file
  -h, --help           Print help
  -V, --version        Print version
```

The file format is the same as the CSV-formatted output produced by the command **model-meta**.

**NOTE:** This command only works with the CSV format. It does not work with JSON. We may implement that option in a future release.

The columns are: MODEL_UUID,NAME,VALUE. One use case is to first read the metadata for some models, edit it externally (for example, with a text editor). This may include modifying values for existing properties or adding new properties and their values.

The required argument is "input" - the name of the CSV formatted input file. There is no need for --uuid here because the UUID is included
in the input file as the first column.

If a property with this name already exists for the model, its value will be overridden with the new value provided.
If the property does not exist, a new property with the provided (but capitalized) name will be created.

**NOTE:** If the metadata property value is an empty string, this command will delete the property for the model. In other words, if you want to delete a property, upload the same with value of an empty string in the input CSV file.

## <a id="read-asm"></a>Reading the assembly structure

The command **assembly-tree** will query for a specific model and return as result the assembly structure.
Obviously, this is ony useful when working with assemblies. The assembly tree could be recursive with 
assemblies having sub-assemblies, and so forth.

The **assembly-tree** command supports the unique output format of "tree".

## <a id="match-model"></a>Matching models to other models

Physna's core expertise is in finding geometric matches for models. The sub-command **match-model** does
part-to-part match. This means that a model is matched as a unit to all other models in the system. 
With this function, we do not cascade from top-level assembly into all of its sub-assemblies, nor we 
try to determine if this model may be a component of another assembly. 

```bash
pcli help match-model
````
```
Matches all models to the specified one

Usage: pcli --tenant <tenant> match-model [OPTIONS] --uuid <uuid> --threshold <threshold>

Options:
  -u, --uuid <uuid>                      The model UUID
  -t, --threshold <threshold>            Match threshold percentage (e.g. '96.5')
  -m, --meta                             Enhance output with model's metadata
      --classification <classification>  The name for the classification metadata property
      --tag <tag>                        The value for the classification metadata property
  -h, --help                             Print help
  -V, --version                          Print version
```

* "uuid" is the UUID of the model we are trying to match.
* "threshold" is the match level. This is a floating point value between [0..1]. For example, 80% match would be 0.8.
* "meta" is an optional flag. When specified, we will query for additional metadata and if present we will add that to the output.
* "classification" is an optional argument and requires that he "meta" is present. It is the name of a metadata property that will be set for each matching model. This way the user can permanently tag models.
* "tag" requirest that the classification argument is present. It is the value to be associated with the "classification" property.

Example:

```bash
pcli --tenant="mytenant" match-model --uuid="95ac73f8-c086-4bec-a8f6-de6ceaxxxxxx" --threshold="97.5"
```

The output contains the list of models that matched the criteria and a value between zero and one indicating the fit.
A value of 1 means 100% match.

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

## <a id="match-folder"></a>Matching entire folders of models

Sometimes, we need to execute match models in bulk. With the commands already provided so far, you could create a driver script to
achieve the effect, but we provide a convenience method for this purpose. In other words, this command will query for the list of models in your folder and for each it will execute **match-model**. It will combine the responses into a single output. The input arguments are the same as the previous command.

```bash
pcli help match-folder
```
```
Matches all models in a folder to other models

Usage: pcli --tenant <tenant> match-folder [OPTIONS] --threshold <threshold>

Options:
  -t, --threshold <threshold>  Match threshold percentage (e.g. '96.5'
  -d, --folder [<folder>...]   Optional: Folder name (e.g. --folder=default). You can specify this argument multiple times. If none specified, it will return all models in the tenant
  -s, --search <search>        Search clause to further filter output (optional: e.g. a model name)
  -e, --exclusive              If specified, the output will include only models that belong to the input folder
  -m, --meta                   Enhance output with model's metadata
  -h, --help                   Print help
  -V, --version                Print version
```

As with the **models** command, you can provide multiple folder name filters, or none at all if you want to match your entire database. However, it is recommended that you always try to narrow down your serches as much as possible for better performance.

Example:

The following command will execute individual matches for all models found in the folder with name "myfolder" at match threshold of 99% (--threshold=0.99).
It will output the result in CSV format (--format=csv) and add header line with column names to it (--pretty).

```bash
pcli --tenant="mytenant" --format="csv" --pretty match-folder --folder="myfolder" --threshold="0.99"
```

You can also specify a search term to further narrow down the filter. Finally, the "--meta" flag will cause any associated metadata to be added to the output.

## <a id="match-scan"></a>Matching scanned model

If you have uploaded a 3D model that has been generated by a 3D scanner techolgy (e.g. photogrammetry), the tessellation may be widely different than a model produced by a CAD system.
The geometry between the two models will be very different and you most likelly would not get a good match. In this situation, Physna provides a different algorithm, which is based 
on bounding box and other parameters. You may get better results this way.

```bash
pcli help match-scan
```
```
Scan-match all models to the specified one

Usage: pcli --tenant <tenant> match-scan [OPTIONS] --uuid <uuid> --threshold <threshold>

Options:
  -u, --uuid <uuid>                      The model UUID
  -t, --threshold <threshold>            Match threshold percentage (e.g. '96.5')
  -m, --meta                             Enhance output with model's metadata
      --classification <classification>  The name for the classification metadata property
      --tag <tag>                        The value for the classification metadata property
  -h, --help                             Print help
  -V, --version                          Print version
```

Otherwise the use is very similar to the **model-match** command.

## <a id="match-report"></a>Generating a match report

The **match-report** command combines multiple operations. It is used to generate comprehensive match report that
could be used as input for further post processing. For example, machine learning algorithms. It produces multiple
outputs and therefore it requires the user to specify file names for each output.

```bash
pcli help match-report
```
```
Generates a match report for the specified models

Usage: pcli --tenant <tenant> match-report [OPTIONS] --uuid <uuid> --threshold <threshold> --duplicates <duplicates> --graph <graph> --dictionary <dictionary>

Options:
  -u, --uuid <uuid>              Top-level assembly UUID (you can provide multiple)
  -t, --threshold <threshold>    Match threshold percentage (e.g. '96.5')
  -d, --duplicates <duplicates>  Output file name to store the duplicate report in CSV format
  -g, --graph <graph>            Output file name to store the assembly graph in DOT Graphviz format
  -r, --dictionary <dictionary>  Output file name to store the index-name-uuid dictionary in JSON format
  -m, --meta                     Enhance output with model's metadata
  -h, --help                     Print help
  -V, --version                  Print version
```

Example:

```bash
pcli --tenant="mytenant" match-report \
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
cat ./my_graph.graphviz | dot -Tsvg > my_graph.svg
```

This will produce a SVG file, which you can view by opening it in your browser or another graphics viewer.

## <a id="environment-status"></a>Tenant environment status

```bash
pcli help status
```

```
Generates a tenant's environment status summary

Usage: pcli --tenant <tenant> status [OPTIONS] --folder <folder>

Options:
  -d, --folder <folder>  Folder name
  -r, --repair           Forces repair operation on any model that is not in status FINISHED
      --noasm            When using --repair, this flag causes assmeblies to be ignored
  -h, --help             Print help
  -V, --version          Print version
```

We provide a convenience command to check on the status of folders in your environment.

The following command would output details about the number of models in the specified folder
per type of file and status.

Example:

```bash
pcli --tenant="mytenant" --format="csv" --pretty status --folder="myfolder"
```

It will produce a summary report with stats of model types and their processing states. A state of "FINISHED" means that all is well and
the model is ready for use. Status of "FAILED" indicates that there is a data issue with the model or perhaps the file does not 
contain any valid geometry.

The "status" command takes an optional flag "--repair". When specified, PCLI will automatically issue **reprocess** command for any model
that is not in "FINISHED" state. Please, notice that the reprocessing takes time and it is an offline action in Physna. 
Therefore, the model will not immediately appear in "FIXED" state. You may need to wait a bit and re-run the **status** command until all
background processing completed.

The --noasm flag can be used when the --repair flag is specified. It causes assmeblies to be excluded from the repair process.

## <a id="2D-to-3D"></a>Searching for 3D models by 2D image

In some cases, we want to find a 3D model by providing a 2D image of the object. For example, we could take a photo with our mobile phone and want to identify the 3D model
that corresponds to this image.

Physna provides this functionality via Vector Based Similarity Search. It is a machine learning algorithm, but it does not need supervised training - it is ready to use immediatelly.

To search by image, PCLI implements the **image-search** command:

```bash
pcli help image-search
```

```
Usage: pcli --tenant <tenant> image-search [OPTIONS] --input <input>

Options:
  -i, --input <input>    Path to the input file
  -l, --limit <limit>    Maximum number of results to be returned (default is 20) [default: 20]
  -s, --search <search>  Search clause to further filter output (optional: e.g. a model name)
  -f, --filter <filter>  Physna filter expression. See: https://api.physna.com/v2/docs#model-FilterExpression
  -h, --help             Print help
  -V, --version          Print version
```

It takes four arguments:
* input - the path to the image file on your local file system
* limit - the maximum number of matches to be returned. This is optional with default of 20
* search - is optional argument. When provided it will sub-select the results based on the text value. For example, you could provide folder name or the value of a metadata property
* filter - is optional. You could specify a Physna filter expression for detailed control of the query. For more details on the syntax, please see [Physna's documentation](https://api.physna.com/v2/docs#model-FilterExpression)

The search results are sorted where the most likely matches are returned first.

Example:

````bash
pcli --tenant="mytenant" --format=csv --pretty image-search --input my_picture.JPG --limit 30
````

This will return a list of matching model records in CSV format. Only the top 30 matches will be returned.

For best results, you should specify the folder in which your models reside by utilizing the --filter argument. For example, if I know that my 3D models are in folder with ID of 100, 
I would add the following filter expression:

````bash
pcli --tenant="mytenant" --format=csv --pretty image-search --input my_picture.JPG --limit 30 --filter='folderId(eq(100))'
````

This would provide the result faster and more accuratelly than searching the entire database.

It is important to take photos that show as many geometric features of the object as possible. In some cases, to get a better match, we need to provide
multiple images of the same object taken from different angles. PCLI allows you to upload multiple images by repeating the --input argument.


````bash
pcli --tenant="mytenant" --format=csv --pretty image-search --input my_picture_take1.JPG ---input my_picture_take2.JPG-limit 30 --filter='folderId(eq(100))'
````

Behind the seens, PCLI will execute two (or more) queries against Physna for each of your pictures. It will then combine the results by ranking up those that 
are repeating in the outputs.

## <a id="model-label"></a>Model labeling

The PCLI client provides its own mechanism for label propagation, which is form of object classification. 
This is implemented in the **label-folder** command. 
It is based entirely on geometric match scores provided by Physna. 

The user provides 3 mandatory and one optional input arguments:

* "folder" - the target folder name in your tenant
* "classification" - the name of a metadata property that will be used for classification
* "threshold" - the confidence threshold value
* "exclusive" - this is a flag and does not take a value. If present, only the models found in the source fodler will be considered for matching. The default is to consider all models in the tenenat regardless of their parent folder.

When executed, PLCI will read the contents of the folder and for each of the models in it, 
it will perform part-to-part match as in the **match-model** command.
The match will be done with the specified threshold. 
It will then rank the matches by their scores and starting from the highest to the lowest will check if
the matching models contain a value for the metadata property specified. 
If they do, the model will also be assigned that same metadata property and value.

The assumption is that if model "A" has metadata property of "classification" with value of "apple" and model "B" is 98.5% geometrically the same as model "A", than we can say with
98.5% confidence that model "B" is also an "apple". 
We indicate that by setting model "A"'s metadata property "classification" to have value of "apple"

```bash
pcli help label-folder
````
```
Labels models in a folder based on KNN algorithm and geometric match score as distance

Usage: pcli --tenant <tenant> label-folder [OPTIONS] --folder <folder> --threshold <threshold> --classification <classification>

Options:
  -d, --folder <folder>
          Folder name
  -t, --threshold <threshold>
          Match threshold percentage (e.g. '96.5')
  -c, --classification <classification>
          The name for the classification metadata property
  -s, --search <search>
          Search clause to further filter output (optional: e.g. a model name)
  -m, --meta
          Enhance output with model's metadata
  -e, --exclusive
          If specified, the output will include only models that belong to the input folder
  -h, --help
          Print help
  -V, --version
          Print version
```

Example:

```bash
pcli --tenant="mytenant" label-folder --folder="myfolder" --threshold=0.9 --classification="classification"
```

The optional --search argument may be used to further refine the target list of models. Only models that match the search
criteria will be labeled and all others ignored. The --search option works the same as for the **models** command.

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

# <a id="errors"></a>Handling erors

Errors may occur for many different reasons. They can be caused by incorrect user input, network operations, etc. 

When an error occurs, the process is interrupted and an error message is returned to the user.

It is important to understand that CLI commands in general have two seprate [standard output streams](https://en.wikipedia.org/wiki/Standard_streams). 
One is called STDIN and it is used for normal output. The other is called STDERR and it is used to print erro messages.

PCLI follows this convention and it will print out error messages to STDERR. This way, if you are piping the output from PCLI to another downstream process
it will not receive confusing data that may include error messages.

Finally, CLI commands normaly return exit code of zero in case of successful operation and exit code other than zero when they encounter errors. PCLI follows
the same rule. You can check the process exit code from PCLI to determine if the operation was successful or not.

# <a id="advanced-use"></a>Advanced use

## <a id="pipes"></a>Using pipes

The real power of this CLI tool comes when you use it in conjunction with other tools. For example,
you can filter down the list of models further by piping the output (formatted as JSON) to [JQ](https://stedolan.github.io/jq/):

```bash
pcli -t="mytenant" models --folders="myfolder" | jq '.[] | select(.id=="96049555-b55a-45b1-bdcb-2555cb0012fe")'
```

JQ has many useful features that could help you manipulate the output as needed.

You can pipe the output to a file on your disk for post-processing of the output:

```bash
pcli -t="mytenant" --format="csv" models --folders="myfolder" > myfile.csv
```

Be aware that "--pretty" adds more to the output. For example, if your output format is CSV, it will add
a header record. If your post-processor counts the number of records in the CSV to tally the number of
models found (as example), you will have to ignore the first record. In this case it is probably better 
not to include the "--pretty" flag. This argument is binary and does not take a value. If it is present, it
means that it is active; if not, it is effectively set to false.

## <a id="nushell"></a>Using NuShell

[NuShell](https://www.nushell.sh/) is an excellent partner for PCLI. You can use the combination of the two to a great effect.
Instead of using the CSV format, you can ask NuShell to parse the JSON output of PCLI and even query the results in interesting ways.

The scope of this document is not to explain in length how this works, but here is an example:

```bash
pcli --tenant="mytenant" models --folder="myfolder" | from json | where name =~ 'Block' | select id name state | to csv
```
```
id,name,state
81a9f730-6a69-4a0d-ae6e-e737d34ca744,Block_Puzzle_1.STL,finished
4345f06d-5113-4329-9aeb-f5e04f34f8aa,Block_Puzzle_3.STL,finished
97b01c4f-5570-4790-9139-d0f50b54a8fd,Block_Puzzle_2.STL,finished
6f071997-19f0-4311-a14e-41f5c168762e,Block_Puzzle_5.STL,finished
4c13d2d4-8bef-43c3-afad-26d248a3da80,Block_Puzzle_4.STL,finished
```

This command will execute PCLI and ask it to return the list of models found in folder "myfolder".
Next, it will filter out only thouse models that have the worf "Block" in the model's name.
It will then subselect only the data from properties "id", "name", "state".
Finally, it will format the output as CSV.

Please, read the NuShell documentation for all the wonderful ways you can use it for data manipulations.


# <a id="support"></a>Support

If you have any questions, please e-mail to [jchultarsky@physna.com](mailto:jchultarsky@physna.com).

<!-- ROADMAP -->
# <a id="roadmap"></a>Roadmap

_The project is work in progress. No release has been provided as of yet. Most of the work is under the 'develop' branch._

- ✅ Implement basic operations (CRUD, search)
- ✅ Implement advanced operation (match report)
- ✅ Add Changelog
- ✅ CI/CD
- ✅ Create documentation
- ✅ Better error messages
- Test suite
- Local data cache

See the [open issues](https://github.com/jchultarsky101/pcli/issues) for a full list of proposed features (and known issues).

<!-- CONTRIBUTING -->
# <a id="contributing"></a>Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<!-- LICENSE -->
# <a id="license"></a>License

Distributed under the Apache License. See `LICENSE` for more information.

<!-- CONTACT -->
# <a id="contact"></a>Contact

Julian Chultarsky - [@jchultarsky101](https://twitter.com/jchultarsky101) - jchultarsky@physna.com

Project Link: [https://jchultarsky101.github.io/pcli](https://jchultarsky101.github.io/pcli)

<!-- ACKNOWLEDGMENTS -->
# <a id="acknowledgments"></a>Acknowledgments

* [Choose an Open Source License](https://choosealicense.com)
* [GitHub Emoji Cheat Sheet](https://www.webpagefx.com/tools/emoji-cheat-sheet)
* [Img Shields](https://shields.io)
* [GitHub Pages](https://pages.github.com)
* [Font Awesome](https://fontawesome.com)
* [Best-README-Template](https://github.com/othneildrew/Best-README-Template)
* [Clap](https://crates.io/crates/clap)
* [Configuration](https://crates.io/crates/configuration)
* [Dirs](https://crates.io/crates/dirs)
* [Env](https://crates.io/crates/env)
* [Keyring](https://crates.io/crates/keyring)
* [Log](https://crates.io/crates/log)

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/jchultarsky101/pcli.svg?style=for-the-badge
[contributors-url]: https://github.com/jchultarsky101/pcli/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/jchultarsky101/pcli.svg?style=for-the-badge
[forks-url]: https://github.com/jchultarsky101/uacli/network/members
[stars-shield]: https://img.shields.io/github/stars/jchultarsky101/pcli.svg?style=for-the-badge
[stars-url]: https://github.com/jchultarsky101/pcli/stargazers
[issues-shield]: https://img.shields.io/github/issues/jchultarsky101/pcli.svg?style=for-the-badge
[issues-url]: https://github.com/jchultarsky101/pcli/issues
[license-shield]: https://img.shields.io/github/license/jchultarsky101/pcli.svg?style=for-the-badge
[license-url]: https://github.com/jchultarsky101/pcli/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://www.linkedin.com/in/julianchultarsky
[product-screenshot]: images/screenshot.png
[Rust-url]: https://www.rust-lang.org/
[Rust-logo]: http://rust-lang.org/logos/rust-logo-128x128.png

