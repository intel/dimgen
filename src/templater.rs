//!dockerfile template generator

use std::fs::File;
use std::io::prelude::*;

use anyhow::Result;

/// generate a dockerfile for creating a docker image
pub fn dockerfile_gen(name: &str, archive: &str, base_os: &str) -> Result<()> {
    let base_os = match base_os {
        "ubuntu" => "ubuntu:20.04",
        "rhel" | "ubi" | "redhat" | "centos" | "\"centos\"" | "\"redhat\"" => "centos:8",
        _ => "ubuntu:20.04",
    };
    let mut dockerfile = File::create(format!("collaterals/{}", name))?;
    dockerfile.write_all(
        format!(
            "From {}
COPY ./{} /workspace/
CMD bash",
            base_os, archive
        )
        .as_bytes(),
    )?;
    Ok(())
}
