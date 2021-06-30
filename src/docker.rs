//!connect to docker daemon and execute commands.
use std::env;
use std::string::FromUtf8Error;

use log::info;

use crate::utils::{execute_command, trim};

/// image name and hash
pub struct Image {
    pub name: String,
    pub hash: String,
}

/// container id and image name from which it was instantiated
pub struct Container {
    pub id: String,
    pub img: Image,
}

/// container Os metadata
pub struct Os {
    pub name: String,
    pub version: String,
}

/// check if docker is installed on the host
pub fn is_docker_installed() -> bool {
    let command = "which docker";
    let result = execute_command(command);

    let result = String::from_utf8(result).unwrap();
    result.contains("docker")
}

impl Image {
    /// docker image hash
    fn new(img_name: &str) -> Self {
        Image::fetch_image(img_name);
        let command = format!(
            "docker inspect --format='{{{{index .Id }}}}' {} | cut -d: -f2 | head -c 10",
            img_name
        );
        let mut hash = String::from_utf8(execute_command(&command)).unwrap();
        trim(&mut hash);
        Self {
            name: img_name.to_string(),
            hash,
        }
    }

    ///daemonize a docker container from an image_name
    pub fn daemonize(self) -> String {
        // get proxy variables if set in env
        let mut http_proxy = env::var("HTTP_PROXY").unwrap_or("None".to_string());
        let mut https_proxy = env::var("HTTPS_PROXY").unwrap_or("None".to_string());
        let mut no_proxy = env::var("NO_PROXY").unwrap_or("None".to_string());
        if http_proxy == "None" || https_proxy == "None" || no_proxy == "None" {
            http_proxy = env::var("http_proxy").unwrap_or("\"\"".to_string());
            https_proxy = env::var("https_proxy").unwrap_or("\"\"".to_string());
            no_proxy = env::var("no_proxy").unwrap_or("\"\"".to_string());
        }
        let command = format!(
            "docker run -u0 -e http_proxy={} -e https_proxy={} -e no_proxy={} -dt {}",
            http_proxy, https_proxy, no_proxy, self.name
        );

        let result = execute_command(&command);
        if result.is_empty() {
            println!("image :: {} cannot be pulled, exiting..", self.name);
            std::process::exit(1);
        }
        String::from_utf8(result).unwrap()[..11].to_string()
    }

    /// check if image exists if not try to pull and make it available
    pub fn fetch_image(img_name: &str) {
        let command = format!("docker inspect {}", img_name);
        let exists = execute_command(&command);
        if exists.len() == 3 {
            println!("image doesn't exists, attempting to pull the image");
            Image::pull_image(img_name);
            info!("image successfully pulled");
        } else {
            info!("image exists locally...");
        }
    }

    /// pull docker image
    pub fn pull_image(img_name: &str) -> String {
        let command = format!("docker pull {}", img_name);
        let result = execute_command(&command);
        String::from_utf8(result).unwrap()
    }
}

impl Container {
    /// deamonize and setup
    pub fn start(img_name: &str) -> Self {
        let img = Image::new(img_name);
        let image = Image {
            name: img.name.to_owned(),
            hash: img.hash.to_owned(),
        };
        let mut container_id = img.daemonize();
        trim(&mut container_id);
        Self {
            id: container_id,
            img: image,
        }
    }

    /// identify operating system of the image
    pub fn get_os(&self) -> Result<Os, FromUtf8Error> {
        // cat /etc/*-release | grep '^VERSION_ID=' | cut -d= -f 2
        let command = "cat /etc/*-release | grep '^ID=' | cut -d= -f 2";
        let mut name = String::from_utf8(self.execute(command))?;
        trim(&mut name);
        let command = "cat /etc/*-release | grep '^VERSION_ID=' | cut -d= -f 2";
        let mut version = String::from_utf8(self.execute(command))?;
        trim(&mut version);
        Ok(Os { name, version })
    }

    /// stop a running container
    pub fn stop(&self) {
        let command = format!("docker stop {}", self.id);
        execute_command(&command);
    }

    /// execute a command using a container
    pub fn execute(&self, command: &str) -> Vec<u8> {
        let command = format!("docker exec -i {} bash -c \"{}\"", self.id, command);
        execute_command(&command)
    }

    /// copy a content from a container to location on host
    pub fn copy(&self, from_location: &str, to_location: &str) {
        let command = format!("docker cp {}:{} {}", self.id, from_location, to_location);
        execute_command(&command);
    }
}
