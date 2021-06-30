use std::io;
use std::path::Path;
use std::{fs::File, io::Read, process::Command};

use anyhow::Error;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::info;

/// execute a shell command
pub fn execute_command(command: &str) -> Vec<u8> {
    info!("executing command {}", command);
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);
    cmd.output().unwrap().stdout
}

///read file to string
pub fn read_to_string<P>(filename: P) -> io::Result<String>
where
    P: AsRef<Path>,
{
    let mut entire_file = String::new();
    let mut file = File::open(filename)?;
    file.read_to_string(&mut entire_file)?;
    Ok(entire_file)
}

///trim end of line and whitespace from end of string
pub fn trim(string: &mut String) {
    while string.ends_with('\n') || string.ends_with('\r') {
        string.pop();
    }
    while string.starts_with('"') || string.ends_with('"') {
        string.pop();
        string.remove(0);
    }
    string.retain(|c| return !r#"/(),";:'"#.contains(c));
    string.retain(|c| return !c.is_whitespace());
}

///compress a dir
pub fn compress(dir: &str, tar_file: &str, ratio: &str) -> Result<bool, Error> {
    let tar_gz = File::create(&tar_file)?;
    let encoder = match ratio {
        "fast" => GzEncoder::new(tar_gz, Compression::fast()),
        "best" => GzEncoder::new(tar_gz, Compression::best()),
        _ => GzEncoder::new(tar_gz, Compression::default()),
    };
    let mut tar = tar::Builder::new(encoder);
    tar.append_dir_all(".", dir)?;
    Ok(true)
}
