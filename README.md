# tskmstr

**Aggregates all tasks/issues together, across your entire world**

> I wrote this to better manage my world, and view across all aspects of my work and personal endeavours.

**tskmstr** is not SaaS, it uses the existing issue stores you intreract with.

<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [tskmstr](#tskmstr)
  - [Installation](#installation)
  - [Building](#building)
  - [Configuration](#configuration)
  - [Usage](#usage)
    - [Adding a Task](#adding-a-task)
  - [Closing a Task](#closing-a-task)
  - [Listing Tasks](#listing-tasks)
  - [Adding and Removing Labels](#adding-and-removing-labels)
  - [Command Reference](#command-reference)
  - [Features](#features)
  - [Contributing](#contributing)
  - [License](#license)

<!-- /code_chunk_output -->


**tskmstr** is a simple command-line tool designed to help you manage and organize your tasks and issues across various aspects of your life. 

It aggregates your TODO/Task/Work items from
- github issues
- gitlab issues

You can use a private repo on gitlab, or github to store your personal **TODO** items, and aggregate these with opensource, and private projects you contribute and work on.

With tskmstr, you can efficiently interact with your tasks, categorize them with labels, and view them, and perform basic operations on them (new, close, re-label).

For more complicated activties, (triage, workflow, attachments) complex editing. Then this tool is not that. Think of **tskmstr** as a pane of glass, into the aggregated view of tasks/todo/lists you need to work on.

## Installation

Download the latest release for your OS from 

* [relases](https://github.com/rbuckland/tskmstr/releases)

1. place it in your path (~/.local/bin, /usr/local/bin)
2. Create a symlink `t` (it's less to type)
    `ln -s $(which tskmstr) $(dirname $(which tskmstr))/e`
3. Now configure - [Configuration](#configuration)

## Building

To use **tskmstr**, you'll need to build it from source. Follow these steps:

1. Clone the repository:

   ```sh
   git clone https://github.com/rbuckland/tskmstr
   cd tskmstr
   ```

2. Build the project using Cargo:

    ```
    cargo build --release
    ```

4. Install
   
    For convenience, the tskmstr binary is just called "t".

    ```
    mkdir -p ~/.local/bin && cp ./target/release/t ~/.local/bin/t
    ```

5. Configure it        

## Configuration

Before using **tskmstr**, you need to configure it with your GitHub and/or GitLab credentials. **tskmstr** reads your credentials securely from your OS keyring.

The configuration is stored in a YAML file, which should look like this:


1. Create a new file `~/.config/tskmstr/tskmstr.config.yml`

```yaml

colors:
  issue_id: bright red
  title: blue
  tags: bright green

labels:
  priority_labels:
    - todo
    - urgent
    - now

github.com:
  credential:
    service: github.com
    username: key_username_in_keyring
  repositories:
    - id: 🅆
      color: blue
      owner: yourgithub_org
      repo: github_repo
      defaults:
        for_new_tasks: true
    - id: 🄿
      color: blue
      owner: other_github_org
      repo: github_repo2

gitlab.com:
  credential:
    service: gitlab.com
    username: key_username_in_keyring
  repositories:
    - id: Ⓐ
      color: blue
      project_id: group%2Fsubgroup%2Frepo

```
Each repository needs a unique character (one or more letters assigned), so you can refer
to eacn issue/task individually across the aggregated set.

You can configure colors, set priority labels, and specify your repositories on both GitHub and GitLab.

2. Put your PAT / API Token passwords into the OS Keyring. 

To add your credentials to the keyring, use the following command for each service:
```
keyring set github.com key_username_in_keyring
keyring set gitlab.com key_username_in_keyring
```

**note, on Ubuntu the `keyring` CLI tool is provided by `python3-keyring`

Now you're ready to start using tskmstr!

```sh
t 
```

Example output looks like below. 
In this example, the ID's for the repos are
    * 🅆 - for a work repository
    * Ⓐ - for a application repository
    * 🄿 - for a personal repository (which has no code, just tasks to do)

```
Priority: now, urgent, todo
----------------------------------------
 - 🅆/54 analyze rock formations (urgent, geology, research)
 - 🅆/53 study sedimentary layers (urgent, geology, analysis)
 - Ⓐ/18 survey geological fault lines (urgent, geology)
 - Ⓐ/8 analyze soil composition (urgent, geology)
 - Ⓐ/7 investigate geological formations (urgent, geology)
 
Tag: <no labels>
----------------------------------------
 - 🅆/16 geological survey of mountain ranges ()
 - 🄿/14 Study Earth's crust composition ()
 - 🄿/13 Analyze rock strata for fossils ()
 - 🄿/1 Study geological time periods ()
 - Ⓐ/19 Volcanic activity observation ()
 - Ⓐ/6 Study seismic fault lines ()

Tag: car
----------------------------------------
 - 🅆/48 organise monthly debit for car wash of ute (car)
 - 🅆/47 re-order instruments in the rear tray (car)

Tag: helpful
----------------------------------------
 - Ⓐ/14 Geological research library (helpful)

Tag: client-3112
----------------------------------------
 - 🅆/30 analyze soil quality for gardening (client-3112)
 - 🅆/29 geological assessment of backyard (client-3112)
 - 🅆/28 Soil stability testing (client-3112)
 - 🅆/27 Foundation rock type analysis (client-3112)
 - 🅆/26 Geological inspection of basement (client-3112)
 - 🅆/25 Geological assessment of attic (client-3112)
 
Tag: hr3
----------------------------------------
 - Ⓐ/17 Geological documentation for HR3 project (hr3)
 - Ⓐ/16 Geological panel report (hr3)

Tag: renovations
----------------------------------------
 - 🅆/20 Geological assessment for pool excavation (renovations)

```

Each of the "repositories" has a unique ID which comes from the config file `<gl><nnn>/<issue_id>` or `<gh><nnn>/<issue_id>`


## Usage
### Adding a Task

To add a new task to your default repository, use the add command:

```
tskmstr add "Task Title" "Task Details" tag1 tag2 tag3
``````

This command adds a new task with the specified title, details, and tags.
It will add it to the `default`, which is set in the config.

Use this form when adding a task to a specified repository.
The `provider-id` is the entry in the config `id: K` or `id: Ⓐ` for example.

```
t --provider-id K add <title> <details>
```

### Closing a Task

To close a task, use the close command:

```
t close <issue_id>
```

Replace <issue_id> with the ID of the task you want to close. (e.g. `Ⓐ/22`, `gh2/444`)

### Listing Tasks

To list all your tasks, grouped by labels and priority, simply run:

```
t
```

## Adding and Removing Labels

You can add and remove labels from a task using the tag add and tag remove commands:

```
tskmstr tags add <issue_id> tag1 tag2 tag3
tskmstr tags remove <issue_id> tag1 tag2 tag3
```

## Command Reference

    add: Add a new task/issue to the default repository.
    close: Close a task/issue.
    list: List all tasks/issues, grouped by labels and priority.
    tags add: Add tags to a task.
    tags remove: Remove tags from a task.


## Features

* [features](doc/features_roadmap.md)

## Contributing

Contributions to tskmstr are welcome! Please check out the [contribution guidelines](./Contributing.md) for more details.

## License

This project is licensed under the MIT License - see the LICENSE.md file for details.

Enjoy using tskmstr to stay organized and manage your tasks across multiple repositories! If you have any questions or encounter issues, feel free to reach out to our community. Happy task management!