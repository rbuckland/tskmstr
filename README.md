# tskmstr

**Aggregates all tasks/issues together, across your entire world**

> I wrote this to better manage my world, and view across all aspects of my work and personal endeavours.

**tskmstr** is not SaaS, it uses the existing issue stores you intreract with.

<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [tskmstr](#tskmstr)
  - [Installation](#installation)
    - [Quick install](#quick-install)
    - [Manual install](#manual-install)
    - [macOS](#macos)
    - [Linux](#linux)
    - [Windows](#windows)
  - [Configuration](#configuration)
  - [Terminology](#terminology)
  - [Usage](#usage)
    - [Listing Tasks](#listing-tasks)
    - [Adding a Task](#adding-a-task)
    - [Closing a Task](#closing-a-task)
    - [Adding and Removing Labels](#adding-and-removing-labels)
    - [Viewing a Task](#viewing-a-task)
    - [Listing Issue Stores](#listing-issue-stores)
    - [Adding Issue Stores](#adding-issue-stores)
    - [Output Ordering](#output-ordering)
    - [Filtering](#filtering)
  - [Tray Widget](#tray-widget)
    - [Starting the tray widget at login](#starting-the-tray-widget-at-login)
    - [Building the widget from source](#building-the-widget-from-source)
    - [Packaging](#packaging)
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

### Quick install

**macOS / Linux**

```sh
curl -fsSL https://raw.githubusercontent.com/rbuckland/tskmstr/main/install.sh | bash
```

**Windows** (PowerShell)

```powershell
irm https://raw.githubusercontent.com/rbuckland/tskmstr/main/install.ps1 | iex
```

Both install the `tskmstr` CLI **and** the `tskmstr-tray` widget from the latest release, add
`alias t=tskmstr` to your shell (`~/.bashrc` / `~/.zshrc` / fish, or your PowerShell `$PROFILE`;
`t.exe` is installed too), and write a config template if you have none. Nothing needs
administrator rights. Options: `TSKMSTR_VERSION=v0.6.3` pins a version; on macOS/Linux
`TSKMSTR_BIN_DIR` (default `~/.local/bin`) and `TSKMSTR_AUTOSTART=1` (register the widget to start at
login) are honoured. The Windows MSI registers start-at-login by default.

### Manual install

Every release ships **both** the `tskmstr` CLI and the `tskmstr-tray` desktop widget,
packaged for each OS. Download from [Releases](https://github.com/rbuckland/tskmstr/releases)
(a `SHA256SUMS` file is published alongside).

| Platform | Package | Contents |
|----------|---------|----------|
| macOS (Intel + Apple Silicon) | `tskmstr-<v>-macos-universal.dmg` | `tskmstr-tray.app` + `tskmstr` CLI |
| Debian / Ubuntu | `tskmstr_<v>_amd64.deb` | `/usr/bin/tskmstr`, `/usr/bin/t`, `/usr/bin/tskmstr-tray`, desktop entry |
| Alpine | `tskmstr_<v>_x86_64.apk` | same layout as the .deb |
| Any Linux (glibc) | `tskmstr-<v>-linux-amd64.tar.gz` | both binaries + desktop entry + icon |
| Linux arm64 | `tskmstr-<v>-linux-arm64-cli.tar.gz` | CLI only |
| Windows | `tskmstr-<v>-windows-x64.msi` | per-user installer: both `.exe`s, `t.exe` alias, PATH, Start Menu, start-at-login |
| Windows | `tskmstr-<v>-windows-x64.zip` | both `.exe`s + `t.exe`, no installer |

### macOS

1. Open the `.dmg` and drag **tskmstr-tray.app** to *Applications*.
2. Copy the `tskmstr` CLI onto your PATH, e.g.

   ```sh
   cp /Volumes/tskmstr/tskmstr /usr/local/bin/tskmstr
   ln -s /usr/local/bin/tskmstr /usr/local/bin/t
   ```
   (the CLI is also inside the bundle at `tskmstr-tray.app/Contents/MacOS/tskmstr`).
3. The binaries are not notarised. If macOS refuses to open the app, right-click it and choose
   *Open*, or run `xattr -dr com.apple.quarantine /Applications/tskmstr-tray.app`.
4. Configure ([below](#configuration)) and optionally [start the widget at login](#starting-the-tray-widget-at-login).

### Linux

```sh
# Debian / Ubuntu
sudo apt install ./tskmstr_<v>_amd64.deb          # pulls in GTK 3 + AppIndicator

# Alpine
sudo apk add --allow-untrusted ./tskmstr_<v>_x86_64.apk

# Tarball
tar xzf tskmstr-<v>-linux-amd64.tar.gz
install -m 0755 tskmstr-<v>-linux-amd64/tskmstr tskmstr-<v>-linux-amd64/tskmstr-tray ~/.local/bin/
ln -sf ~/.local/bin/tskmstr ~/.local/bin/t
```

The tray widget needs GTK 3 and an AppIndicator library at runtime
(`libgtk-3-0 libxdo3 libayatana-appindicator3-1` on Debian/Ubuntu; the .deb and .apk declare
these dependencies). On GNOME, install the *AppIndicator and KStatusNotifierItem Support*
extension so tray icons are shown.

### Windows

Run `tskmstr-<v>-windows-x64.msi`. It installs per-user (no administrator rights) into
`%LOCALAPPDATA%\Programs\tskmstr`, adds that folder to your PATH (so `tskmstr` and `t` work in a
new terminal), creates a Start Menu shortcut and registers the tray widget to start at login
(the *Start the tray widget at login* feature; deselect it in a custom install, or run
`tskmstr-tray autostart disable` afterwards).

Prefer no installer? Unzip `tskmstr-<v>-windows-x64.zip` anywhere and add the folder to your PATH.

## Configuration

Before using **tskmstr**, you need to configure it with your GitHub, Jira and/or GitLab credentials. **tskmstr** reads your credentials securely from your OS keyring.

Refer to the comprehensive sample configuration [sample-config](sample/tskmstr.config.yml), that provides a good set of examples.

The config file lives at `~/.config/tskmstr/tskmstr.config.yml` on all platforms
(Linux, macOS and Windows). You can override the location with `--config <path>`.

1. Create the config file. The quickest way is to let **tskmstr** write a commented template for you:

    ```sh
    tskmstr init          # creates ~/.config/tskmstr/tskmstr.config.yml
    tskmstr init --force  # overwrite an existing file
    ```

    Then edit it, removing the provider sections you don't use. A complete example looks like this:

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

    # optional, defaults shown
    output_ordering:
      grouped_by_tags: true
      ordered_by_provider: false
      show_tag_heading: true

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

**tskmstr** never stores tokens in the config file. Each `credential:` block names a
keyring entry by `service` and `username`, and the token is read from your OS keyring
at runtime. Store one entry per provider, using the same `service` and `username`
values you put in the config.

**macOS** (built-in `security` tool, writes to the login Keychain):

```sh
security add-generic-password -U -s github.com -a <username> -w
security add-generic-password -U -s gitlab.com -a <username> -w
security add-generic-password -U -s <your-jira-instance>.atlassian.net -a user@example.com -w
```

With `-w` and no value, `security` prompts for the token so it stays out of your shell
history. `-U` updates the entry if it already exists.

**Linux** (GNOME Keyring / KWallet via `secret-tool`, from `libsecret-tools`):

```sh
secret-tool store --label='tskmstr github' service github.com username <username>
secret-tool store --label='tskmstr gitlab' service gitlab.com username <username>
secret-tool store --label='tskmstr jira' service <your-jira-instance>.atlassian.net username user@example.com
```

**Any platform** (Python `keyring` CLI, on Ubuntu from `python3-keyring`):

```sh
keyring set github.com <username>
keyring set gitlab.com <username>
keyring set <your-jira-instance>.atlassian.net user@example.com
```

For GitHub and GitLab, `<username>` is just the lookup key for the keyring entry and
does not have to be your account name. For Jira it must be the email you log in with,
because it is sent along with the API token for authentication.

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

### Commenting on a task

```
❯ t comment J/ITA-9 "fixed issue, now testing" 
Comment added successfully.
```

### Adding and Removing Labels

You can add and remove labels from a task using the tag add and tag remove commands:

```
tskmstr tags add <issue_id> tag1 tag2 tag3
tskmstr tags remove <issue_id> tag1 tag2 tag3

# example: 
tskmstr tags remove J/PROJ-2 this-label that-label another-label
```
### Viewing a Task

`view` shows everything about one (or more) tasks: title, URL, state, labels, author,
timestamps, the description, and the comments in chronological order.

```
> tskmstr view P/78
P/78 Libraries and software topics to learn
https://github.com/rbuckland/tskmstr-tasks/issues/78

State:   open
Labels:  software
Author:  rbuckland
Created: 2023-10-22T22:08:56Z
Updated: 2023-10-22T22:13:09Z

Python
- [ ] Xgboost
- [ ] Plot9

Comments (1)
----------------------------------------
[rbuckland @ 2023-11-02T09:14:00Z]
Also look at seaborn

> tskmstr view P/78 J/PROJ-12     # several at once, separated by a rule
```

GitLab system notes ("added label ...") are filtered out; Jira descriptions and comments
are shown as the plain text / wiki markup the v2 REST API returns.

### Listing Issue Stores

```
> tskmstr issue-stores
T - https://api.github.com/uation ser/repos
🄿 - https://api.github.com/user/tskmstr-tasks
🅆 - https://gitlab.com/username%2Fsome-sub-repo
```

Use this to determine the `-i <id>` you need to use for `tskmstr add -i <id> <tile> <details> [<tag>...]`

### Adding Issue Stores

New repositories / projects can be added to an existing provider (a configured
credential + endpoint) from the command line, without editing the YAML by hand.

First find the `provider_id` to add the store under:

```
> tskmstr issue-stores list-providers
github/rbuckland_sqce  github  https://api.github.com  [T, 🄿]
gitlab/rbuckland       gitlab  https://gitlab.com      [🅆]
My Jira                jira    https://sqc.atlassian.net  [J]
```

Then add the store, giving it a short unique id (the prefix used in issue ids), the
provider, the repository/project and optionally a color (default `white`):

```
tskmstr issue-stores add <shortcode> <provider_id> <target> [<color>]

# GitHub: <owner>/<repo>
tskmstr issue-stores add SPG github/rbuckland_sqce sqc-internal/pretty_goat blue

# GitLab: <group>/<project> (URL-encoded for you) or a numeric project id
tskmstr issue-stores add W2 gitlab/rbuckland mygroup/sub/project green

# Jira: the project key
tskmstr issue-stores add OPS "My Jira" OPS "bright yellow"
```

The config file is rewritten in place and the previous version is kept as
`tskmstr.config.yml.bak`. Because the file is re-serialised, YAML comments are dropped;
`defaults:` and `filter:` for the new store can be added by editing the file afterwards.

### Output Ordering

By default `tskmstr list` shows the priority-labelled issues first, then one group per
distinct set of tags, each under a `Tag: ...` heading. The optional `output_ordering`
section changes that layout (the tray widget follows the same settings):

```yaml
output_ordering:
  grouped_by_tags: true       # default
  ordered_by_provider: false  # default
  show_tag_heading: true      # default
```

| Setting | `true` | `false` |
|---------|--------|---------|
| `grouped_by_tags` | One group per tag set, sorted by name | One flat list of all non-priority issues |
| `ordered_by_provider` | Within each group, issues are kept together by issue store, in config-file order (GitHub, then GitLab, then Jira) | Issues stay in the order the providers returned them |
| `show_tag_heading` | `Tag: a, b` heading and divider above each group | Headings and dividers omitted; the tags are still shown after each title |

The `Priority:` heading is always shown. For example, a compact single list with issues
clustered by store:

```yaml
output_ordering:
  grouped_by_tags: false
  ordered_by_provider: true
  show_tag_heading: false
```

### Filtering

In the configuration you can set static filters for each issue store (project, repository).

This can be used when 
- you only need to see your personal tickets
- you only need to see issues related to your team
- you only want to see issues that are related to a phase in your workflow

The filtering utilises the underlying providers extra query parameters 

| Provider | Filtering Technique | Documentation | Example / Notes   |
|----------|---------------------|---------------|------------|
| GitHub   | Query Parameters    | [REST API Issues Query Params](https://docs.github.com/en/rest/issues/issues?apiVersion=2022-11-28#list-repository-issues--parameters)              | Example:<br/>`filter: labels=team-x,team-support`<br/>`filter: assignee=username`<br/>`filter: assignee=username&labels=support`
| GitLab   | Query Parameters    | [REST API Issues Query Params](https://docs.gitlab.com/ee/api/issues.html)              | Example:<br/>`filter: labels=team-x,team-support`<br/>`filter: assignee_username=username`<br/>`filter: assignee_username=username&labels=support` <br/>Labels are AND'd not OR'd
| Jira     | JQL                 | [JQL](https://support.atlassian.com/jira-software-cloud/docs/jql-operators/) |  The filter is appended to <br/> `project={} AND resolution = unresolved` <br/>Example:<br/>`filter: labels in (label2, label9) AND assignee = currentUser() ` |

Filtering is perhaps the core feature you will want. The idea being, at the CLI you just want to know what YOU need to do today. 

## Tray Widget

`tskmstr-tray` is a small system tray / menu bar widget (macOS, Windows, Linux) that
sits alongside the CLI and reads the same config file and keyring credentials.

* The tray icon is a **(t) nn** pill showing the number of open tasks.
* Click the icon (or choose *Show tasks* from its menu) to drop down the task panel.
  Tasks are grouped exactly as `tskmstr list` shows them: the priority group first,
  then one group per tag set.
* Click a task title to open it in your browser.
* Click the **[x]** to the right of a task to mark it done (closes the issue, same as
  `tskmstr close <id>`).
* The panel hides when it loses focus. Right-click the icon for *Refresh now* and *Quit*.

### Starting the tray widget at login

The widget can register itself with the OS login mechanism:

```sh
tskmstr-tray autostart enable     # start at login
tskmstr-tray autostart status
tskmstr-tray autostart disable
```

`--config` and `--refresh-secs` given on the `enable` command are baked into the autostart
entry. What it does per platform, and how to do the same by hand:

**macOS** — writes a LaunchAgent to `~/Library/LaunchAgents/com.thebuckland.tskmstr-tray.plist`
and loads it with `launchctl`. Manual equivalent (adjust the path if the app is elsewhere):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
    <key>Label</key><string>com.thebuckland.tskmstr-tray</string>
    <key>ProgramArguments</key>
    <array><string>/Applications/tskmstr-tray.app/Contents/MacOS/tskmstr-tray</string></array>
    <key>RunAtLoad</key><true/>
</dict></plist>
```

```sh
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.thebuckland.tskmstr-tray.plist
```

Alternatively add `tskmstr-tray.app` under *System Settings → General → Login Items*.

**Windows** — sets `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\tskmstr-tray` to the
executable path. The MSI installer does this for you by default. Manual equivalents: run
`shell:startup` from Win+R and drop a shortcut to `tskmstr-tray.exe` in the folder that opens,
or toggle it under *Settings → Apps → Startup* once registered.

**Linux** — writes an XDG autostart entry to `~/.config/autostart/tskmstr-tray.desktop`
(honoured by GNOME, KDE, XFCE, …). Manual equivalent:

```ini
[Desktop Entry]
Type=Application
Name=tskmstr tray
Exec=tskmstr-tray
Icon=tskmstr-tray
Terminal=false
```

### Building the widget from source

The widget is behind the `tray` cargo feature (it is not part of the default build):

```sh
cargo build --release --features tray
./target/release/tskmstr-tray            # sits in the tray; refreshes every 5 minutes
./target/release/tskmstr-tray --open     # also show the panel immediately
./target/release/tskmstr-tray --refresh-secs 60 --config ~/other.config.yml
```

On Linux the widget needs GTK and AppIndicator development libraries to build
(`sudo apt install libgtk-3-dev libxdo-dev libayatana-appindicator3-dev` on Debian/Ubuntu), and
because AppIndicator icons do not report clicks, use the *Show tasks* menu item to open
the panel. On macOS the first refresh may ask you to allow `tskmstr-tray` to read the
API token from your keychain.

### Packaging

`packaging/` holds everything the release pipeline uses, runnable locally:

```sh
cargo build --release --features tray
./target/release/tskmstr-tray render-icon --out-dir assets          # .png set, .icns, .ico
packaging/macos/build-app.sh 0.6.3 target/release/tskmstr-tray target/release/tskmstr assets/tskmstr.icns staging
packaging/macos/build-dmg.sh staging tskmstr-0.6.3-macos-universal.dmg
packaging/linux/build-packages.sh 0.6.3 amd64 "$PWD/target/release" "$PWD/assets" "$PWD/dist" deb apk tgz   # needs nfpm
wix build packaging/windows/tskmstr.wxs -d Version=0.6.3 -d BinDir=target/release -d IcoFile=assets/tskmstr.ico -o tskmstr.msi
```

## Command Reference

The full command help can be obtained with `--help`
* `list`: List all tasks/issues, grouped by labels and priority.
* `add <title> <details> [ tags,... ]`: Add a new task/issue to the default repository.
* `close <issue_id>`: Close a task/issue.
* `view <issue_id>...`: Show the full detail (description, comments) of one or more tasks/issues.
* `comment <issue_id> <comment>`: Add a comment to a task/issue
* `tags add <issue_id>`: Add tags to a task.
* `tags remove <issue_id>`: Remove tags from a task.
* `issue-stores [list]`: list the configured issues-stores (repositories, todo lists)
* `issue-stores list-providers`: list the configured providers (credential + endpoint) stores can be added to
* `issue-stores add <shortcode> <provider_id> <owner/repo|project_key> [<color>]`: add an issue store to the config file
* `init [--force]`: write a template config file to `~/.config/tskmstr/tskmstr.config.yml`
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
    cargo build --release                  # CLI only
    cargo build --release --features tray  # CLI + tskmstr-tray desktop widget
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