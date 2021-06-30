use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::{fs::File, io::BufWriter};

use std::collections::HashSet;

use anyhow::Result;

/// read entire content of file to vector
fn read_to_string<P>(file_name: P) -> io::Result<String>
where
    P: AsRef<Path>,
{
    let mut buffer = String::new();
    let mut file = File::open(file_name)?;
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// difference between two files, with result being
/// contents in file2 that is not in file1
fn diff_content<P>(file_a: P, file_b: P) -> Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let buffer1 = read_to_string(file_a)?;
    let buffer2 = read_to_string(file_b)?;
    let mut buffer3 = Vec::new();

    let ucontent_1: HashSet<_> = buffer1.split("\n").collect();
    let ucontent_2: HashSet<_> = buffer2.split("\n").collect();
    for line in ucontent_2.difference(&ucontent_1) {
        buffer3.push(line.to_string());
    }
    Ok(buffer3)
}

///write content to disk
fn write_to_disk<P>(path: P, content: &Vec<String>) -> Result<()>
where
    P: AsRef<Path>,
{
    let file = File::create(path)?;
    let mut writer = BufWriter::new(&file);
    for line in content.iter() {
        writer.write(line.as_bytes())?;
        writer.write("\n".as_bytes())?;
    }
    Ok(())
}

/// write `to_path` file difference of 2 files with
/// result being contents in file2 that is not in file1
pub fn diff_files<P>(path_1: P, path_2: P, to_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let buffer = diff_content(path_1, path_2).unwrap();
    write_to_disk(to_path, &buffer).unwrap();
    Ok(())
}
