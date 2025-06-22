use clap::{Parser, Subcommand};
use colored::Colorize;
use serde_derive::Deserialize;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Diff on all repos
    Diff,
    /// Show on all repos
    Show,
    /// Pull on all repos
    Pull,
    /// Checkout on both repos or fallback to master (give me a branch name)
    Checkout { branch: String },
    /// Grep branches in local branches (give me some text to match)
    Grep { text: String },
}

// TODO:
//      - add status, log?, stash, fetch, add/commit?
//      - file to configure repos and fallback
//      - parallelize with rayon or tokio ==> think about how to stdout
//      - better errors
//      - option for verbosity (stdout, stderr, printf)
//      - context option if already in one repos
//      - interactive prompt ?
//      - customizable args

// ------------ RUNNERS ------------

fn safe_run(cmd: &mut Command) -> io::Result<Output> {
    match cmd.output() {
        Ok(output) => Ok(output),
        Err(e) => {
            println!("{}", "Sorry.. Can't do that ;-; ~".red().italic());
            Err(e)
        }
    }
}

fn execute(cmd: Output) -> io::Result<()> {
    io::stdout().write_all(&cmd.stdout)?;
    io::stderr().write_all(&cmd.stderr)?;
    Ok(())
}

fn run_git_command(dir: &str, args: &[&str]) -> io::Result<()> {
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

fn git_diff(dir: &str) -> io::Result<()> {
    print_header("DIFF", dir);
    run_git_command(dir, &["status", "-vv", "--porcelain"])
}

fn git_show(dir: &str) -> io::Result<()> {
    print_header("SHOW", dir);
    run_git_command(dir, &["show", "--name-only"])
}

fn git_pull(dir: &str) -> io::Result<()> {
    print_header("PULL", dir);
    run_git_command(dir, &["pull"])
}

fn git_checkout(dir: &str, branch: &str) -> io::Result<()> {
    print_header("CHECKOUT", dir);

    let mut cmd = Command::new("git");
    cmd.args(["checkout", branch]).current_dir(dir);
    let output = safe_run(&mut cmd)?;

    if output.status.success() {
        execute(output)
    } else {
        println!(
            "{}",
            format!("Sorry.. '{}' does not exist on '{}'..", branch, dir)
                .red()
                .italic()
        );
        println!("{}", "> Failling back to 'master' ~".purple());

        let mut fallback_cmd = Command::new("git");
        fallback_cmd.args(["checkout", "master"]).current_dir(dir);
        let fallback = safe_run(&mut fallback_cmd)?;

        if fallback.status.success() {
            execute(fallback)
        } else {
            println!(
                "{}",
                "Sorry.. master branch did not work, you may need to check what is going on here ~"
                    .red()
                    .italic()
            );
            Err(io::Error::other("Failed to checkout branch and fallback"))
        }
    }
}

fn git_grep_branch(dir: &str, grep: &str) -> io::Result<()> {
    print_header("GREP BRANCH", dir);

    let gbranch = Command::new("git")
        .args(["branch", "--color=always"])
        .current_dir(dir)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Sorry.. Can't spawn git branch");

    let mut grep_cmd = Command::new("grep");
    grep_cmd
        .arg(grep)
        .stdin(Stdio::from(gbranch.stdout.unwrap()));
    let output = safe_run(&mut grep_cmd)?;
    execute(output)
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

// ------------- RUN --------------

fn main() -> io::Result<()> {
    let args = Args::parse();
    let repos_path = load()?;

    let func_to_execute: Box<dyn Fn(&str) -> io::Result<()>> = match args.command {
        Cmd::Diff => Box::new(move |repo: &str| git_diff(repo)),
        Cmd::Show => Box::new(move |repo: &str| git_show(repo)),
        Cmd::Pull => Box::new(move |repo: &str| git_pull(repo)),
        Cmd::Checkout { branch } => Box::new(move |repo: &str| git_checkout(repo, &branch)),
        Cmd::Grep { text } => Box::new(move |repo: &str| git_grep_branch(repo, &text)),
    };

    for r in repos_path {
        func_to_execute(&r)?
    }

    Ok(())
}
