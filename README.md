# tskmstr

**Aggregates all tasks/issues together, across your entire world**

> I wrote this to better manage my world, and view across all aspects of my work and personal endeavours.

**tskmstr** is not SaaS, it uses the existing issue stores you intreract with.

<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [tskmstr](#tskmstr)
  - [Installation](#installation)
  - [Configuration](#configuration)
  - [Terminology](#terminology)
  - [Usage](#usage)
    - [Listing Tasks](#listing-tasks)
    - [Adding a Task](#adding-a-task)
    - [Closing a Task](#closing-a-task)
  - [Adding and Removing Labels](#adding-and-removing-labels)
  - [Listing Issue Stores](#listing-issue-stores)
  - [Command Reference](#command-reference)
  - [Features](#features)
  - [Building](#building)
  - [Contributing](#contributing)
  - [License](#license)

<!-- /code_chunk_output -->


**tskmstr** is a simple command-line tool designed to help you manage and organize your tasks and issues across various aspects of your life. 

It aggregates your TODO/Task/Work items from
- github issues
- gitlab issues
- jira issues

You can use a private repo on gitlab, or github to store your personal **TODO** items, and aggregate these with opensource, and private projects you contribute and work on.

With tskmstr, you can efficiently interact with your tasks, categorize them with labels, and view them, and perform basic operations on them (new, close, re-label).

For more complicated activities, (triage, workflow, attachments) complex editing. Then this tool is not that. Think of **tskmstr** as a pane of glass, into the aggregated view of tasks/todo/lists you need to work on.

## Installation

Download the latest release for your OS from 

* [Compiled Releases](https://github.com/rbuckland/tskmstr/releases)
   - Linux
   - Mac
   - Windows

1. Place it in your PATH (~/.local/bin, /usr/local/bin)
2. Create a symlink `t` (it's less to type)
    `ln -s $(which tskmstr) $(dirname $(which tskmstr))/e`
3. Now configure - [Configuration](#configuration)


## Configuration

Before using **tskmstr**, you need to configure it with your GitHub, Jira and/or GitLab credentials. **tskmstr** reads your credentials securely from your OS keyring.

Refer to the comprehensive sample configuration [sample-config](sample/tskmstr.config.yml), that provides a good set of examples.

This config file needs to go into:

- Linux - `~/.config/tskmstr/tskmstr.config.yml`
- Windows - `%LOCALAPPDATA%/tskmstr/tskmstr.config.yml`
- Mac OSX - `~/Library/Preferences/tskmstr/tskmstr.config.yml`

You can override the config file with `--config`

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
      - provider_id: Stuff On GitHub
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
            filter: labels=bugs    
          - id: 🄿
            color: blue
            owner: other_github_org
            repo: github_repo2


    gitlab.com:
      - provider_id: work-repos
        credential:
          service: gitlab.com
          username: key_username_in_keyring
        repositories:
          - id: Ⓐ
            color: blue
            project_id: group%2Fsubgroup%2Frepo
            filter: labels=phase::selected         

    jira:
      - provider_id: Jira on SaaS
        endpoint: https://yourjira-instance.atlassian.net
        credential:
          service: yourjira-instance.atlassian.net
          username: user@email.com # this and the password are used for the auth, so make it correct

        projects:
          - id: J # tskmstr short_code
            color: green
            project_key: KAN # the jira PROJECT_ID
            default_issue_type: Bug # this defaults to Task in the Code.
            close_transition_id: 31
            filter: labels in (label2, label9) AND assignee = currentUser()            

    ```
Each repository needs a unique character (one or more letters assigned), so you can refer
to each issue/task individually across the aggregated set.

You can configure colors, set priority labels, and specify your repositories on both GitHub and GitLab.

2. Put your PAT / API Token passwords into the OS Keyring. 

To add your credentials to the keyring, use the following command for each service:
```
keyring set github.com key_username_in_keyring
keyring set gitlab.com key_username_in_keyring
```

For jira, it needs to look like 

```
keyring set <your-jira-instance> <jirs-username>
# example
keyring set special.atlassian.net user@foobar.com
```

**note, on Ubuntu the `keyring` CLI tool is provided by `python3-keyring`

Now you're ready to start using tskmstr!

```sh
tskmstr
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

## Terminology

 Because we are aggregating across different vendor solutions, terminology does get a little mixed up. This table will help.
 
| What we Call It |   Gitlab      |  GitHub       | Jira |
|-----------------|---------------|---------------|------|
| Issue      | Issue (subtasks and epics are not supported by **tskmstr**)        | Issues | Issue (subtasks and epics are not supported by **tskmstr**)  |
| Tags^            | Labels        |  Labels       | Labels |


^ tags was chosen because it is less to "type" on the command line. But really tags and labels are synonymous.

**Provider** - a provider is the "system", github/gitlab/jira. In the configuration, this is a `provider_id:`.
             

**Issue Store, Issue/Task Repository** - specific configured repository of a provider. (it is synonymous with a `repository`) - the provider of issues. This is the "IssueStoreID" In the configuration it is `id:`

## Usage


**tskmstr** supports basic operations.
- list tasks (optionally filtered)
- add a new task
- close a task
- add tags to a task
- remove tags from a task

For anything more complex, we suggest you use the dedicated UI or CLI tool of each solution.
- `glab` - Command Line tool for GitLab [gh CLI](https://github.com/cli/cli)
- `gh` - Command Line tool for GitHub - [glab CLI](https://docs.gitlab.com/ee/editor_extensions/gitlab_cli/)
- `go-jira` - 3rd Party Command Line tool for Jira.[go-jira CLI](https://github.com/go-jira/jira)

### Listing Tasks

To list all your tasks, grouped by labels and priority, simply run:

```
tskmstr

# to filter on just one repo/project
tskmstr list -i P
```

### Adding a Task

To add a new task to your default repository, use the add command:

```
tskmstr add "Task Title" "Task Details" tag1 tag2 tag3

# only add a task to the "W" repo
tskmstr add -i W "Task Title" "Task Details" tag1 tag2 tag3

```

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

# close issue 101 on repo, with `id` X 
tskmstr close X/101

#  close a Jira ticket
tskmstr close JIRA/ABC-123
```

Replace `<issue_id>` with the ID of the task you want to close. (e.g. `Ⓐ/22`, `gh2/444`)
The issue ID is listed when you run `tskmstr` or `tskmstr list [-i <id>]`


## Adding and Removing Labels

You can add and remove labels from a task using the tag add and tag remove commands:

```
tskmstr tags add <issue_id> tag1 tag2 tag3
tskmstr tags remove <issue_id> tag1 tag2 tag3

# example: 
tskmstr tags remove J/PROJ-2 this-label that-label another-label
```
## Listing Issue Stores

```
> tskmstr issue-stores
T - https://api.github.com/user/repos
🄿 - https://api.github.com/user/tskmstr-tasks
🅆 - https://gitlab.com/username%2Fsome-sub-repo
```

Use this to determine the `-i <id>` you need to use for `tskmstr add -i <id> <tile> <details> [<tag>...]`
## Command Reference

The full command help can be obtained with `--help`
* `list`: List all tasks/issues, grouped by labels and priority.
* `add <title> <details> [ tags,... ]`: Add a new task/issue to the default repository.
* `close <issue_id>`: Close a task/issue.
* `tags add <issue_id>`: Add tags to a task.
* `tags remove <issue_id>`: Remove tags from a task.
* `issue-stores`: list the configured issues-stores (repositories, todo lists)
* `jira-transitions` <ISSUE-ID> # special required for configuring jira

## Features

For current and upcoming (intended features, see the more detailed list here)

* [features roadmap](doc/features_roadmap.md)

* Planned Features - [tskmstr features](https://github.com/rbuckland/tskmstr/issues?q=is%3Aopen+is%3Aissue+label%3Afeature)


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
   
    For convenience, the **tskmstr** binary is just called "t".

    ```
    mkdir -p ~/.local/bin && cp ./target/release/t ~/.local/bin/t
    ```

5. Configure it        


## Contributing

Contributions to **tskmstr** are welcome! Please check out the [contribution guidelines](./Contributing.md) for more details.

## License

This project is licensed under the MIT License - see the LICENSE.md file for details.

Enjoy using **tskmstr** to stay organized and manage your tasks across multiple repositories! If you have any questions or encounter issues, feel free to reach out to our community. Happy task management!