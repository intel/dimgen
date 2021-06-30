use std::{fs, path::Path, process};

use anyhow::Result;
use console::{style, Emoji};
use indicatif::ParallelProgressIterator;
use log::error;
use rayon::prelude::*;

use crate::docker::{Container, Os};
use crate::pkgdiff::diff;
use crate::progress;
use crate::templater::dockerfile_gen;
use crate::utils::{compress, execute_command, read_to_string, trim};

static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");
static PAPER: Emoji<'_, '_> = Emoji("📃  ", "");

pub struct Sources {}

impl Sources {
    /// generate source pkgs for container
    pub fn generate(container: &Container, os: &Os, seperate: bool) -> Result<()> {
        let (pkgs_path, pkgs_dir) = Sources::list_pkgs(&container, &*os.name).unwrap();
        if seperate == true {
            // generate base pkgs and added pkgs as seperate files
            println!(
                " {}      {}",
                style("[*/*]").blue().bold(),
                style("generating sources separately for base and layers").bold()
            );
            let base_image = format!("{}:{}", os.name, os.version);
            let base_container = Container::start(&base_image);
            let (base_pkgs_path, base_pkgs_dir) =
                Sources::list_pkgs(&base_container, &*os.name).unwrap();
            let diff_pkgs_path = format!(
                "/tmp/diff_pkgs_{}_{}",
                &base_container.img.hash, &container.img.hash
            );
            let diff_pkgs_dir = format!(
                "/tmp/pkg_sources_{}_{}",
                &base_container.img.hash, &container.img.hash
            );
            fs::create_dir_all(Path::new(&diff_pkgs_dir)).unwrap();
            diff::diff_files(&base_pkgs_path, &pkgs_path, &diff_pkgs_path).unwrap();
            rayon::scope(|s| {
                s.spawn(|_| {
                    Sources::generate_sources(
                        "base",
                        &base_container,
                        &os,
                        &base_pkgs_path,
                        &base_pkgs_dir,
                    )
                    .unwrap()
                });
                s.spawn(|_| {
                    Sources::generate_sources(
                        "layers",
                        &container,
                        &os,
                        &diff_pkgs_path,
                        &diff_pkgs_dir,
                    )
                    .unwrap()
                });
            });
            base_container.stop();
        } else {
            // fetch sources for the entire image
            println!(
                " {}      {}",
                style("[*/*]").blue().bold(),
                style("generating sources for entire image").bold()
            );
            Sources::generate_sources("full", &container, &os, &pkgs_path, &pkgs_dir)?;
        }
        Ok(())
    }

    /// list packages in the container are generated and path to the file is returned
    fn list_pkgs(container: &Container, os_name: &str) -> Option<(String, String)> {
        let pkgs_dir = format!("/tmp/pkg_sources_{}_{}", &os_name, &container.id);
        let pkgs_path = format!("{}/pkgs.list", &pkgs_dir);

        // init
        container.execute("mkdir -p /tmp/pkg_sources");
        fs::create_dir_all(Path::new("./collaterals/")).unwrap();
        fs::create_dir_all(Path::new(&pkgs_dir)).unwrap();

        match os_name {
            "centos" | "redhat" | "fedora" => {
                container.execute("rpm -qa > /tmp/pkg_sources/pkgs.list");
                container.copy("/tmp/pkg_sources/pkgs.list", &pkgs_path);
            }
            "ubuntu" | "debian" => {
                container.execute(
                    "apt list --installed | cut -d/ -f 1 | tail -n +2 > /tmp/pkg_sources/pkgs.list",
                );
                container.copy("/tmp/pkg_sources/pkgs.list", &pkgs_path);
            }
            _ => {
                println!(
                    "{}: [{}]",
                    style("image os not supported, stopping early").bold(),
                    style("✘").red()
                );
            }
        }
        Some((pkgs_path, pkgs_dir))
    }

