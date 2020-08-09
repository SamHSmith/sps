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

#[derive(Clap)]
enum SubCommand {
    #[clap(
        version = "0.1",
        author = "Sam H. Smith <sam.henning.smith@protonmail.com>"
    )]
    Package(Package),
    Build(Build),
    Install(Install),
    Remove(Remove),
}

/*
Package a peice of software in Source or Binary form.
*/
#[derive(Clap)]
struct Package {
    /// Make Source packages instead of a Binary package
    #[clap(short, long)]
    source_package: bool,
    /// Folder to package
    #[clap()]
    src: String,
    /// Output directory
    #[clap(short, long)]
    output: Option<String>,
}

/*
Build a binary package from a source package
*/
#[derive(Clap)]
struct Build {
    /// SPS Source Package to build, ends in .ssp.tar.xz
    #[clap()]
    src: String,
    /// Output directory
    #[clap(short, long)]
    output: Option<String>,
}

/*
Install a binary package
*/
#[derive(Clap)]
struct Install {
    /// SPS Binary Package to install
    #[clap()]
    pkg: String,
}

/*
Remove a binary package
 */
#[derive(Clap)]
struct Remove {
    /// SPS Binary Package to remove, you should just enter the bit before the .sbp.tar.xz
    #[clap()]
    pkg_name: String,
}

use std::path::Path;
fn main() {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Package(b) => {
            let srcfolder = std::fs::File::open(&b.src);

            if srcfolder.is_err() {
                eprintln!(
                    "Error while reading {}, does it exist and do you have read permission?",
                    &b.src
                );
                return;
            } else {
                if b.source_package {
                    let output = match b.output {
                        Some(o) => o,
                        None => format!("{}.ssp.tar.xz", b.src),
                    };
                    panic!("Source packaging is not implemented");
                } else {
                    let output = match b.output {
                        Some(o) => o,
                        None => std::path::Path::new(&b.src)
                            .parent()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned(),
                    };
                    let path = Path::new(&b.src);
                    package_binary_package(&path, &output);
                }
            }
        }
        SubCommand::Build(b) => {
            let srcpkg = std::fs::File::open(&b.src);

            if srcpkg.is_err() {
                println!(
                    "Error while reading {}, does it exist and do you have read permission?",
                    &b.src
                );
                return;
            } else {
                build_src_package(&b.src, &b.output);
            }
        }
        SubCommand::Install(b) => {
            let conflicts = install_bin_pkg(&b.pkg);
            match conflicts {
                Some(c) => {
                    use std::ops::Add;
                    let mut prompt = "The following packages need to be removed:\n".to_owned();
                    for con in c.iter() {
                        prompt = prompt.add("   ");
                        prompt = prompt.add(con);
                        prompt = prompt.add("\n");
                    }
                    prompt = prompt.add("Do you want to continue? [y/n] : ");
                    //print!("{}", &prompt);
                    quest::ask(&prompt);
                    let yesno = quest::yesno(false).unwrap();
                    if yesno.is_some() && yesno.unwrap() {
                    for con in c.iter() {
                        remove_bin_pkg(con);
                    }
                    let conflicts = install_bin_pkg(&b.pkg);
                    if conflicts.is_some() && conflicts.unwrap().len() > 0 {
                        panic!();
                    }}
                },
                None => (),
            }
        }
        SubCommand::Remove(b) => {
            remove_bin_pkg(&b.pkg_name);
        }
    }
}

fn remove_bin_pkg(pkg: &str) {
    let spu_install_dir = match std::env::var("SPU_INSTALL_DIR") {
        Ok(val) => val,
        Err(_) => "/".to_owned(),
    };

    

    let filedb_path = std::path::PathBuf::from(format!("{}/usr/spu/filedb", &spu_install_dir));
    let mut filedb = std::fs::OpenOptions::new().read(true).write(true).open(filedb_path).unwrap();
    let mut db: std::collections::HashMap<String, String> =open_hash_table_data_base(&mut filedb);

    std::fs::create_dir_all(format!("{}/usr/spu/pkgs", &spu_install_dir)).unwrap();

    let db_file_string = format!("{}/usr/spu/pkgs/{}", &spu_install_dir, &pkg);

    let db_file = std::path::Path::new(&db_file_string);
    if !db_file.exists() {
        eprintln!("{} not installed.", &pkg);
        return;
    }
    {
        let db_file = std::fs::File::open(db_file).unwrap();

        use std::io::BufRead;

        for line in std::io::BufReader::new(db_file).lines() {
            if line.is_err() {
                continue;
            }
            let line = line.unwrap();
            if line.len() <= 0 {
                continue;
            }
            println!("removing {} ...", &line);
            let file_path = std::path::PathBuf::from(format!("{}/{}", &spu_install_dir, &line));
            if file_path.is_dir() {
                std::fs::remove_dir_all(&file_path).unwrap();
            } else if file_path.is_file() {
                std::fs::remove_file(&file_path).unwrap();
            }
            db.remove(&line).unwrap();
        }
    }
    std::fs::remove_file(&db_file).unwrap();
    close_hash_table_data_base(filedb, &db);
}
use fs2::FileExt;
fn open_hash_table_data_base(file : &mut std::fs::File) -> std::collections::HashMap<String, String>{
    use std::io::Read;
    file.lock_exclusive().unwrap();
    let db : std::collections::HashMap<String, String>= {
        let f = std::io::BufReader::new(file);
        bincode::deserialize_from(f).unwrap()
    };
    db
}

