use log::debug;
use oci_client::{Client, Reference};

use crate::{
    builder::dist_client,
    error::{BuilderError, BuilderResult},
    utils::{self, digest},
};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub async fn pull(&self, image_name: &str) -> BuilderResult<digest::Digest> {
        self.lock()?;

        let reference: Reference = match image_name.parse() {
            Ok(img_ref) => img_ref,
            Err(err) => return Err(BuilderError::InvalidImageName(image_name.to_string(), err)),
        };

        match self.image_store().image_digest(&reference.to_string()) {
            Ok(dg) => return Ok(dg),
            Err(err) => {
                if err.to_string() != BuilderError::ImageNotFound(reference.to_string()).to_string()
                {
                    return Err(err);
                }
            }
        }

        let auth = dist_client::build_auth(&reference, true)?;
        let client_config = dist_client::build_client_config(true)?;

        let client = Client::new(client_config);

        println!("Trying pull image {}...", reference);

        let (manifest, digest, config) =
            match client.pull_manifest_and_config(&reference, &auth).await {
                Ok((manifest, digest, config)) => (manifest, digest, config),
                Err(err) => return Err(BuilderError::OciDistError(err)),
            };

        let image_digest = utils::digest::Digest::new(&digest)?;

        for layer in &manifest.layers {
            let mut blob: Vec<u8> = Vec::new();
            debug!("pull blob: {}", layer.digest);

            let layer_digest = utils::digest::Digest::new(&layer.digest)?;
            println!("Copying blob {:.1$}", layer_digest.encoded, 12);

            match client.pull_blob(&reference, &layer, &mut blob).await {
                Ok(_) => {
                    self.layer_store().write_blob(&layer_digest, &blob)?;

                    // create layer overlay dir
                    self.layer_store().create_layer_overlay_dir(&layer_digest)?;

                    // add pulled layer to layers
                    self.layer_store().add_layer_desc(layer)?;
                }
                Err(err) => return Err(BuilderError::OciDistError(err)),
            }
        }

        // write image config
        println!("Copying config {:.1$}", image_digest.encoded, 12);
        self.image_store().write_config(&image_digest, &config)?;

        // write image manifest
        println!("Writing image manifest");
        self.image_store()
            .write_manifest(&image_digest, &manifest)?;

        // update images
        self.image_store().write_images(&reference, &image_digest)?;

        self.unlock()?;

        Ok(image_digest)
    }
}
