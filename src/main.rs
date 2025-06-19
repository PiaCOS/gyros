use std::process::{ Command, Stdio, Output };
use std::io::{ self, Write };
use clap::Parser;
use colored::Colorize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Diff on community and enterprise
    #[clap(long, short, action, default_value_t = false)]
    diff: bool,

    /// Pull on community and enterprise
    #[clap(long, short, action, default_value_t = false)]
    pull: bool,

    /// Checkout on community and enterprise (give me a branch name)
    #[arg(short, long, default_value = None)]
    checkout: Option<String>,

    /// Grep branches in remote (give me some text to match)
    #[arg(short, long, default_value = None)]
    grep: Option<String>,
}

fn gyros() {
    println!("{}", ">-------------------------------------------------------------------------------<".yellow());
    println!("{}", ">---<>---<>---<>---<>---<>---<>---< G Y R O S >---<>---<>---<>---<>---<>---<>---<".yellow());
    println!("{}", ">-------------------------------------------------------------------------------<".yellow());
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

fn run_git_command(dir :&str, args: &[&str]) -> io::Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(dir);
    let output = safe_run(&mut cmd)?;
    execute(output)
}

// ------------ COMMANDS ------------

fn git_diff(dir: &str) -> io::Result<()> {
    println!("{}", format!("\n<>--------<> DIFF -> {} <>--------<>", dir).green().bold());
    run_git_command(dir, &["diff", "--color=always", "--name-status"])
}

fn git_pull(dir: &str) -> io::Result<()> {
    println!("{}", format!("\n<>--------<> PULL -> {} <>--------<>", dir).green().bold());
    run_git_command(dir, &["pull"])
}

fn git_checkout(dir: &str, branch: &str) -> io::Result<()> {
    println!("{}", format!("\n<>--------<> CHECKOUT {} -> {} <>--------<>", dir, branch).green().bold());
    
    let mut cmd = Command::new("git");
    cmd.args(["checkout", branch]).current_dir(dir);
    let output = safe_run(&mut cmd)?;

    if output.status.success() {
        // println!("{}", format!("> Checked out branch '{}' ~\n", branch).purple());
        execute(output)
    } else {
        println!("{}", format!("Sorry.. '{}' does not exist on '{}'..", branch, dir).red().italic());
        println!("{}", "> Failling back to 'master' ~".purple());

        let mut fallback_cmd = Command::new("git");
        fallback_cmd.args(["checkout", "master"]).current_dir(dir);
        let fallback = safe_run(&mut fallback_cmd)?;

        if fallback.status.success() {
            // println!("{}", "> Checked out fallback 'master' branch ~\n".purple());
            execute(fallback)
        } else {
            println!("{}", "Sorry.. master branch did not work, you may need to check what is going on here ~".red().italic());
            Err(io::Error::new(io::ErrorKind::Other, "Failed to checkout branch and fallback"))
        }
    }
}

fn git_grep_branch(dir: &str, grep: &str) -> io::Result<()> {
    println!("{}", format!("\n<>--------<> GREP BRANCH {} -> {} <>--------<>", dir, grep).green().bold());

    let gbranch = Command::new("git")
        .args(["branch", "--color=always"])
        .current_dir(dir)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Sorry.. Can't spawn git branch");

    let mut grep_cmd = Command::new("grep");
    grep_cmd.arg(grep).stdin(Stdio::from(gbranch.stdout.unwrap()));
    let output = safe_run(&mut grep_cmd)?;
    execute(output)
}

fn main() -> io::Result<()> {
    let dir1 = "/home/odoo/Dev/odoo/community/";
    let dir2 = "/home/odoo/Dev/odoo/enterprise/";

    let args = Args::parse();

    match args {
        Args { diff: true, pull: false, checkout: None, grep: None} => {
            gyros();
            git_diff(dir1)?;
            git_diff(dir2)?;
        },
        Args { diff: false, pull: true, checkout: None, grep: None} => {
            gyros();
            git_pull(dir1)?;
            git_pull(dir2)?;
        },
        Args { diff: false, pull: false, checkout: Some(br), grep: None} => {
            gyros();
            git_checkout(dir1, &br)?;
            git_checkout(dir2, &br)?;
        },
        Args { diff: false, pull: false, checkout: None, grep: Some(gr)} => {
            gyros();
            git_grep_branch(dir1, &gr)?;
            git_grep_branch(dir2, &gr)?;
        }
        _ => println!("{}", "<>---<> Exactly one instruction should be given to me :3 <>---<>".red().italic())
    }

    Ok(())
}
