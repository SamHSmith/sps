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
    Push(Push),
    Daemon(Daemon),
    Delete(Delete),
}

#[derive(Clap)]
struct Add {
    path_to_repo: PathBuf,
    #[clap(default_value = ".")]
    path_to_proj: PathBuf,
}
#[derive(Clap)]
struct New {
    /// Repository to create.
    #[clap()]
    path_to_repo: PathBuf,
    #[clap(short, default_value = "16461")] // Ports 16386-16618 are uncontested
    port : u16,
    #[clap(short, default_value = "16462")]
    swarm_port : u16
}
#[derive(Clap)]
struct Push {
    // Repository to push
    path_to_repo : PathBuf
}               
#[derive(Clap)]
struct Daemon {
    // Repository to start the daemon for.
    path_to_repo: PathBuf
}    
#[derive(Clap)]
struct Delete {
    // Repository to delete
    #[clap()]
    path_to_repo: PathBuf,
}
use semver::Version;
#[derive(Debug)]
struct PackageMetaData {
    name: String,
    version: Version,
    description: String,
}
#[derive(Debug)]
struct ProjectConfig {
    flags: Vec<String>,
    archs: Vec<String>,
    enums: Vec<(String, Vec<String>)>,
}

#[derive(Debug)]
struct RepoMetaData {
name : String,
key : String,
address : String,
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Add(a) => {
            assert!(a.path_to_proj.is_dir());
            assert!(a.path_to_repo.is_dir());
            use std::fs::*;
            use toml::Value;
            let mut proj_meta_path = a.path_to_proj.clone();
            proj_meta_path.push("meta.toml");
            let proj_meta = read_to_string(&proj_meta_path)
                .unwrap()
                .parse::<Value>()
                .unwrap();
            let metadata = PackageMetaData {
                name: proj_meta["name"].as_str().unwrap().to_string(),
                version: Version::parse(proj_meta["version"].as_str().unwrap())
                    .expect("Invalid version string"),
                description: proj_meta["description"]
                    .as_str()
                    .unwrap_or(proj_meta["name"].as_str().unwrap())
                    .to_string(),
            };
            println!("{:?}", metadata);

            let mut proj_conf_path = a.path_to_proj.clone();
            proj_conf_path.push("config.toml");
            let proj_conf = read_to_string(&proj_conf_path)
                .unwrap()
                .parse::<Value>()
                .unwrap();
            assert!(
                proj_conf.as_table().unwrap().contains_key("enums"),
                "Found no enums = [] in config.toml"
            );
            assert!(
                proj_conf.as_table().unwrap().contains_key("archs"),
                "Found no archs = [] in config.toml"
            );
            assert!(
                proj_conf.as_table().unwrap().contains_key("flags"),
                "Found no flags = [] in config.toml"
            );
            let mut enums = Vec::new();
            for e in proj_conf["enums"]
                .as_array()
                .expect("Found no enums = [] in config.toml")
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
            {
                assert!(
                    proj_conf.as_table().unwrap().contains_key(&e),
                    format!("There was no enum name {} in config.toml", &e),
                );
                enums.push((
                    e.clone(),
                    proj_conf[&e]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_str().unwrap().to_string())
                        .collect(),
                ));
            }

