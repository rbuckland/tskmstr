# tskmstr Copilot Instructions

## Build & Run

```sh
# Requires stable Rust (rust-toolchain.toml pins `channel = "stable"`)
cargo build
cargo build --release

# Run directly (binary is named `tskmstr`, typically symlinked to `t`)
cargo run -- [args]

# Desktop tray/menu-bar widget (optional `tray` feature, binary `tskmstr-tray`)
cargo build --release --features tray
cargo run --features tray --bin tskmstr-tray -- [--open] [--refresh-secs N]

# Cross-compile (uses Cross.toml for OpenSSL setup)
cross build --target <target-triple> --release
```

The only automated tests are the icon-rendering unit tests in the tray binary
(`cargo test --features tray --bin tskmstr-tray`). Lint and format with:

```sh
cargo clippy
cargo fmt
```

## Architecture

`tskmstr` is a CLI tool that **aggregates issues/tasks** from GitHub, GitLab, and Jira into a single terminal view. It reads a YAML config file and routes all commands to the appropriate provider.

```
lib.rs           → Library crate root; exposes config/control/output/providers to both binaries
main.rs          → CLI arg parsing (clap), config loading, dispatches to do_work()
config.rs        → AppConfig (deserialized from YAML), IssueTaskRepository trait,
                   TaskIssueProvider enum (GitHub/GitLab/Jira), provider lookup logic,
                   config path resolution + load_config()
control.rs       → Business logic: add/close/comment/tag operations, delegates to providers
output.rs        → collect_all_tasks() aggregates issues from all providers; group_tasks()
                   groups by priority labels then by tag (shared with the tray widget);
                   renders colored terminal output
bin/tray/        → `tskmstr-tray` desktop widget (feature "tray"): eframe/egui panel,
                   tray-icon menu bar icon, background worker thread owning the tokio
                   runtime, tiny-skia icon renderer, `autostart` + hidden `render-icon`
                   subcommands
install.sh /     → One-line installers (`curl … | bash`, `irm … | iex`): download the latest
install.ps1        release asset, install CLI + tray, add `alias t=tskmstr`, run `tskmstr init`.
                   install.sh honours TSKMSTR_VERSION / TSKMSTR_DOWNLOAD_BASE (file:// works)
                   so it can be tested against locally built packages with a throwaway HOME.
packaging/       → Release packaging, runnable locally (see README "Packaging"):
                   macos/ (Info.plist, build-app.sh, build-dmg.sh), linux/ (nfpm.yaml.in
                   template, .desktop, build-packages.sh for deb/apk/tgz), windows/
                   (WiX v4 tskmstr.wxs for the per-user MSI)
providers/
  common/
    model.rs     → Canonical Issue + Label types (all providers map into these)
    credentials.rs → HasSecretToken trait — reads PAT/API tokens from OS keyring
  github/        → GitHub REST API: model, methods (collect/add/close/comment/label)
  gitlab/        → GitLab REST API: model, methods
  jira/          → Jira REST API: model, methods (uses JQL for filtering)
```

**Data flow for listing:** `output::aggregate_and_display_all_tasks` → `output::collect_all_tasks` calls each provider's `collect_tasks_from_*` → all results merged into `Vec<common::model::Issue>` → `output::group_tasks(issues, &DisplayOptions)` groups by priority labels first, then by remaining tags (or one flat list) → printed with `colored`. `DisplayOptions::from_config` bundles the priority labels, the `output_ordering` config (`grouped_by_tags`, `show_tag_heading`) and the store order; every group is sorted by store (config order) then issue number descending. The priority group is omitted when empty; with `show_tag_heading: false` the rest is one flat list. Each `TaskGroup` carries `show_heading` and `separator_before` so renderers never re-derive them. The tray widget uses the same `collect_all_tasks` + `group_tasks` so its panel matches `tskmstr list`.

**Tray widget (`src/bin/tray/`):** `worker.rs` runs a thread with its own tokio runtime that owns the `AppConfig`, refreshes tasks on an interval and performs closes (provider panics are caught and surfaced as errors). `app.rs` is the `eframe::App`: the window is undecorated, always-on-top and hidden until the tray icon is clicked; it hides again on focus loss. `tray.rs` builds the `tray-icon` (on Linux, inside a dedicated GTK thread). `display.rs` converts the icon's rect (which tray-icon reports in physical pixels of the icon's own screen) into global logical points; on macOS it uses `NSScreen` so mixed-DPI multi-monitor setups place the panel under the icon, and the app re-checks the position for a few frames after showing because the scale factor is only reliable once the window is mapped. `icon.rs` renders the icons with `tiny-skia` + `ab_glyph` using egui's embedded Ubuntu-Light font: on macOS a monochrome template image (solid disc with a knocked-out "t", then the count; width hugs the count; `Layout::is_template` drives `with_icon_as_template`), elsewhere a coloured square. The binary is gated behind `required-features = ["tray"]` so the default `cargo build` never pulls GUI dependencies; on Linux it needs `libgtk-3-dev libxdo-dev libappindicator3-dev`.

**Issue ID format:** `<store_id>/<issue_number>` (e.g., `🄿/14`, `J/ABC-123`). The `store_id` matches the `id:` field in config and is used throughout the codebase to route operations to the correct provider via `AppConfig::find_provider_for_issue`.

## Key Conventions

### Adding a new provider
1. Create `src/providers/<name>/` with `mod.rs`, `model.rs`, `methods.rs`
2. The config struct must implement `HasSecretToken` (for keyring credential access)
3. Each repository/project struct must implement `IssueTaskRepository` (provides `id()`, `color()`, `defaults()`)
4. Add a new variant to `TaskIssueProvider` enum in `config.rs`
5. Add `Vec<NewConfig>` field to `AppConfig` with `#[serde_inline_default(Vec::<NewConfig>::new())]`
6. Wire into `control.rs` match arms and `output.rs` aggregation

