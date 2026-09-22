# tskmstr Copilot Instructions

## Build & Run

```sh
# Requires stable Rust (rust-toolchain.toml pins `channel = "stable"`)
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

### Stable Rust
The project builds on stable Rust; `rust-toolchain.toml` pins `channel = "stable"` and CI uses the stable toolchain. Do not reintroduce nightly-only `#![feature(...)]` gates.

### Config file location
- All platforms: `$HOME/.config/tskmstr/tskmstr.config.yml` (resolved in `main.rs` from the `HOME` env var; no `directories` crate)
- Override with `--config <path>`
- `tskmstr init [--force]` writes a commented template config to that path. It is handled in `main()` before the config is loaded, so its `do_work` match arm is `unreachable!()`.

### Filtering
Each issue store supports a `filter:` string in config. GitHub/GitLab filters are appended as query parameters; Jira filters are appended to a base JQL query (`project={} AND resolution = unresolved AND <filter>`).
