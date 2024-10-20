use std::ffi::OsString;

use clap::{Parser, Subcommand};
use ocibuilder::commands::{containers, from, images, pull, reset};

#[derive(Parser, Debug)]
#[clap(version = env!("CARGO_PKG_VERSION"), about)]
struct Opts {
    /// Path to storage root directory
    #[clap(short, long)]
    root: Option<OsString>,

    /// ocibuilder commands
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Start a new build from a new and empty image or an existing image
    From(from::From),

    /// List images in local storage
    Images(images::Images),

    /// List container in local storage
    Containers(containers::Containers),

    /// Pull an image from specified registry
    Pull(pull::Pull),

    /// Reset local storage
    Reset(reset::Reset),
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    let opts = Opts::parse();
    let root_dir = opts.root;

    let result = match opts.subcmd {
        SubCommand::From(from) => from.exec(root_dir).await,
        SubCommand::Images(images) => images.exec(root_dir),
        SubCommand::Containers(containers) => containers.exec(root_dir),
        SubCommand::Pull(pull) => pull.exec(root_dir).await,
        SubCommand::Reset(reset) => reset.exec(root_dir),
    };

    match result {
        Ok(_) => {}
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        }
    }
}
