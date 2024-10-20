use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Config {
    /// Set image author contact information
    #[clap(long, required = false)]
    pub author: Option<String>,

    /// Set default user to run inside containers based on image
    #[clap(long, required = false)]
    pub user: Option<String>,

    /// Set working directory for containers based on image
    #[clap(long, required = false)]
    pub working_dir: Option<String>,

    /// Set stop signal for containers based on image
    #[clap(long, required = false)]
    pub stop_signal: Option<String>,

    /// Set description of how the image was created
    #[clap(long, required = false)]
    pub created_by: Option<String>,

    pub container_id: String,
}

impl Config {
    pub fn new(
        container_id: String,
        author: Option<String>,
        user: Option<String>,
        working_dir: Option<String>,
        stop_signal: Option<String>,
        created_by: Option<String>,
    ) -> Self {
        Self {
            author,
            user,
            working_dir,
            stop_signal,
            created_by,
            container_id,
        }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;
        builder.update_config(self)?;

        Ok(())
    }
}
