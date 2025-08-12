# GYROS

Gyros is a simple multirepo tool, which helps running git commands in multiple repos at the same time.
It's probably overkill for most people/workflow but it's perfect for me! I use it everyday :sparkles:

## Installation

Gyros can be built from source. After cloning the repo do:

```bash
cd gyros
cargo install --path .
```

Don't forget to add `.cargo/bin` to your PATH:

### Bash / Zsh

```bash
export PATH=~/.cargo/bin:$PATH  
```

### Fish

```fish
set PATH $PATH ~/.cargo/bin
```

## Setup

Firstly you need to create a `.gyros.toml` file:

> `.gyros.toml`

```toml
[repos]
repo1 = "/home/pia/Dev/repo1"
repo2 = "/home/pia/Dev/repo2"
repo3 = "/home/pia/other/some/path/repo3"
```
The repo name can be an alias and will act as an identifier to the repository

Which would then execute git commands inside the 3 repositories from the folder where `.gyros.toml` was saved.
```bash 
/home/pia/
├── Dev/
│   ├── repo1
│   ├── repo2
│   └── .gyros.conf
└── other/
    └── some/
        └── path/
            └── repo3
```

For now, having multiple config file in the same folder leads to indeterminate behavior.

## Usage

The `user` command is just a pass-through to run normal git commands in your repos.

```bash 
gyros user diff --name-status
```

You can run a command in only one repository with the `--only repo_name` command:
```bash 
gyros user push -u dev --only repo1
```

The repo name is the one used as identifier in your `.gyros.toml`.

### Other Commands

For now, 2 other can be used with gyros, `fetch-all` and `pull-all`. More will be added in the future.

## Improvementsi :honeybee:

Here's what i want to add to Gyros:
- run commands in parallel (faster)
- more shortcut commands
- add custom commands in the conf file
