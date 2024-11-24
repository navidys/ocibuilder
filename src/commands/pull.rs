use std::ffi::OsString;

use clap::Parser;
use log::debug;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Pull {
    /// image name
    image: String,
    /// Using http insecure connection instead of https
    #[clap(short, long)]
    insecure: bool,
    /// Using anonymous credential for registry
    #[clap(short, long)]
    anonymous: bool,
}

impl Pull {
    pub fn new(image: String, insecure: bool, anonymous: bool) -> Self {
        Self {
            image,
            insecure,
            anonymous,
        }
    }

    pub async fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        debug!("pulling image...");

        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        builder
            .pull(&self.image, &self.insecure, &self.anonymous)
            .await?;

        Ok(())
    }
}
