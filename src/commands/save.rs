use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Save {
    #[clap(short, long, required = true)]
    /// Output file
    output: String,

    ///image name or ID
    image: String,
}

impl Save {
    pub fn new(image: String, output: String) -> Self {
        Self { image, output }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        builder.save(&self.image, &self.output)?;

        Ok(())
    }
}
