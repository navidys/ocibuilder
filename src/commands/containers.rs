use std::{ffi::OsString, io::Write};

use clap::Parser;
use tabwriter::TabWriter;

use crate::{
    builder,
    error::{BuilderError, BuilderResult},
    utils,
};

#[derive(Parser, Debug)]
pub struct Containers {
    /// Pretty-print containers to JSON
    #[clap(short)]
    json: bool,
}

impl Containers {
    pub fn new(json: bool) -> Self {
        Self { json }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        let containers = builder.container_store().containers()?;

        if self.json {
            match serde_json::to_string_pretty(&containers) {
                Ok(output) => println!("{}", output),
                Err(err) => return Err(BuilderError::SerdeJsonError(err)),
            }

            return Ok(());
        }

        let mut tw = TabWriter::new(std::io::stdout());
        let mut output = "CONTAINER ID\tIMAGE ID\tIMAGE NAME\tCONTAINER NAME\n".to_string();

        for cnt in containers {
            output = format!(
                "{}{:.12}\t{:.12}\t{}\t{}\n",
                output,
                cnt.id(),
                cnt.image_id(),
                cnt.image_name(),
                cnt.name(),
            );
        }

        match write!(&mut tw, "{}", output) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::TabWriterError(err.to_string())),
        }

        match tw.flush() {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::TabWriterError(err.to_string())),
        }

        Ok(())
    }
}