            let configdata = ProjectConfig {
                flags: proj_conf["flags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect(),
                archs: proj_conf["archs"]
                    .as_array()
                    .expect("Found no archs = [] in config.toml")
                    .iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect(),
                enums: enums,
            };
            println!("{:?}", configdata);

            let mut dest_path = a.path_to_repo.clone();
            dest_path.push(a.path_to_repo.file_name().unwrap().to_str().unwrap());
            dest_path.push(&metadata.name);
            dest_path.push(&format!("{}", metadata.version.major));
            dest_path.push(&format!("{}", metadata.version));
            println!("{:?}", dest_path);

            create_dir_all(&dest_path);
            let mut dest_meta_path = dest_path.clone();
            dest_meta_path.push("meta.toml");
            let mut dest_conf_path = dest_path.clone();
            dest_conf_path.push("config.toml");
            let mut dest_index_path = dest_path.clone();
            dest_index_path.push("index.toml");

            copy(&proj_meta_path, &dest_meta_path).unwrap();
            copy(&proj_conf_path, &dest_conf_path).unwrap();

            let build_ops: Vec<Vec<(String, String)>> = {
                //Will one day get extracted
                let mut options: Vec<(String, Vec<String>)> = Vec::new();
                for flag in configdata.flags.iter() {
                    options.push((flag.to_string(), vec!["".to_owned(), "1".to_owned()]));
                }

                if configdata.archs.len() > 0 {
                    options.push(("archs".to_owned(), configdata.archs.clone()));
                }
                options.extend_from_slice(&configdata.enums);
                let mut option_counts = Vec::new();
                for x in options.iter() {
                    option_counts.push(x.1.len());
                }

                for o in options.iter() {
                    assert!(
                        o.1.len() > 0,
                        format!("Enum {} must have atleast one possible value.", &o.0)
                    );
                }
                let mut current_option = vec![0; options.len()];

                let mut all_options = Vec::new();
                all_options.push(current_option.clone());
                let mut digit = 0;
                loop {
                    if digit >= current_option.len() {
                        break;
                    }
                    current_option[digit] += 1;
                    if current_option[digit] >= option_counts[digit] {
                        current_option[digit] = 0;
                        digit += 1;
                    } else {
                        digit = 0;
                        all_options.push(current_option.clone());
                    }
                }
                all_options
                    .iter()
                    .map(|ao| {
                        ao.iter()
                            .enumerate()
                            .map(|(i, v)| (options[i].0.clone(), options[i].1[*v].clone()))
                            .collect()
                    })
                    .collect()
            };

            //create build file
            let mut proj_build_file_path = a.path_to_proj.clone();
            proj_build_file_path.push("sps_build.sh");
            let build_file_string = read_to_string(&proj_build_file_path)
                .expect("Missing sps_build.sh file in project.");

            let copy_options = {
                let mut copy_options = fs_extra::dir::CopyOptions::new();
                copy_options.copy_inside = true; // Equivilant to cp -r
                copy_options.overwrite = true;
                copy_options
            };

            let open_options = {
                let mut open_options = OpenOptions::new();
                open_options.create(true).write(true).truncate(true);
                open_options
            };
use std::io::Write;
use std::io::BufWriter;

let index_path = { let mut index_path = dest_path.clone();
index_path.push("index"); index_path };
let mut index_file = open_options.open(&index_path).unwrap();
let mut index_file = BufWriter::new(index_file);

            for (index, b) in build_ops.iter().enumerate() {
                println!("{:?}", b);

                let mut out_path = dest_path.clone();
                out_path.push(format!("{}", index));
                create_dir_all(&out_path).unwrap();
                fs_extra::dir::copy(&a.path_to_proj, &out_path, &copy_options).unwrap();

                //Remove dups
                {
                    out_path.push("meta.toml");
                    remove_file(&out_path);
                    out_path.pop();
                }
                {
                    out_path.push("config.toml");
                    remove_file(&out_path);
                    out_path.pop();
                }

                //write build file
                {
                    out_path.push("sps_build.sh");
                    let mut f = open_options.open(&out_path).unwrap();
                    let mut f = BufWriter::new(f);

                    f.write(
                        "# SPS configuration values. Automatically generated at packaging time.\n"
                            .as_bytes(),
                    )
                    .unwrap();
                    for (key, val) in b {
                        f.write(format!("SPS_CONFIG_{}={}\n", key, val).as_bytes())
                            .unwrap();
                    }
                    f.write("\n".as_bytes()).unwrap();
                    f.write(build_file_string.as_bytes()).unwrap();
                    f.flush();
                    out_path.pop();
                }
                tar_and_zstd_dir(&out_path);
                remove_dir_all(&out_path).unwrap();
                out_path.pop();
                out_path.push(format!("{}.tar.zst", index));
                
                let hash = ipfs_add_and_rm(&a.path_to_repo, &out_path);
                index_file.write(format!("{} = \"{}\"\n", index, &hash).as_bytes()).unwrap();
            }
        }
SubCommand::Daemon(d) => {
let exit_status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("IPFS_PATH={}/ipfs ipfs daemon", d.path_to_repo.to_str().unwrap()))
        .spawn()
        .expect("failed to execute process")
        .wait().unwrap();
assert!(exit_status.success());
}
SubCommand::Push(p) => {
let mut repo_index_path = p.path_to_repo.clone();
let name = p.path_to_repo.file_name().unwrap().to_str().unwrap();
repo_index_path.push(name);

let meta_data = {
use std::fs::*;
            use toml::Value;
            
            repo_index_path.push("meta.toml");
            let repo_meta = read_to_string(&repo_index_path)
                .unwrap()
                .parse::<Value>()
                .unwrap();
            let repo_meta = RepoMetaData {
                name: repo_meta["name"].as_str().unwrap().to_string(),
                key: repo_meta["key"].as_str().unwrap().to_string(),
                address: repo_meta["address"].as_str().unwrap().to_string(),
            };
repo_index_path.pop();
repo_meta
};
tar_and_zstd_dir(&repo_index_path);
repo_index_path.pop();
repo_index_path.push(format!("{}.tar.zst", meta_data.name));

let hash = ipfs_add_and_rm(&p.path_to_repo, &repo_index_path);
std::fs::remove_file(&repo_index_path);
println!("here's the hash {}", hash);
let pub_hash = ipfs_name_publish(&p.path_to_repo, &meta_data.key, &hash);

println!("here's the published hash, {}. Here's the reference hash, {}.", &pub_hash,
        &meta_data.address);
}
        SubCommand::New(n) => {
            if n.path_to_repo.exists() {
                assert!(false, "Big bad. It exists");
            }
            use std::fs::*;
            use uuid::Uuid;
            let mut path = n.path_to_repo.clone();
            path.push(n.path_to_repo.file_name().unwrap().to_str().unwrap());
            create_dir_all(&path).unwrap();

let exit_status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
"export IPFS_PATH={}/ipfs && ipfs init && 
ipfs config --json Addresses '{{\"Swarm\":[\"/ip4/0.0.0.0/tcp/{}\",\"/ip6/::/tcp/{}\"],\"API\":\"/ip4/127.0.0.1/tcp/{}\"}}'",
     n.path_to_repo.to_str().unwrap(), n.swarm_port, n.swarm_port,
            n.port))
        .spawn()
        .expect("failed to execute process")
        .wait().unwrap();