    /// generate sources
    pub fn generate_sources(
        prefix: &str,
        container: &Container,
        os: &Os,
        pkgs_path: &str,
        pkgs_dir: &str,
    ) -> Result<()> {
        let mut container_name = container.img.name.to_string();
        trim(&mut container_name);
        let tar_file = format!(
            "{}_{}_{}.tar.gz",
            &prefix, &container.img.hash, &container_name
        );
        let tar_file_path = format!("collaterals/{}", &tar_file);
        let dockerfile_name = format!(
            "{}_{}_{}.Dockerfile",
            &prefix, &container.img.hash, &container_name
        );

        let num_pkgs = match &*os.name {
            "ubuntu" => Sources::fetch_sources(container, "ubuntu", pkgs_path),
            "debian" => Sources::fetch_sources(container, "debian", pkgs_path),
            "centos" | "fedora" | "redhat" => {
                Sources::fetch_sources(container, "centos", pkgs_path)
            }
            _ => {
                println!(
                    "{}: [{}]",
                    style("image os not supported, stopping early").bold(),
                    style("✘").red()
                );
                container.stop();
                process::exit(1);
            }
        };
        let rm_sources_path = format!("rm -r {}", &pkgs_path);
        execute_command(&rm_sources_path);
        println!(
            " {} Attempted to fetch {} sources, please verify. [{}]",
            style("[*/*]").bold().dim(),
            style(num_pkgs).bold(),
            style("✔").green()
        );
        container.execute("cd /tmp && tar -czvf pkg_sources.tar.gz pkg_sources");
        container.copy(
            "/tmp/pkg_sources.tar.gz",
            &(pkgs_dir.to_string() + "/sources.tar.gz"),
        );
        println!(
            " {} {} compressing source packages [{}]",
            style("[*/*]").bold().dim(),
            TRUCK,
            style("✔").green()
        );
        dockerfile_gen(&dockerfile_name, &tar_file, &*os.name)?;
        compress(&pkgs_dir, &tar_file_path, "fast")?;
        println!(
            " {} {} sources compressed to : {} [{}]",
            style("[*/*]").bold().dim(),
            PAPER,
            &tar_file_path,
            style("✔").green()
        );
        let pkgs_dir = Path::new(&pkgs_dir);
        fs::remove_dir_all(&pkgs_dir)?;
        Ok(())
    }

    /// setup repos for oses
    fn setup_repos(container: &Container, os: &str) {
        match os {
            "centos" | "redhat" | "fedora" => {
                container.execute("yum install -y yum-utils");
            }

            "ubuntu" => {
                let focal_src_repos = r###"
    echo 'deb-src http://archive.ubuntu.com/ubuntu/ focal main restricted
    deb-src http://archive.ubuntu.com/ubuntu/ focal-updates main restricted
    deb-src http://archive.ubuntu.com/ubuntu/ focal universe
    deb-src http://archive.ubuntu.com/ubuntu/ focal-updates universe
    deb-src http://archive.ubuntu.com/ubuntu/ focal-backports main restricted universe
    deb-src http://security.ubuntu.com/ubuntu/ focal-security main restricted
    deb-src http://security.ubuntu.com/ubuntu/ focal-security universe' >> /etc/apt/sources.list
    apt-get update
"###;
                container.execute(&focal_src_repos);
            }
            "debian" => {
                let buster_src_repos = r###"
    echo 'deb-src http://deb.debian.org/debian/ stable main
    deb-src http://deb.debian.org/debian/ stable-updates main
    deb-src http://deb.debian.org/debian-security stable/updates main
    deb-src http://ftp.debian.org/debian buster-backports main' >> /etc/apt/sources.list
    apt-get update
"###;
                container.execute(&buster_src_repos);
            }
            _ => error!("os not supported.."),
        }
    }

    // fetch sources
    fn fetch_sources(container: &Container, os: &str, pkgs_path: &str) -> usize {
        let mut num_pkgs = usize::default();
        Sources::setup_repos(&container, &os);
        match os {
            "centos" | "redhat" | "fedora" => {
                num_pkgs = parallel_execute(&container, "centos", pkgs_path);
            }

            "ubuntu" | "debian" => {
                num_pkgs = parallel_execute(&container, "ubuntu", pkgs_path);
            }
            _ => error!("os not supported.."),
        }
        num_pkgs
    }
}

/// parallel fetch packages for Oses
fn parallel_execute(container: &Container, os: &str, pkgs_path: &str) -> usize {
    let progress_bar = progress::config();
    let mut num_pkgs = usize::default();

    if let Ok(pkgs) = read_to_string(pkgs_path) {
        num_pkgs = pkgs.lines().count();
        pkgs.par_lines()
            .progress_with(progress_bar.clone())
            .for_each(|pkg| {
                let get_pkg_sources = match os {
                    "ubuntu" => format!("cd /tmp/pkg_sources/ && apt-get -q source {}", pkg),
                    "centos" => format!(
                        "cd /tmp/pkg_sources && yumdownloader --resolve --source {}",
                        pkg
                    ),
                    _ => "os not supported".to_string(),
                };
                let progress_bar = progress_bar.clone();
                progress_bar.set_message(&format!("fetching: {}", &pkg));
                container.execute(&get_pkg_sources);
            });
    }
    num_pkgs
}
