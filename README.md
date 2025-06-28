# GYROS

## IDEAL WORKFLOW:

> gyros init
> gyros fetch-all
> gyros user -- git show --name-status
> gyros list
> gyros alias ~/dev/project/repo_with_cool_feature_v2 cool_feature_2
> gyros --only cool_feature_2 checkout -b better_feature


## TODO:

### 1 - Command Revolution

#### Refactor stdout stderr handling

Mainly to have a better setup when gyros will use parallelized tasks

- Capture and control output per repo
- Split stdout/stderr
- Structure output

#### Refactor CLI

- Subcommand Enum 
    - fetch-all
    - pull-all
    - user (custom funtions)
    - only flag

#### Better TOML config

- Combine Repo struct with assert_is_same_repo (asser_equal)
- Alias support for repo
- use --only flag with aliases 

### 2 - Intelligence tools

#### Dryrun setup

- [DRY-RUN]: blabkabla
- to test setup

#### Error tracking + summary

- Have a better output like stats on who failed who succeeded

### 3- Kachow

#### Parallel execution

- learn about frameworks (rayon, tokio, thread, ...)
- parallel flag

### 4 - Optional

#### Think about config

- overrides, conf, ...
- gyros init
- --confirm (or prompted with yes/no) for important tasks