fn close_hash_table_data_base(mut file : std::fs::File, db : &std::collections::HashMap<String, String>) {
    use std::io::Write;
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
    bincode::serialize_into(std::io::BufWriter::new(&file), db).unwrap();
    file.flush().unwrap();
    file.unlock().unwrap();
}

fn install_bin_pkg(pkg: &str) -> Option<Vec<String>> {
    if !pkg.ends_with(".sbp.tar.xz") {
        eprintln!("Error, pkg must end in .sbp.tar.xz");
        return None;
    }
    let pkg_path = std::path::Path::new(pkg);
    if !pkg_path.exists() {
        eprintln!("Error, {} does not exist", pkg_path.to_str().unwrap());
        return None;
    }
    let mut pkg_path_parent = pkg_path.parent().unwrap();
    if pkg_path_parent.to_str().unwrap().len() == 0 {
        pkg_path_parent = std::path::Path::new(".");
    }

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && tar -xf {}",
            pkg_path_parent.to_str().unwrap(),
            pkg_path.file_name().unwrap().to_str().unwrap()
        ))
        .output()
        .expect("failed to execute process");
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert_eq!(0, output.stderr.len());

    let pkg_path = std::fs::canonicalize(std::path::PathBuf::from(
        pkg_path.to_str().unwrap().replace(".tar.xz", ""),
    ))
    .unwrap();

    use glob::glob_with;
    use glob::MatchOptions;

    let mut files_to_install = Vec::new();

    let system_dir_string = format!("{}/system", pkg_path.to_str().unwrap());
    let system_dir_path = std::path::Path::new(&system_dir_string);

    for entry in glob_with(&format!("{}/**/*", &system_dir_string), MatchOptions::new())
        .expect("Failed to read glob pattern")
    {
        match entry {
            Ok(path) => {
                //this if is nececary so that we don't get a new path for each folder depth in a tree
                if path.is_file() || (path.is_dir() && path.read_dir().unwrap().next().is_none()) {
                    // is the dir empty, then keep it. So that packages can empty create dirs too.
                    files_to_install
                        .push(path.strip_prefix(&system_dir_path).unwrap().to_path_buf());
                }
            }
            Err(e) => {
                eprintln!("EROR something whent wrong while globing : {:?}", e);
                panic!();
            }
        }
    }

    let spu_install_dir = match std::env::var("SPU_INSTALL_DIR") {
        Ok(val) => val,
        Err(_) => "/".to_owned(),
    };

    std::fs::create_dir_all(format!("{}/usr/spu/", &spu_install_dir)).unwrap();

    let mut db: std::collections::HashMap<String, String>;

    let filedb_path = std::path::PathBuf::from(format!("{}/usr/spu/filedb", &spu_install_dir));
    let mut filedb = if !filedb_path.exists() {
        db = std::collections::HashMap::<String, String>::new();
        let file = std::fs::File::create(&filedb_path).unwrap();
        file.lock_exclusive().unwrap();

        file
    } else {
        let mut file = std::fs::OpenOptions::new().read(true).write(true).open(filedb_path).unwrap();
        db = open_hash_table_data_base(&mut file);
        file
    };

    let mut conflicts = Vec::new();

    for path in files_to_install.iter() {
        let get = db.get(path.to_str().unwrap());
        if get.is_some() {
            conflicts.push(get.unwrap().to_owned());
        }
    }

    if conflicts.len() > 0 {
        return Some(conflicts);
    }

    let mut installed_files = Vec::new();

    for file in files_to_install.iter() {
        let copy_to = format!("{}/{}", &spu_install_dir, file.to_str().unwrap());
        if !std::path::Path::new(&copy_to).exists() {
            installed_files.push(file.to_str().unwrap());
            let result = std::fs::copy(
                format!("{}/{}", &system_dir_string, file.to_str().unwrap()),
                &copy_to,
            );
            if result.is_err() {
                eprintln!("Failed to install {}, do you have permission?", copy_to);
            }
        } else {
            println!("skipping path : {}/{}", &spu_install_dir, file.display());
            //TODO Do package collision check and ask about override
        }
    }

    if installed_files.len() <= 0 {
        println!("Nothing was installed.");
        return None;
    }

    std::fs::create_dir_all(format!("{}/usr/spu/pkgs", &spu_install_dir)).unwrap();

    let pkg_id =pkg_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
        .replace(".sbp", "");

    println!("Installing {}...", &pkg_id);

    let db_file_string = format!(
        "{}/usr/spu/pkgs/{}",
        &spu_install_dir,
        &pkg_id,
    );

    let mut db_file = std::fs::File::create(&db_file_string).unwrap();

    for file in installed_files.iter() {
        db_file.write((*file).as_bytes()).unwrap();
        db_file.write("\n".as_bytes()).unwrap();
    }

    db_file.flush().unwrap();

    for path in files_to_install.iter() {
        db.insert(path.to_str().unwrap().to_owned(), std::path::Path::new(&db_file_string).file_name().unwrap().to_str().unwrap().to_owned());
    }
    close_hash_table_data_base(filedb, &db);

    std::fs::remove_dir_all(pkg_path).unwrap();

    println!("Done");

    None
}

