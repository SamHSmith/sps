use clap::Clap;

#[derive(Clap)]
#[clap(
    version = "0.1",
    author = "Sam H. Smith <sam.henning.smith@protonmail.com>"
)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}
use std::path::PathBuf;

#[derive(Clap)]
enum SubCommand {
    #[clap(
        version = "0.1",
        author = "Sam H. Smith <sam.henning.smith@protonmail.com>"
    )]
    Add(Add),
    New(New),
    Delete(Delete),
}

#[derive(Clap)]
struct Add {
    /// Repository to add this package to.
    #[clap()]
    path_to_repo: PathBuf,
}
#[derive(Clap)]
struct New {
    /// Repository to create.
    #[clap()]
    path_to_repo: PathBuf,
}
#[derive(Clap)]
struct Delete {
    // Repository to delete
    #[clap()]
    path_to_repo: PathBuf,
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::New(n) => {
            if n.path_to_repo.exists() {
                assert!(false, "Big bad. It exists");
            }
            use std::fs::*;
            use uuid::Uuid;
            let mut path = n.path_to_repo.clone();
            create_dir_all(&path).unwrap();
            path.push("repo.toml");

            let name = path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();

            let key = format!(
                "{}-{}",
                name,
                Uuid::new_v4()
                    .to_simple()
                    .encode_lower(&mut Uuid::encode_buffer())
            );
            let address = ipfs_key_gen(&key);
            write(
                &path,
                &format!(
"name = \"{}\"
key = \"{}\"
address = \"{}\"
",
                    name, &key, &address
                ),
            );
        }
        SubCommand::Delete(d) => {
            if !d.path_to_repo.exists() {
                assert!(false, "Big bad. It does not exists");
            }
            use std::fs::*;
            let mut path = d.path_to_repo.clone();
            path.push("repo.toml");

            let repo = read_to_string(&path).unwrap();
            use toml::Value;

            let repo = repo.parse::<Value>().unwrap(); // Return type Value::Table
            let key = repo["key"].as_str().unwrap();
            path.pop();
            remove_dir_all(&path).unwrap();
            ipfs_key_rm(key);
        }
        _ => (),
    }
}

fn ipfs_key_gen(key_name: &str) -> String {
    let mut output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("ipfs key gen {}", key_name,))
        .output()
        .expect("failed to execute process");
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
    output.stdout.truncate(output.stdout.len()-1);
    String::from_utf8(output.stdout).unwrap()
}

fn ipfs_key_rm(key_name: &str) {
    let mut output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("ipfs key rm {}", key_name,))
        .output()
        .expect("failed to execute process");
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
}

