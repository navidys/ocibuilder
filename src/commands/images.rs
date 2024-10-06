use std::ffi::OsString;
use std::io::Write;

use clap::Parser;
use oci_client::Reference;
use tabwriter::TabWriter;

use crate::{
    builder,
    error::{BuilderError, BuilderResult},
    utils,
};

#[derive(Parser, Debug)]
pub struct Images {
    /// Pretty-print containers to JSON
    #[clap(short)]
    json: bool,
}

impl Images {
    pub fn new(json: bool) -> Self {
        Self { json }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        let images = builder.image_store().images()?;
        if self.json {
            match serde_json::to_string_pretty(&images) {
                Ok(output) => println!("{}", output),
                Err(err) => return Err(BuilderError::SerdeJsonError(err)),
            }

            return Ok(());
        }

        let mut tw = TabWriter::new(std::io::stdout());
        let mut output = "REPOSITORY\tTAG\tIMAGE ID\n".to_string();

        for img in images {
            if img.name().is_some() {
                let reference: Reference = match img.name().unwrap_or_default().parse() {
                    Ok(img_ref) => img_ref,
                    Err(err) => {
                        return Err(BuilderError::InvalidImageName(
                            img.name().unwrap_or_default(),
                            err,
                        ))
                    }
                };

                let img_name = format!("{}/{}", reference.registry(), reference.repository());
                output = format!(
                    "{}{}\t{}\t{:.12}\n",
                    output,
                    img_name,
                    reference.tag().unwrap_or_default(),
                    img.id(),
                );
            } else {
                output = format!("{}<none>\t<none>\t{:.12}\n", output, img.id());
            }
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
