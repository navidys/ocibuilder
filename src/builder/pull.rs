use oci_client::{Client, Reference};

use crate::{
    builder::dist_client,
    error::{BuilderError, BuilderResult},
    utils,
};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub async fn pull(&self, image_name: &str) -> BuilderResult<()> {
        let reference: Reference = match image_name.parse() {
            Ok(img_ref) => img_ref,
            Err(err) => return Err(BuilderError::InvalidImageName(image_name.to_string(), err)),
        };

        let auth = dist_client::build_auth(&reference, true)?;
        let client_config = dist_client::build_client_config(true)?;

        let client = Client::new(client_config);

        println!("Trying pull image {}...", reference);

        match client.pull_manifest_and_config(&reference, &auth).await {
            Ok((manifest, digest, config)) => {
                let image_digest = utils::digest::Digest::new(&digest)?;
                self.image_store()
                    .write_manifest(&image_digest, &manifest)?;
                self.image_store().write_config(&image_digest, &config)?;
            }
            Err(err) => return Err(BuilderError::OciDistError(err)),
        }

        Ok(())
    }
}