assert!(exit_status.success());

            path.push("meta.toml");

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
            let address = ipfs_key_gen(&n.path_to_repo, &key);
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
            path.push(d.path_to_repo.file_name().unwrap().to_str().unwrap());
            path.push("meta.toml");

            let repo = read_to_string(&path).unwrap();
            use toml::Value;

            let repo = repo.parse::<Value>().unwrap(); // Return type Value::Table
            let key = repo["key"].as_str().unwrap();
            path.pop();
            path.pop();
            ipfs_key_rm(&d.path_to_repo, key);
            remove_dir_all(&path).unwrap();
        }
        _ => (),
    }
}

fn tar_and_zstd_dir(dir_path: &std::path::Path) {
    let absolute_path = dir_path.canonicalize().unwrap();
    let dir_name = absolute_path.file_name().unwrap().to_str().unwrap();
    let mut output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
  "cd {}/.. && bsdtar --format=pax -cf {}.tar {} && zstd --rm -f {}.tar",
            absolute_path.to_str().unwrap(),
                dir_name, dir_name, dir_name
        ))
        .spawn()
        .expect("failed to execute process");
    let res = output.wait().unwrap();
    assert!(res.success());
}

fn ipfs_add_and_rm(repo_path : &std::path::Path, item_path : &std::path::Path) -> String {
    let mut output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("IPFS_PATH={}/ipfs ipfs add -Q {} && rm {}",
            repo_path.to_str().unwrap(), item_path.to_str().unwrap(),
                    item_path.to_str().unwrap()))
        .output()
        .expect("failed to execute process");
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
    if output.stdout.len() > 0 {
        output.stdout.truncate(output.stdout.len() - 1);
    }
    String::from_utf8(output.stdout).unwrap()
}

fn ipfs_name_publish(repo_path : &std::path::Path, key_name: &str, hash: &str) -> String {
    let mut output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("IPFS_PATH={}/ipfs ipfs name publish -Q --key={} {}", repo_path.to_str().unwrap(),
                key_name, hash))
        .output()
        .expect("failed to execute process");
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success(), "Is the repo's local ipfs node daemon running?");
    if output.stdout.len() > 0 {
        output.stdout.truncate(output.stdout.len() - 1);
    }
    String::from_utf8(output.stdout).unwrap()
}

fn ipfs_key_gen(repo_path : &std::path::Path, key_name: &str) -> String {
    let mut output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("IPFS_PATH={}/ipfs ipfs key gen {}", repo_path.to_str().unwrap(),
                key_name,))
        .output()
        .expect("failed to execute process");
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
    if output.stdout.len() > 0 {
        output.stdout.truncate(output.stdout.len() - 1);
    }
    String::from_utf8(output.stdout).unwrap()
}

fn ipfs_key_rm(repo_path: &std::path::Path, key_name: &str) {
    let mut output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("IPFS_PATH={}/ipfs ipfs key rm {}", repo_path.to_str().unwrap(),
                key_name,))
        .output()    
        .expect("failed to execute process");
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
}












