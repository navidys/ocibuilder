use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Run {
    /// Path to container runtime directory
    #[clap(short, long)]
    rundir: Option<OsString>,

    /// Enable systemd cgroup manager, rather then use the cgroupfs directly (default true).
    #[clap(short, long, default_value_t = true)]
    pub systemd_cgroup: bool,

    /// container name or ID
    container: String,

    /// command to run
    cmd: Vec<String>,
}

impl Run {
    pub fn new(
        container: String,
        cmd: Vec<String>,
        rundir: Option<OsString>,
        systemd_cgroup: bool,
    ) -> Self {
        Self {
            container,
            cmd,
            rundir,
            systemd_cgroup,
        }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        builder.run(
            &self.container,
            &self.cmd,
            &self.rundir,
            &self.systemd_cgroup,
        )?;

        Ok(())
    }
}