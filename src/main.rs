use clap::{Parser, Subcommand};
use colored::Colorize;
use core::str;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write, stderr, stdout};
use std::process::Command;

// ---------- M A C R O -----------

macro_rules! eprintln_red {
    ($($a:tt)*) => {
        eprintln!("{}", format!($($a)*).red().italic())
    };
}

// ------------ CLI -------------

#[derive(Parser, Debug)]
#[command(name = "gyros")]
#[command(about = "Run git commands in multiple repos", long_about = None)]
struct Args {
    /// Run the command in the specified repo only
    #[arg(long, global = true)]
    only: Option<String>,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// FetchAll all repos
    FetchAll,
    /// PullAll all repos
    PullAll,
    /// Run a git command on all repos
    User {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        trail: Vec<String>,
    },
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
    let mut log = Log::new(repo);

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
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(&repo.path);
    Ok(logged_run(&mut cmd, &repo.path))
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
        io::Error::new(io::ErrorKind::InvalidData, format!("TOML parse error: {e}"))
    })?;
    if data.repos.is_empty() {
        return Err(io::Error::new(
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
    alias: String,
    path: String,
}

impl Repo {
    fn new(alias: &str, path: &str) -> Self {
        Self {
            alias: alias.to_owned(),
            path: path.to_owned(),
        }
    }
}

// ------------- RUN --------------

fn get_header(label: &str) -> String {
    format!("{}", format!(":: {label}\n").purple().bold())
}

fn filter_repos(list_of_repos: Vec<Repo>, repo: &str) -> Vec<Repo> {
    let filtered: Vec<Repo> = list_of_repos
        .into_iter()
        .filter(|x| x.alias == repo)
        .collect();
    if !filtered.is_empty() {
        filtered
    } else {
        eprintln_red!("Failed to find this repo: '{}' ;-;", repo);
        std::process::exit(1);
    }
}

fn summary(success_count: usize, failure_count: usize) -> io::Result<()> {
    if failure_count > 0 {
        writeln!(
            stderr(),
            "{}",
            format!("\nDone :: {success_count} succeeded - {failure_count} failed\n")
                .red()
                .italic()
        )?;
    } else {
        writeln!(
            stdout(),
            "{}",
            format!("\nDone :: {success_count} succeeded - {failure_count} failed\n")
                .green()
                .italic()
        )?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let list_of_repos = load()?;

    // Filter repos depending on the presence of only tag
    let selected_repos: Vec<Repo> = match args.only {
        Some(repo) => filter_repos(list_of_repos, &repo),
        None => list_of_repos,
    };

    // Parse commands
    let git_args = match args.command {
        Cmd::FetchAll => vec!["fetch".to_owned()],
        Cmd::PullAll => vec!["pull".to_owned()],
        Cmd::User { trail } => trail,
    };

    let mut success_count = 0;
    let mut failure_count = 0;

    // Run the command on each filtered repo
    for repo in selected_repos {
        match run_git_command(&repo, &git_args) {
            Ok(log) => {
                if let Err(e) = log.display() {
                    eprintln_red!("Display failed: {}", e);
                }
                if !log.success {
                    failure_count += 1;
                    eprintln_red!("Command failed in: '{}'", repo.path);
                } else {
                    success_count += 1;
                }
            }
            Err(e) => {
                eprintln_red!("Run failed: {}", e);
            }
        }
    }
    summary(success_count, failure_count)?;
    Ok(())
}
