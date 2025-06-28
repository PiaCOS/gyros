use clap::Parser;
use colored::Colorize;
use core::str;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

// ------------ CLI -------------

#[derive(Parser, Debug)]
#[command(name = "gyros")]
#[command(about = "Run git commands in multiple repos", long_about = None)]
struct Args {
    /// Only run command in this repo (partial match)
    #[arg(long)]
    only: Option<String>,

    /// The git subcommand and args, e.g. status, log, etc.
    #[arg(required = true, trailing_var_arg = true)]
    git_args: Vec<String>,
}

// ------------ LOG -------------

struct Log {
    stdout: String,
    stderr: String,
    success: bool,
    label: String,
}

impl Log {
    fn new(label: &str) -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
            success: true,
            label: label.to_owned(),
        }
    }

    fn display(&self) -> io::Result<()> {
        io::stdout().write_all(get_header(&self.label).as_bytes())?;
        io::stdout().write_all(self.stdout.as_bytes())?;
        io::stderr().write_all(self.stderr.as_bytes())?;
        Ok(())
    }
}

// ------------ RUNNERS ------------

fn logged_run(cmd: &mut Command, repo: &str) -> Log {
    let mut log = Log::new(&repo.to_owned());

    match cmd.output() {
        Ok(output) => {
            log.success = output.status.success();
            log.stdout = String::from_utf8_lossy(&output.stdout).to_string();
            log.stderr = String::from_utf8_lossy(&output.stderr).to_string();
        }
        Err(e) => {
            log.success = false;
            log.stderr = e.to_string();
        }
    }
    log
}

fn run_git_command(repo: &Repo, args: &[String]) -> io::Result<Log> {
    if let Some(path) = &repo.path {
        let mut cmd = Command::new("git");
        cmd.args(args).current_dir(&path);
        Ok(logged_run(&mut cmd, &path))
    } else {
        Err(to_io_err(
            io::ErrorKind::Other,
            "There is no path to run the commannd".to_owned(),
        ))
    }
}

// Do I need that ? Should i put it elsewhere ?
fn to_io_err(kind: io::ErrorKind, text: String) -> io::Error {
    io::Error::new(kind, text)
}

// ------------- TOML -------------

#[derive(Deserialize)]
struct Data {
    repos: HashMap<String, String>,
}

fn load() -> io::Result<Vec<Repo>> {
    let cwd = env::current_dir()?;
    let conf_path = format!("{}/.gyros.toml", cwd.display());

    let contents = fs::read_to_string(conf_path)?;
    let data: Data = toml::from_str(&contents).map_err(|e| {
        to_io_err(
            io::ErrorKind::InvalidData,
            format!("TOML parse error: {}", e),
        )
    })?;
    if data.repos.is_empty() {
        return Err(to_io_err(
            io::ErrorKind::InvalidData,
            "No repos found in .gyros.toml".to_owned(),
        ));
    }
    Ok(data
        .repos
        .iter()
        .map(|(alias, path)| Repo::new(alias, path))
        .collect::<Vec<Repo>>())
}

// ------------- REPO -------------

struct Repo {
    alias: Option<String>,
    path: Option<String>,
}

impl Repo {
    fn new(alias: &str, path: &str) -> Self {
        Self {
            alias: Some(alias.to_owned()),
            path: Some(path.to_owned()),
        }
    }

    fn from_name(name: &str) -> Self {
        Self {
            alias: alias_from_name(name),
            path: None,
        }
    }

    fn assert_equal(&self, repo: &Repo) -> bool {
        if let Some(alias1) = &self.alias
            && let Some(alias2) = &repo.alias
        {
            alias1 == alias2
        } else {
            false
        }
    }
}

fn alias_from_name(name: &str) -> Option<String> {
    PathBuf::from(name).file_name()?.to_str().map(str::to_owned)
}

// ------------- RUN --------------

fn get_header(label: &str) -> String {
    format!(
        "{}",
        format!("\n<>--------<> {} <>--------<>\n", label)
            .green()
            .bold()
    )
}
fn main() -> io::Result<()> {
    let list_of_repos = load()?;
    let args = Args::parse();

    // Filter repos depending on the presence of only tag
    let selected_repos: Vec<Repo> = match args.only {
        Some(name) => {
            let repo = Repo::from_name(&name);
            let filtered: Vec<Repo> = list_of_repos
                .into_iter()
                .filter(|x| x.assert_equal(&repo))
                .collect();
            if filtered.len() > 0 {
                filtered
            } else {
                eprintln!(
                    "{}",
                    format!("Coundn't find the repo named '{}' ;-;", name)
                        .red()
                        .italic()
                );
                std::process::exit(1);
            }
        }
        None => list_of_repos,
    };
    for repo in selected_repos {
        match run_git_command(&repo, &args.git_args) {
            Ok(log) => {
                log.display()?;
                if !log.success {
                    eprintln!("'{}' failed", repo.path.unwrap_or("It".to_owned()));
                }
            }
            Err(e) => {
                eprintln!("Couldn't even run: {}", e);
            }
        }
    }
    Ok(())
}
