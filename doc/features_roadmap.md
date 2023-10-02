# Features Roadmap

| Service Provider | Create an Issue/Task | Close an Issue/Task | Add Tags/Labels | Remove Tags/Labels | Create a Note | Filtering | Support Multiple Repositories/Stores | Support Multiple Logins on Same Service |
|----|---|---|---|---|---|---|---|---|
| Github | ✅ | ✅ | ✅ |✅ | | ✅ |  ✅ |  ✅ |
| Gitlab | ✅ | ✅ | ✅ |✅ | | ✅ | ✅ |✅ |
| Jira | ✅ | ✅ | ✅ |✅ | | ✅ | ✅ |✅ |
| Office365 (Todo Items) | 
| Office365 (Teams Tasks) | 
| Google Tasks | 
| iCloud Tasks | 
| Android Tasks | 
| Android Notes | 


## Vendor Anomalies

### Gitlab

* It appers that labels filter cannot be **OR'd**. that is

    ```yaml
    filter: labels=abc,a123
    ```
    will only show labels that have `abc` AND `a123`

    If this really was a problem, then create to "repository" entries in the config for the same repo, and set he filter(s) appropriately.
    

## Comments

This tool does NOT replace the specific UI/CLI/App of each of the providers. It is just meant to aggregate.

For anything more complex, we suggest you use the dedicated UI or CLI tool of each solution.
- `glab` - Command Line tool for GitLab [gh CLI](https://github.com/cli/cli)
- `gh` - Command Line tool for GitHub - [glab CLI](https://docs.gitlab.com/ee/editor_extensions/gitlab_cli/)
- `go-jira` - 3rd Party Command Line tool for Jira.[go-jira CLI](https://github.com/go-jira/jira)
