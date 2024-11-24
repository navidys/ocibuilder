use std::ffi::OsString;

use clap::Parser;

use crate::{
    builder,
    error::{BuilderError, BuilderResult},
    utils,
};

#[derive(Parser, Debug)]
pub struct Inspect {
    // image or container name or ID
    name: String,
}

impl Inspect {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        let inspect_data = builder.inspect(&self.name)?;
        match serde_json::to_string_pretty(&inspect_data) {
            Ok(output) => println!("{}", output),
            Err(err) => return Err(BuilderError::SerdeJsonError(err)),
        }

        Ok(())
    }
}
