# tskmstr

**Aggregates all tasks/issues together, across your entire world**


<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [tskmstr](#tskmstr)
  - [Installation](#installation)
  - [Configuration](#configuration)
  - [Usage](#usage)
    - [Adding a Task](#adding-a-task)
  - [Closing a Task](#closing-a-task)
  - [Listing Tasks](#listing-tasks)
  - [Adding and Removing Labels](#adding-and-removing-labels)
  - [Command Reference](#command-reference)
  - [Contributing](#contributing)
  - [License](#license)

<!-- /code_chunk_output -->


**tskmstr** is a versatile command-line tool designed to help you manage and organize your tasks and issues across various repositories on GitHub and GitLab. With tskmstr, you can efficiently interact with your tasks, categorize them with labels, and view them in a well-structured table format.




## Installation

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

    ```
    mkdir -p ~/.local/bin && cp ./target/release/tskmstr ~/.local/bin/t
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
    - owner: yourgithub_org
      repo: github_repo
      default: true
    - owner: other_github_org
      repo: github_repo2

gitlab.com:
  credential:
    service: gitlab.com
    username: key_username_in_keyring
  repositories:
    - project_id: group%2Fsubgroup%2Frepo

```

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

Example output looks like
```
Priority: now, urgent, todo
----------------------------------------
 - gh0/54 analyze rock formations (urgent, geology, research)
 - gh0/53 study sedimentary layers (urgent, geology, analysis)
 - gl0/18 survey geological fault lines (urgent, geology)
 - gl0/8 analyze soil composition (urgent, geology)
 - gl0/7 investigate geological formations (urgent, geology)
 
Tag: <no labels>
----------------------------------------
 - gh0/16 geological survey of mountain ranges ()
 - gh1/14 Study Earth's crust composition ()
 - gh1/13 Analyze rock strata for fossils ()
 - gh1/1 Study geological time periods ()
 - gl0/19 Volcanic activity observation ()
 - gl0/6 Study seismic fault lines ()

Tag: car
----------------------------------------
 - gh0/48 organise monthly debit for car wash of ute (car)
 - gh0/47 re-order instruments in the rear tray (car)

Tag: helpful
----------------------------------------
 - gl0/14 Geological research library (helpful)

Tag: client-3112
----------------------------------------
 - gh0/30 analyze soil quality for gardening (client-3112)
 - gh0/29 geological assessment of backyard (client-3112)
 - gh0/28 Soil stability testing (client-3112)
 - gh0/27 Foundation rock type analysis (client-3112)
 - gh0/26 Geological inspection of basement (client-3112)
 - gh0/25 Geological assessment of attic (client-3112)
 
Tag: hr3
----------------------------------------
 - gl0/17 Geological documentation for HR3 project (hr3)
 - gl0/16 Geological panel report (hr3)

Tag: renovations
----------------------------------------
 - gh0/20 Geological assessment for pool excavation (renovations)

```

Each of the "repositories" is numbered, `<gl><nnn>/<issue_id>` or `<gh><nnn>/<issue_id>`


## Usage
### Adding a Task

To add a new task to your default repository, use the add command:

```
tskmstr add "Task Title" "Task Details" --tags tag1 tag2 tag3
``````

This command adds a new task with the specified title, details, and tags.
It will add it to the `default`, which is set in the config.

Use this form when adding a task to a specified repository.

```
t add  <title> <details> -- gh2
```

## Closing a Task

To close a task, use the close command:

```
tskmstr close <issue_id>
```

Replace <issue_id> with the ID of the task you want to close. (e.g. `gl0/22`, `gh2/444`)

## Listing Tasks

To list all your tasks, grouped by labels and priority, simply run:

```
tskmstr
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

## Contributing

Contributions to tskmstr are welcome! Please check out the [contribution guidelines](./Contributing.md) for more details.

## License

This project is licensed under the MIT License - see the LICENSE.md file for details.

Enjoy using tskmstr to stay organized and manage your tasks across multiple repositories! If you have any questions or encounter issues, feel free to reach out to our community. Happy task management!