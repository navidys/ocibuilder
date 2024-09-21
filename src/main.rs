use std::ffi::OsString;

use clap::{Parser, Subcommand};
use ocibuilder::commands::pull;

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
    /// Pull an image from specified registry
    Pull(pull::Pull),
}

fn main() {
    env_logger::builder().format_timestamp(None).init();

    let opts = Opts::parse();
    let root_dir = opts.root;

    let result = match opts.subcmd {
        SubCommand::Pull(pull) => pull.exec(root_dir),
    };

    match result {
        Ok(_) => {}
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        }
    }
}
