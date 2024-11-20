use log::debug;
use oci_client::{manifest::OciDescriptor, Client, Reference};

use crate::{
    builder::dist_client, error::{BuilderError, BuilderResult}, layer::store::LayerStore, utils::{self, digest}
};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub async fn pull(&self, image_name: &str, insecure: &bool) -> BuilderResult<digest::Digest> {
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
        let client_config = dist_client::build_client_config(insecure)?;

        let client = Client::new(client_config);

        println!("Trying pull image {}...", reference);

        let (manifest, _digest, config) =
            match client.pull_manifest_and_config(&reference, &auth).await {
                Ok((manifest, digest, config)) => (manifest, digest, config),
                Err(err) => return Err(BuilderError::OciDistError(err)),
            };

        let image_digest = utils::digest::Digest::new(&manifest.config.digest)?;

        let mut pull_handlers = Vec::new();
        for layer in &manifest.layers {
            let spawn_ref = reference.clone();
            let spawn_layer = layer.clone();
            let spawn_layerstore = self.layer_store().clone();
            let spawn_client = client.clone();
            let pull_job = tokio::spawn(async move { pull_image_blob(spawn_layerstore, spawn_client, spawn_ref, spawn_layer).await });
            pull_handlers.push(pull_job);
        }

        for phandler in pull_handlers {
            match phandler.await {
                Ok(_) => {},
                Err(err) => return Err(BuilderError::SpawnError(err.to_string())),
            }
        }

        // write image config
        println!("Copying config {:.1$}", image_digest.encoded, 12);
        self.image_store().write_config(&image_digest, &config)?;

        // write image manifest
        println!("Writing manifest to image destination");
        self.image_store()
            .write_manifest(&image_digest, &manifest)?;

        // update images
        self.image_store().write_images(&reference, &image_digest)?;

        self.unlock()?;

        Ok(image_digest)
    }
}

async fn pull_image_blob(layerstore: LayerStore, client: Client, reference: Reference, layer: OciDescriptor) -> BuilderResult<()> {
    let mut blob: Vec<u8> = Vec::new();
    debug!("pull blob: {}", layer.digest);

    let layer_digest = utils::digest::Digest::new(&layer.digest)?;
    println!("Copying blob {:.1$}", layer_digest.encoded, 12);

    match client.pull_blob(&reference, &layer, &mut blob).await {
        Ok(_) => {
            layerstore.write_blob(&layer_digest, &blob)?;

            // create layer overlay dir
            layerstore.create_layer_overlay_dir(&layer_digest)?;

            // add pulled layer to layers
            layerstore.add_layer_desc(&layer)?;

            // extract content to overlay diff
            let over_diff = layerstore.overlay_diff_path(&layer_digest);
            let buf = flate2::read::GzDecoder::new(blob.as_slice());
            let mut blob_archive = tar::Archive::new(buf);
            blob_archive.set_preserve_ownerships(false);
            match blob_archive.unpack(over_diff) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::ArchiveError(err.to_string())),
            }
        }
        Err(err) => return Err(BuilderError::OciDistError(err)),
    }

    Ok(())
}
