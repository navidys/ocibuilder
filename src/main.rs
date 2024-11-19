use std::ffi::OsString;

use clap::{Parser, Subcommand};
use ocibuilder::commands::{
    commit, config, containers, from, images, mount, pull, reset, rm, rmi, run, save, umount,
};

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
    /// Create an image from a working container
    Commit(commit::Commit),

    /// Modifies the configuration values which will be saved to the image.
    Config(config::Config),

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

    /// Remove one or more working containers
    Rm(rm::RemoveContainer),

    /// Remove one or more images from local storage.
    Rmi(rmi::RemoveImage),

    /// Run a command inside of the container.
    Run(run::Run),

    /// Save an image to oci-archive
    Save(save::Save),

    /// Mounts a working container's root filesystem for manipulation
    Mount(mount::Mount),

    /// Unmounts the root file system of the specified working containers
    Umount(umount::Umount),
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    let opts = Opts::parse();
    let root_dir = opts.root;

    let result = match opts.subcmd {
        SubCommand::Commit(commit) => commit.exec(root_dir),
        SubCommand::Config(config) => config.exec(root_dir),
        SubCommand::From(from) => from.exec(root_dir).await,
        SubCommand::Images(images) => images.exec(root_dir),
        SubCommand::Containers(containers) => containers.exec(root_dir),
        SubCommand::Pull(pull) => pull.exec(root_dir).await,
        SubCommand::Rm(rm) => rm.exec(root_dir),
        SubCommand::Rmi(rmi) => rmi.exec(root_dir),
        SubCommand::Run(run) => run.exec(root_dir),
        SubCommand::Reset(reset) => reset.exec(root_dir),
        SubCommand::Save(save) => save.exec(root_dir),
        SubCommand::Mount(mount) => mount.exec(root_dir),
        SubCommand::Umount(umount) => umount.exec(root_dir),
    };

    match result {
        Ok(_) => {}
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        }
    }
}
