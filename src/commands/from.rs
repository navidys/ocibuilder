use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct From {
    image_name: String,
    /// container name
    #[clap(short, long)]
    name: Option<String>,
}

impl From {
    pub fn new(image_name: String, name: Option<String>) -> Self {
        Self { image_name, name }
    }

    pub async fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        let cnt_name = builder.from(&self.image_name, self.name.clone()).await?;
        println!("{}", cnt_name);

        Ok(())
    }
}
