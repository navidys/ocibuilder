use std::ffi::OsString;

use clap::Parser;
use log::debug;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Push {
    /// image name
    image: String,
    /// registry destination
    destination: String,
    /// Using http insecure connection instead of https
    #[clap(short, long)]
    insecure: bool,

    /// Using anonymous credential for registry
    #[clap(short, long)]
    anonymous: bool,
}

impl Push {
    pub fn new(image: String, destination: String, insecure: bool, anonymous: bool) -> Self {
        Self {
            image,
            destination,
            insecure,
            anonymous,
        }
    }

    pub async fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        debug!("pushing image {} ...", self.image);

        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        builder
            .push(
                &self.image,
                &self.destination,
                &self.insecure,
                &self.anonymous,
            )
            .await?;

        Ok(())
    }
}
