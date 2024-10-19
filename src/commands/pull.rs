use std::ffi::OsString;

use clap::Parser;
use log::debug;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Pull {
    image_name: String,
    /// Using http insecure connection instead of https
    #[clap(short, long)]
    insecure: bool,
}

impl Pull {
    pub fn new(image_name: String, insecure: bool) -> Self {
        Self {
            image_name,
            insecure,
        }
    }

    pub async fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        debug!("pulling image...");

        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        builder.pull(&self.image_name, &self.insecure).await?;

        Ok(())
    }
}
