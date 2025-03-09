use std::{ffi::OsString, io};

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Reset {
    /// Do not prompt for confirmation
    #[clap(short, long)]
    force: bool,
}

impl Reset {
    pub fn new(force: bool) -> Self {
        Self { force }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);

        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        if self.force {
            builder.reset()?;

            return Ok(());
        }

        println!("WARNING! this will remove all containers, images and layers.");
        println!("Are you sure you want to continue? [y/N]");

        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read input");

        if user_input.to_lowercase() == "y\n" {
            builder.reset()?;
        }

        Ok(())
    }
}
