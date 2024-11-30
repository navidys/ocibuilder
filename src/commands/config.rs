use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Config {
    /// Add an entry for this operation to the image's history.
    #[clap(long, required = false)]
    pub add_history: bool,

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

    /// Set the default command to run for containers based on the image
    #[clap(long, required = false)]
    pub cmd: Option<String>,

    /// Set entry point for containers based on image
    #[clap(long, required = false)]
    pub entrypoint: Option<String>,

    /// Add environment variable to be set when running containers based on image
    #[clap(long, required = false)]
    pub env: Option<String>,

    /// Add image configuration label e.g. label=value
    #[clap(long, required = false)]
    pub label: Option<String>,

    /// Add port to expose when running containers based on image
    #[clap(long, required = false)]
    pub port: Option<String>,

    pub container_id: String,
}

impl Config {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        container_id: String,
        author: Option<String>,
        user: Option<String>,
        working_dir: Option<String>,
        stop_signal: Option<String>,
        created_by: Option<String>,
        cmd: Option<String>,
        entrypoint: Option<String>,
        env: Option<String>,
        label: Option<String>,
        port: Option<String>,
        add_history: bool,
    ) -> Self {
        Self {
            add_history,
            author,
            user,
            working_dir,
            stop_signal,
            created_by,
            cmd,
            entrypoint,
            env,
            label,
            port,
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
