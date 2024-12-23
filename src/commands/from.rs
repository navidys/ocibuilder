use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct From {
    // image name or ID
    image: String,
    /// container name
    #[clap(short, long)]
    name: Option<String>,
    /// Using http insecure connection instead of https
    #[clap(short, long)]
    insecure: bool,
    /// Using anonymous credential for registry
    #[clap(short, long)]
    anonymous: bool,
}

impl From {
    pub fn new(image: String, name: Option<String>, insecure: bool, anonymous: bool) -> Self {
        Self {
            image,
            name,
            insecure,
            anonymous,
        }
    }

    pub async fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        let cnt_name = builder
            .from(
                &self.image,
                self.name.clone(),
                &self.insecure,
                &self.anonymous,
            )
            .await?;

        println!("{}", cnt_name);

        Ok(())
    }
}
