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
    path_to_repo: PathBuf,
    #[clap(default_value = ".")]
    path_to_proj: PathBuf,
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

            let build_ops: Vec<Vec<(String, String)>> = {//Will one day get extracted
                let mut options: Vec<(String, Vec<String>)> = Vec::new();
                for flag in configdata.flags.iter() {
                    options.push((
                        flag.to_string(),
                        vec!["false".to_owned(), "true".to_owned()],
                    ));
                }

                if configdata.archs.len() > 0 {
                    options.push(("archs".to_owned(), configdata.archs.clone()));
                }
                options.extend_from_slice(&configdata.enums);
                let mut option_counts = Vec::new();
                for x in options.iter() {
                    option_counts.push(x.1.len());
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
                            .map(|(i, v)| (options[i].0.clone(),
                                            options[i].1[*v].clone()))
                            .collect()
                    })
                    .collect()
            };

            for b in build_ops.iter() {
                println!("{:?}", b);
            }
        }
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
    output.stdout.truncate(output.stdout.len() - 1);
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