use serde_derive::Deserialize;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct BuildConfig {
    flags : Vec<String>,
}

fn build_src_package(src: &str, output_dir: &Option<String>) {
    println!("Building package from {}", src);

    let mut src_parent = std::path::Path::new(src)
        .parent()
        .unwrap()
        .to_str()
        .unwrap();
    if src_parent.len() == 0 {
        src_parent = ".";
    }
    let src_file_name = std::path::Path::new(src)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("cd {} && tar -xf {}", src_parent, src_file_name))
        .output()
        .expect("failed to execute process");
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert_eq!(0, output.stderr.len());

    let src_string = src.replace(".tar.xz", "");
    let src = src_string.as_str();
    let newdir = format!("{}.sbp", src.replace(".ssp", ""));
    let newdir_path = std::path::Path::new(&newdir);
    let mut newdir_parent = newdir_path.parent().unwrap();
    if newdir_parent.to_str().unwrap().len() == 0 {
        newdir_parent = std::path::Path::new(".");
    }
    let newdir_file_name = newdir_path.file_name().unwrap();

    {
        if newdir_path.exists() {
            if newdir_path.is_file() {
                std::fs::remove_file(newdir_path).unwrap();
            } else {
                std::fs::remove_dir_all(newdir_path).unwrap();
            }
        }
    }
    std::fs::create_dir(&newdir).expect(&format!("Failed to create directory {}", &newdir));

    let newdir_can = std::fs::canonicalize(&newdir).unwrap();
    let newdir = newdir_can.to_str().unwrap();

    let systemdir = format!("{}/system", &newdir);
    std::fs::create_dir(&systemdir).expect(&format!("Failed to create directory {}", &systemdir));

    let config_string = std::fs::read_to_string(format!("{}/config.toml", src)).unwrap();
    let config : BuildConfig = toml::from_str(&config_string).unwrap();

    let mut flags = String::new();
    for f in config.flags.iter() {
        flags.push_str(&format!("SPU_CONFIG_{}=true ", f));
    }

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && cp meta.toml {} && {} SPU_INSTALL_DIR={} ./spu_build",
            src, newdir,flags, systemdir
        ))
        .output()
        .expect("failed to execute process");
    std::io::stderr().write_all(&output.stderr).unwrap();
    //assert_eq!(0, output.stderr.len()); this one fails for some reason

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("rm -dr {}", src,))
        .output()
        .expect("failed to execute process");
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert_eq!(0, output.stderr.len());

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && tar -caf {}.tar.xz {} && rm -dr {}",
            newdir_parent.to_str().unwrap(),
            newdir_file_name.to_str().unwrap(),
            newdir_file_name.to_str().unwrap(),
            newdir_file_name.to_str().unwrap(),
        ))
        .output()
        .expect("failed to execute process");
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert_eq!(0, output.stderr.len());

    if output_dir.is_some() {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "mv {}.tar.xz {}",
                newdir,
                (*output_dir).as_ref().unwrap().as_str()
            ))
            .output()
            .expect("failed to execute process");
        std::io::stderr().write_all(&output.stderr).unwrap();
        assert_eq!(0, output.stderr.len());
    }
}

fn package_binary_package(srcfolder: &Path, output_name: &str) {
    println!(
        "Contructing binary package from {}",
        srcfolder.to_str().unwrap()
    );
    let src_name = srcfolder.file_name().unwrap().to_str().unwrap();
    let src_path = srcfolder.parent().unwrap().to_str().unwrap();

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && tar -caf {}.sbp.tar.xz {}",
            src_path, src_name, src_name
        ))
        .output()
        .expect("failed to execute process");
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert_eq!(0, output.stderr.len());

    if !format!("{}", src_path).eq(output_name) {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "mv {}/{}.sbp.tar.xz {}",
                src_path, src_name, output_name
            ))
            .output()
            .expect("failed to execute process");
        std::io::stderr().write_all(&output.stderr).unwrap();
        assert_eq!(0, output.stderr.len());
    }
}

