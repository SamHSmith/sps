mod repo;
use repo::*;

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
    Repository(Repository),
    Add_Repo(Add_Repo),
}

#[derive(Clap)]
struct Add_Repo {
    repo_hash: String,
}

fn main() {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Repository(r) => {
            repository_cli(r);
        }
        SubCommand::Add_Repo(a) => {
            use std::fs::*;
            use std::path::*;
            let mut current_path = PathBuf::from("./usr/sps/repos");
            create_dir_all(&current_path).unwrap();
            ipfs_get_and_uncompress(&current_path,
                &format!("/ipns/{}", &a.repo_hash), &a.repo_hash);
            
            let mut second_path = current_path.clone();
            second_path.push(&a.repo_hash);
            current_path.push(&format!("{}.tar", &a.repo_hash));
            rename(&second_path, &current_path);
            un_tar(&current_path);
            
        }
    }
}
fn un_tar(tar_path: &std::path::Path) {
let mut output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && bsdtar -xf {} && rm {}",
            tar_path.parent().unwrap().to_str().unwrap(),
            tar_path.file_name().unwrap().to_str().unwrap(),
            tar_path.file_name().unwrap().to_str().unwrap(),
        ))
        .spawn()
        .expect("failed to execute process")
        .wait().unwrap();
    assert!(output.success());
}

fn ipfs_get_and_uncompress(output_dir: &std::path::Path, ipfs_address: &str,
        out_name: &str) {
    let mut output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && ipfs get -o {}.zst {} && zstd --rm -fd {}.zst",
            output_dir.to_str().unwrap(),
            out_name, ipfs_address, out_name,
        ))
        .spawn()
        .expect("failed to execute process")
        .wait().unwrap();
    assert!(output.success(), "did you pass a valid ipfs address?");
}














