use colored::Colorize;
use serde_derive::Deserialize;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Output};


// TODO:
//      - add status, log?, stash, fetch, add/commit?
//      - file to configure repos and fallback
//      - parallelize with rayon or tokio ==> think about how to stdout
//      - better errors
//      - option for verbosity (stdout, stderr, printf)
//      - context option if already in one repos
//      - interactive prompt ?
//      - customizable args

fn get_args() -> Vec<String> {
    let args: Vec<String> = env::args().collect();
    args
}

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
    print_header(&args[1], dir);
    run_git_command(dir, &args[1..])
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
    let repos_path = load()?;
    let args = get_args();

    for r in repos_path {
        command(&r, &args)?;
    }
    Ok(())
}
