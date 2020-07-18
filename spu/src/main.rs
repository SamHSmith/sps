use clap::Clap;

#[derive(Clap)]
#[clap(
    version = "0.1",
    author = "Sam H. Smith <sam.henning.smith@protonmail.com>"
)]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "default.conf")]
    config: String,
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
            install_bin_pkg(&b.pkg);
        }
    }
}

fn install_bin_pkg(pkg: &str) {
    if !pkg.ends_with(".sbp.tar.xz") {
        eprintln!("Error, pkg must end in .sbp.tar.xz");
        return;
    }
    let pkg_path = std::path::Path::new(pkg);
    if !pkg_path.exists() {
        eprintln!("Error, {} does not exist", pkg_path.to_str().unwrap());
        return;
    }

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && tar -xf {}",
            pkg_path.parent().unwrap().to_str().unwrap(),
            pkg_path.file_name().unwrap().to_str().unwrap()
        ))
        .output()
        .expect("failed to execute process");
    use std::io::Write;
    std::io::stderr().write_all(&output.stderr);
    assert_eq!(0, output.stderr.len());

    let pkg_path = std::fs::canonicalize(std::path::PathBuf::from(
        pkg_path.to_str().unwrap().replace(".tar.xz", ""),
    ))
    .unwrap();

    use glob::glob_with;
    use glob::MatchOptions;

    let mut files_to_install = Vec::new();

    println!("{}", pkg_path.to_str().unwrap());

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

    for file in files_to_install.iter() {
        println!("path : {}", file.display());
    }
}

fn build_src_package(src: &str, output_dir: &Option<String>) {
    println!("Building package from {}", src);

    let src_parent = std::path::Path::new(src)
        .parent()
        .unwrap()
        .to_str()
        .unwrap();
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
    std::io::stderr().write_all(&output.stderr);
    assert_eq!(0, output.stderr.len());

    let src_string = src.replace(".tar.xz", "");
    let src = src_string.as_str();
    let newdir = format!("{}.sbp", src.replace(".ssp", ""));
    let newdir_path = std::path::Path::new(&newdir);
    let newdir_parent = newdir_path.parent().unwrap();
    let newdir_file_name = newdir_path.file_name().unwrap();

    {
        if newdir_path.exists() {
            if newdir_path.is_file() {
                std::fs::remove_file(newdir_path);
            } else {
                std::fs::remove_dir_all(newdir_path);
            }
        }
    }
    std::fs::create_dir(&newdir).expect(&format!("Failed to create directory {}", &newdir));

    let newdir_can = std::fs::canonicalize(&newdir).unwrap();
    let newdir = newdir_can.to_str().unwrap();

    let systemdir = format!("{}/system", &newdir);
    std::fs::create_dir(&systemdir).expect(&format!("Failed to create directory {}", &systemdir));

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && cp meta.toml {} && SPU_INSTALL_DIR={} ./spu_build",
            src, newdir, systemdir
        ))
        .output()
        .expect("failed to execute process");
    std::io::stderr().write_all(&output.stderr);
    assert_eq!(0, output.stderr.len());

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("rm -dr {}", src,))
        .output()
        .expect("failed to execute process");
    std::io::stderr().write_all(&output.stderr);
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
    std::io::stderr().write_all(&output.stderr);
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
        std::io::stderr().write_all(&output.stderr);
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
    std::io::stderr().write_all(&output.stderr);
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
        std::io::stderr().write_all(&output.stderr);
        assert_eq!(0, output.stderr.len());
    }
}
