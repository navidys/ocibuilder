use std::ffi::OsString;

use clap::Parser;
use log::debug;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Pull {
    image_name: String,
}

impl Pull {
    pub fn new(image_name: String) -> Self {
        Self { image_name }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        debug!("pulling image...");

        let root_dir_path = utils::get_root_dir(root_dir);
        let _builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        Ok(())
    }
}
