use log::info;

use anyhow::Result;
use console::{style, Emoji};

mod cli;
mod docker;
mod logger;
mod pkgdiff;
mod progress;
mod sources;
mod templater;
mod utils;

use cli::Args;
use docker::is_docker_installed;
use docker::Container;
use logger::log_config;
use sources::Sources;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

fn main() -> Result<()> {
    log_config();
    let args: Args = argh::from_env();
    if !is_docker_installed() {
        println!("docker client not found on the host, exiting...");
        std::process::exit(1);
    }
    println!("\n");
    println!(
        " {} {} setting up container for image :: {} [{}]",
        style("[1/3]").bold().dim(),
        TRUCK,
        &args.image,
        style("✔").green()
    );
    let container = Container::start(&args.image);
    let os = container.get_os()?;
    info!("image os name :: {}", &os.name);
    info!("image os version :: {}", &os.version);
    info!("image hash: {}", &container.img.hash);
    println!(
        " {} {} fetching metadata for container :: {} [{}]",
        style("[2/3]").bold().dim(),
        LOOKING_GLASS,
        &container.id,
        style("✔").green()
    );
    Sources::generate(&container, &os, args.seperate)?;
    println!(
        " {} {}  stopping container :: {} [{}]",
        style("[3/3]").bold().dim(),
        SPARKLE,
        &container.id,
        style("✔").green()
    );
    container.stop();
    Ok(())
}
