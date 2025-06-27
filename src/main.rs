use colored::Colorize;
use serde_derive::Deserialize;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Output};
use clap::Parser;
use std::path::PathBuf;

// TODO:
//      - add status, log?, stash, fetch, add/commit?
//      - file to configure repos and fallback
//      - parallelize with rayon or tokio ==> think about how to stdout
//      - better errors
//      - option for verbosity (stdout, stderr, printf)
//      - context option if already in one repos
//      - interactive prompt ?
//      - customizable args

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

// ------------ RUNNERS ------------

fn safe_run(cmd: &mut Command) -> io::Result<Output> {
    match cmd.output() {
        Ok(output) => Ok(output),
        Err(e) => {
            eprintln!("{}", "Sorry.. Can't do that ;-; ~".red().italic());
            Err(e)
        }
    }
}

fn execute(cmd: Output) -> io::Result<()> {
    io::stdout().write_all(&cmd.stdout)?;
    io::stderr().write_all(&cmd.stderr)?;
    Ok(())
}

fn run_git_command(dir: &str, args: &[String]) -> io::Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(dir);
    let output = safe_run(&mut cmd)?;
    execute(output)
}

fn print_header(label: &str, repo: &str) {
    println!(
        "{}",
        format!("\n<>--------<> {} -> {} <>--------<>", label, repo)
            .green()
            .bold()
    );
}

// ------------ COMMANDS ------------

fn command(dir: &str, args: &Vec<String>) -> io::Result<()> {
    print_header(&args[0], dir);
    run_git_command(dir, &args)
} 

// ------------- TOML -------------

#[derive(Deserialize)]
struct Data {
    config: Config,
}

#[derive(Deserialize)]
struct Config {
    repos_list: Vec<String>,
}

fn load() -> io::Result<Vec<String>> {
    let cwd = env::current_dir()?;
    let conf_path = format!("{}/.gyros.toml", cwd.display());

    let contents = fs::read_to_string(conf_path)?;
    let data: Data = toml::from_str(&contents).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TOML parse error: {}", e),
        )
    })?;
    Ok(data.config.repos_list)
}

// ------------- REPO -------------

fn assert_is_repo(path: &str, repo: &str) -> bool {
    let folder = PathBuf::from(path);
    if let Some(folder_name) = folder.file_name() {
        folder_name == repo
    } else {
        false
    }
}

// ------------- RUN --------------

fn main() -> io::Result<()> {
    let repos_path = load()?;
    let args = Args::parse();

    let selected_repos: Vec<String> = match args.only {
        Some(repo) => {
            let filtered: Vec<String> = repos_path
                .into_iter()
                .filter(|r| assert_is_repo(&r, &repo))
                .collect();
            if filtered.len() > 0 {
                filtered
            } else {
                eprintln!("{}", format!("Coundn't find the repo named '{}' ;-;", repo).red().italic());
                std::process::exit(1);
            }
        },
        None => repos_path,
    };
    println!("{:?} -- {:?}", selected_repos, args.git_args);
    for r in selected_repos {
        command(&r, &args.git_args)?;
    }
    Ok(())
}
