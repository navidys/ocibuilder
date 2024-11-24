use std::io::Write;
use std::{ffi::OsString, time::Duration};

use chrono::Local;
use clap::Parser;
use indicatif::{DecimalBytes, HumanDuration};
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
    #[clap(short, long)]
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
        let mut output = "REPOSITORY\tTAG\tIMAGE ID\tCREATED\tSIZE\n".to_string();

        for img in images {
            let image_size = DecimalBytes(u64::try_from(img.size()).unwrap_or_default());
            let created_time: chrono::DateTime<chrono::Utc> = img.created().to_utc();
            let current_time = Local::now();
            let diff_time = current_time.signed_duration_since(created_time);
            let diff_time_seconds = u64::from_ne_bytes(diff_time.num_seconds().to_ne_bytes());
            let image_created = HumanDuration(Duration::new(diff_time_seconds, 0));
            if img.repository() == "/" && img.tag().is_empty() {
                output = format!(
                    "{}<none>\t<none>\t{:.12}\t{} ago\t{}\n",
                    output,
                    img.id(),
                    image_created,
                    image_size,
                );
            } else {
                let img_ref = format!("{}:{}", img.repository(), img.tag());
                let reference: Reference = match img_ref.parse() {
                    Ok(img_ref) => img_ref,
                    Err(err) => return Err(BuilderError::InvalidImageName(img_ref, err)),
                };
                let img_name = format!("{}/{}", reference.registry(), reference.repository());
                output = format!(
                    "{}{}\t{}\t{:.12}\t{} ago\t{}\n",
                    output,
                    img_name,
                    img.tag(),
                    img.id(),
                    image_created,
                    image_size,
                );
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
