# Creating a New Provider

Start by creating a module for each Todo Provider. Each module will have its own subdirectory containing its internal models, methods, configuration, and authentication logic. For example:

```
src
├── config.rs
├── control.rs
├── main.rs
├── output.rs
└── providers
    ├── common
    │   ├── model.rs
    │   └── mod.rs
    ├── github
    │   ├── auth.rs
    │   ├── methods.rs
    │   ├── model.rs
    │   └── mod.rs
    ├── gitlab
    │   ├── auth.rs
    │   ├── methods.rs
    │   ├── model.rs
    │   └── mod.rs
    └── mod.rs
```
* `providers/<provider>/model.rs` - Define Internal Models:

    In each provider module (github, gitlab), define the internal models that represent the data structures specific to that provider. For example, the model.rs file in the github module will define the GitHub-specific structs.

* `providers/<provider>/methods.rs` - Define Methods:

    Create a methods.rs file within each provider module to define methods for reading, creating, closing, and updating todo items for that provider. These methods will use the internal models defined in the same module.

* `providers/<provider>/auth.rs` - Authentication and Configuration:

    Define a separate auth.rs file within each provider module to handle authentication related to that provider. Also, each module can have its own configuration structure. For instance, the github module can have a GitHubConfig structure and the gitlab module can have a GitLabConfig structure.

* `providers/common/model.rs` -Common Models:

    If there are common data structures or models that are shared among different providers, you can define them in a separate common module. For example, if both GitHub and GitLab use similar label structures, you can define them in common/model.rs.


### ChatGPT Advice

Here's a high-level overview of what each file could contain:

* `providers/github/model.rs` Define GitHub-specific data structures.

* `providers/github/methods.rs` Implement methods for interacting with GitHub's API.

* `providers/github/auth.rs` Implement authentication logic specific to GitHub.

* `providers/gitlab/model.rs` Define GitLab-specific data structures.

* `providers/gitlab/methods.rs` Implement methods for interacting with GitLab's API.

* `providers/gitlab/auth.rs` Implement authentication logic specific to GitLab.

* `common/model.rs` Define common data structures shared among providers.

* `main.rs` Import and use the different provider modules.

By organizing your code in this way, you can easily add new providers in the future without cluttering your main codebase. Each provider's logic will be encapsulated within its module, making it easier to understand, maintain, and extend your application.