# tskmstr Copilot Instructions

## Build & Run

```sh
# Requires nightly Rust (rust-toolchain.toml pins this automatically)
cargo build
cargo build --release

# Run directly (binary is named `tskmstr`, typically symlinked to `t`)
cargo run -- [args]

# Cross-compile (uses Cross.toml for OpenSSL setup)
cross build --target <target-triple> --release
```

There are no automated tests in the repository. Lint and format with:

```sh
cargo clippy
cargo fmt
```

## Architecture

`tskmstr` is a CLI tool that **aggregates issues/tasks** from GitHub, GitLab, and Jira into a single terminal view. It reads a YAML config file and routes all commands to the appropriate provider.

```
main.rs          → CLI arg parsing (clap), config loading, dispatches to do_work()
config.rs        → AppConfig (deserialized from YAML), IssueTaskRepository trait,
                   TaskIssueProvider enum (GitHub/GitLab/Jira), provider lookup logic
control.rs       → Business logic: add/close/comment/tag operations, delegates to providers
output.rs        → Aggregates issues from all providers, groups by priority labels then
                   by tag, renders colored terminal output
providers/
  common/
    model.rs     → Canonical Issue + Label types (all providers map into these)
    credentials.rs → HasSecretToken trait — reads PAT/API tokens from OS keyring
  github/        → GitHub REST API: model, methods (collect/add/close/comment/label)
  gitlab/        → GitLab REST API: model, methods
  jira/          → Jira REST API: model, methods (uses JQL for filtering)
```

**Data flow for listing:** `output::aggregate_and_display_all_tasks` → calls each provider's `collect_tasks_from_*` → all results merged into `Vec<common::model::Issue>` → grouped by priority labels first, then by remaining tags → printed with `colored`.

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

### Nightly Rust
The project uses `#![feature(fn_traits)]` and `#![feature(unboxed_closures)]` for the `find_by` closure pattern in `config.rs`. The `rust-toolchain.toml` pins `channel = "nightly"` — this is intentional.

### Config file location
- Linux: `~/.config/tskmstr/tskmstr.config.yml`
- macOS: `~/Library/Preferences/tskmstr/tskmstr.config.yml`
- Windows: `%LOCALAPPDATA%/tskmstr/tskmstr.config.yml`
- Override with `--config <path>`

### Filtering
Each issue store supports a `filter:` string in config. GitHub/GitLab filters are appended as query parameters; Jira filters are appended to a base JQL query (`project={} AND resolution = unresolved AND <filter>`).
