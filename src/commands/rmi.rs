use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct RemoveImage {
    /// image name(s) or ID(s)
    image: Vec<String>,

    /// force removal of the image and any containers using the image
    #[clap(short, long)]
    force: bool,
}

impl RemoveImage {
    pub fn new(image: Vec<String>, force: bool) -> Self {
        Self { image, force }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        builder.rmi(&self.image, &self.force)?;

        Ok(())
    }
}
