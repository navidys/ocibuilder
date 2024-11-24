use std::sync::mpsc;
use std::{thread, time::Duration};

use indicatif::{DecimalBytes, MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, error};
use oci_client::{manifest, Client, Reference};

use crate::{
    builder::dist_client,
    error::{BuilderError, BuilderResult},
    layer::store::LayerStore,
    utils::{self, digest},
};

use rand::Rng;

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

        let m: MultiProgress = MultiProgress::new();

        let mut pull_handlers: Vec<tokio::task::JoinHandle<Result<(), BuilderError>>> = Vec::new();
        let mut threads = vec![];
        let mut image_oci_manifest = manifest.clone();
        let mut image_oci_disctriptor = Vec::new();

        for layer in &manifest.layers {
            image_oci_disctriptor.push(manifest::OciDescriptor {
                media_type: manifest::IMAGE_LAYER_GZIP_MEDIA_TYPE.to_string(),
                digest: layer.digest.clone(),
                size: layer.size,
                urls: layer.urls.clone(),
                annotations: layer.annotations.clone(),
            });

            let layer_digest = utils::digest::Digest::new(&layer.digest)?;
            let layer_size = DecimalBytes(u64::try_from(layer.size).unwrap_or_default());

            let (tx, rx) = mpsc::channel();

            let spinner_bar = ProgressBar::new_spinner();
            let mspinner_bar = m.clone().add(spinner_bar);
            let style_message = format!(
                "Copying blob {:.12} {{msg}} {{prefix:.bold}}{{spinner:.yellow}}",
                layer_digest.encoded,
            );
            let style = match ProgressStyle::with_template(&style_message) {
                Ok(st) => st,
                Err(err) => return Err(BuilderError::TerminalMultiProgressError(err.to_string())),
            };

            mspinner_bar.enable_steady_tick(Duration::from_millis(100));
            mspinner_bar.set_style(style.clone());
            mspinner_bar.set_message(format!("{} in progress", layer_size));
            threads.push(thread::spawn(move || loop {
                match rx.recv() {
                    Ok(_) => {
                        mspinner_bar.enable_steady_tick(Duration::from_millis(100));
                        mspinner_bar.set_style(style.clone());
                        mspinner_bar.set_message(format!("{} in progress", layer_size));
                        thread::sleep(
                            rand::thread_rng()
                                .gen_range(Duration::from_secs(1)..Duration::from_secs(5)),
                        );
                    }
                    Err(err) => {
                        debug!("spinner rx received: {:?}", err);
                        mspinner_bar.finish_with_message("done");
                        break;
                    }
                }
            }));

            let spawn_ref = reference.clone();
            let spawn_layer = layer.clone();
            let spawn_layerstore = self.layer_store().clone();
            let spawn_client = client.clone();
            let pull_job = tokio::spawn(async move {
                pull_image_blob(spawn_layerstore, spawn_client, spawn_ref, spawn_layer, tx).await
            });
            pull_handlers.push(pull_job);
        }

        for phandler in pull_handlers {
            match phandler.await {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::SpawnError(err.to_string())),
            }
        }

        for sthread in threads {
            if !sthread.is_finished() {
                match sthread.join() {
                    Ok(_) => {}
                    Err(err) => {
                        error!("spinner thread join error: {:?}", err);

                        return Err(BuilderError::SpawnError(
                            "cannot stop spinner thread".to_string(),
                        ));
                    }
                }
            }
        }

        image_oci_manifest.layers = image_oci_disctriptor;

        for layer in &image_oci_manifest.layers {
            debug!("adding layers to layerstore");

            self.layer_store().add_layer_desc(layer)?;
        }

        // write image config
        println!("Copying config {:.1$}", image_digest.encoded, 12);
        self.image_store().write_config(&image_digest, &config)?;

        // write image manifest
        println!("Writing manifest to image destination");
        self.image_store()
            .write_manifest(&image_digest, &image_oci_manifest)?;

        // update images
        let mut image_size = self.calculate_image_layers_size(image_oci_manifest.layers)?;
        image_size += image_oci_manifest.config.size;

        let image_config = self.image_store().get_config(&image_digest)?;

        self.image_store().write_images(
            &reference,
            &image_digest,
            &image_size,
            &image_config.created.unwrap_or_default(),
        )?;

        self.unlock()?;

        Ok(image_digest)
    }
}

async fn pull_image_blob(
    layerstore: LayerStore,
    client: Client,
    reference: Reference,
    layer: manifest::OciDescriptor,
    tx: mpsc::Sender<()>,
) -> BuilderResult<()> {
    let mut blob: Vec<u8> = Vec::new();
    debug!("pull blob: {}", layer.digest);

    let layer_digest = utils::digest::Digest::new(&layer.digest)?;

    match client.pull_blob(&reference, &layer, &mut blob).await {
        Ok(_) => {}
        Err(err) => return Err(BuilderError::OciDistError(err)),
    };

    layerstore.write_blob(&layer_digest, &blob)?;

    // create layer overlay dir
    layerstore.create_layer_overlay_dir(&layer_digest)?;

    // extract content to overlay diff
    let over_diff = layerstore.overlay_diff_path(&layer_digest);
    let buf = flate2::read::GzDecoder::new(blob.as_slice());
    let mut blob_archive = tar::Archive::new(buf);
    blob_archive.set_preserve_ownerships(false);
    match blob_archive.unpack(over_diff) {
        Ok(_) => {}
        Err(err) => return Err(BuilderError::ArchiveError(err.to_string())),
    }

    match tx.send(()) {
        Ok(_) => {}
        Err(err) => return Err(BuilderError::SpawnError(err.to_string())),
    }

    Ok(())
}