### Credentials
All API tokens are stored in the **OS keyring** (never in config files). Config only stores `service` and `username` fields that identify the keyring entry. Access via `HasSecretToken::get_token()`.

### `serde_inline_default`
Used extensively on config structs to provide field-level defaults (e.g., `endpoint`, `provider_id`, `default_issue_type`). Apply `#[serde_inline_default]` on the struct and `#[serde_inline_default(expr)]` on fields that need defaults.

### Stable Rust
The project builds on stable Rust; `rust-toolchain.toml` pins `channel = "stable"` and CI uses the stable toolchain. Do not reintroduce nightly-only `#![feature(...)]` gates.

### Config file location
- All platforms: `$HOME/.config/tskmstr/tskmstr.config.yml` (resolved in `config.rs` from `HOME`, falling back to `USERPROFILE` on Windows; no `directories` crate)
- Override with `--config <path>`
- `tskmstr init [--force]` writes a commented template config to that path. It is handled in `main()` before the config is loaded, so its `do_work` match arm is `unreachable!()`.

### `issue-stores` subcommands
`tskmstr issue-stores` is a subcommand group; with no subcommand it behaves as
`issue-stores list`. `list-providers` prints `AppConfig::providers()` (one row per
configured GitHub/GitLab/Jira credential + endpoint, keyed by `provider_id`).
`issue-stores add <shortcode> <provider_id> <target> [color]` calls
`config::add_issue_store`, which validates against the loaded `AppConfig` (unique id,
known provider, `colored::Color` name), then edits the YAML as a `serde_yaml::Value`
tree so unmodelled fields survive, re-parses the result as `AppConfig` before saving,
copies the old file to `<config>.bak` and rewrites it. Comments are not preserved.
`target` is `owner/repo` (GitHub), `group/project` URL-encoded to `%2F` (GitLab) or the
project key (Jira). `do_work` receives the resolved config path for this.

### `view` command
`tskmstr view <id>...` → `control::view_task` resolves the store from the id prefix and
calls the provider's `view_issue_*`, which fetches the single issue plus its comments and
maps them into `common::model::IssueDetail` (wraps an `Issue` + state, body, author,
timestamps, `Vec<Comment>`). `output::display_issue_detail` renders it. GitHub uses
`/issues/{n}` + `/comments`; GitLab `/issues/{iid}` + `/notes` (system notes dropped);
Jira `/rest/api/2/issue/{key}` with `fields=...,comment` (v2 so bodies are plain strings).
A new provider must supply a `view_issue_*` and a `view_task` match arm.

### Issue store colours
Each store's `color:` is carried on `Issue.color` by the provider collect functions and
used by `output::issue_id_color` (CLI) and the tray row renderer to paint the issue id.
The global `colors.issue_id` is only the fallback for stores without a valid colour.
When adding a provider, set `color: Some(store.color.clone())` when building `Issue`s.

### Releases and packaging
Releases are cut automatically by `.github/workflows/release.yml` on every push to `main`: convco computes the next semver from conventional commits (`--treat-major-zero-as-stable`, so `feat:` → minor, `fix:` → patch, `!`/`BREAKING CHANGE` → major; docs/chore/ci commits release nothing), the workflow commits `chore(release): vX.Y.Z` (via `packaging/set-version.sh`, which edits Cargo.toml and only the root entry of Cargo.lock), tags it, then builds and publishes a macOS universal DMG (.app + CLI), a Debian .deb, an Alpine .apk (musl build in `rust:alpine`, `continue-on-error`), Linux tarballs, and a Windows per-user MSI + zip, plus `SHA256SUMS`. Every package contains **both** binaries. A manually pushed `v*` tag or a `workflow_dispatch` (optionally with a version) also releases. Build jobs run `set-version.sh` themselves, so they do not depend on the bump commit having landed on `main` (branch protection may block the bot; a `RELEASE_TOKEN` PAT secret fixes that). Builds therefore must not use `--locked`. **Write conventional commit messages**: the type decides whether a release happens. Icons are not checked in: `tskmstr-tray render-icon --out-dir DIR` writes the PNG set, `.icns` and `.ico` at build time (`src/bin/tray/assets.rs`). Keep the `packaging/` scripts self-contained so they can be exercised without CI; `nfpm.yaml.in` is a template that `build-packages.sh` fills with `sed`.

### Commit and PR authorship
Do not add `Co-Authored-By:` trailers or any other AI/tool attribution (e.g. "Generated with Claude Code") to commit messages or pull request descriptions.

### Autostart
`tskmstr-tray autostart enable|disable|status` (`src/bin/tray/autostart.rs`) registers the widget with the platform login mechanism: a LaunchAgent plist labelled `com.thebuckland.tskmstr-tray` on macOS, the `HKCU\...\CurrentVersion\Run` value `tskmstr-tray` on Windows (the MSI writes the same value), and `~/.config/autostart/tskmstr-tray.desktop` on Linux. It shells out to `launchctl` / `reg` rather than adding crates. The tray binary is `windows_subsystem = "windows"` in release builds so it has no console window.

### Issue URLs
`Issue.html_url` must be a browsable URL (it is what `list --all` prints and what the tray widget opens). Jira's REST `self` link is not browsable, so the Jira provider builds `<endpoint>/browse/<KEY>` instead.

### Filtering
Each issue store supports a `filter:` string in config. GitHub/GitLab filters are appended as query parameters; Jira filters are appended to a base JQL query (`project={} AND resolution = unresolved AND <filter>`).
