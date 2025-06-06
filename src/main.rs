use std::process::{ Command, Stdio, Output };
use std::io::{ self, Write };
use clap::Parser;

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

fn gyros() -> () {
    println!(">-------------------------------------------------------------------------------<");
    println!(">---<>---<>---<>---<>---<>---<>---< G Y R O S >---<>---<>---<>---<>---<>---<>---<");
    println!(">-------------------------------------------------------------------------------<");
}

// TODO:
// -> use clap to have git_diff on -d
// -> create another command with -p
// -> create another command with -c
//      => ask for a branch name
//      => if it doesnt exists in one repo: ask which branch for the other
//      => if both doesn't exists: ask again
//      => if changes
//          ==> raise error
//      => checkout

fn git_diff(dir: &str) -> io::Result<()> {
    println!("\n<>--------<> DIFF -> {} <>--------<>", &dir);
    execute(
        Command::new("git")
            .args(&["diff", "--color=always"])
            .current_dir(dir)
            .output()
            .expect("Sorry.. Can't do that :5\n")
    )
}

fn git_pull(dir: &str) -> io::Result<()> {
    println!("\n<>--------<> PULL -> {} <>--------<>", &dir);
    execute(
        Command::new("git")
            .args(&["pull"])
            .current_dir(dir)
            .output() 
            .expect("Sorry.. Can't do that :5\n")
    )
}

fn git_checkout(dir: &str, branch: &str) -> io::Result<()> {
    println!("\n<>--------<> CHECKOUT {} -> {} <>--------<>", &dir, &branch);
    execute(
        Command::new("git")
            .args(&["checkout", &branch])
            .current_dir(&dir)
            .output()
            .expect(&format!("Sorry.. {} do no exist on {}", &branch, &dir))
        )
}


fn git_grep_branch(dir: &str, grep: &str) -> io::Result<()> {
    println!("\n<>--------<> GREP BRANCH {} -> {} <>--------<>", &dir, &grep);
    
    let gbranch = Command::new("git")
            // .args(&["branch", "-r", "--color=always"])
            .args(&["branch", "--color=always"])
            .current_dir(&dir)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
    execute(
        Command::new("grep")
            .arg(&grep)
            .stdin(Stdio::from(gbranch.stdout.unwrap()))
            .output()
            .expect(&format!("Sorry.. Cant do that :5"))
        )
}

fn execute(cmd: Output) -> io::Result<()> {
    println!("{}\n", cmd.status);
    io::stdout().write_all(&cmd.stdout)?;
    io::stderr().write_all(&cmd.stderr)?;
    Ok(())
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
        _ => println!("<>---<> Exactly one instruction should be given to me :3 <>---<>")
    }

    Ok(())
}

