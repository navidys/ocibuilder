use std::{fs, sync::mpsc, thread, time::Duration};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, error};
use oci_client::{client, Client, Reference};

use crate::{
    error::{BuilderError, BuilderResult},
    layer::store::LayerStore,
    utils,
};

use rand::Rng;

use super::{dist_client, oci::OCIBuilder};

impl OCIBuilder {
    pub async fn push(
        &self,
        image_name: &str,
        destination: &str,
        insecure: &bool,
        anonymous: &bool,
    ) -> BuilderResult<()> {
        let image_dg = self.image_store().image_digest(image_name)?;
        let image_config_file = self.image_store().get_config(&image_dg)?;
        let image_config = match client::Config::oci_v1_from_config_file(image_config_file, None) {
            Ok(c) => c,
            Err(err) => return Err(BuilderError::OciDistError(err)),
        };

        let image_manifest = self.image_store().get_manifest(&image_dg)?;

        let reference: Reference = match destination.parse() {
            Ok(img_ref) => img_ref,
            Err(err) => return Err(BuilderError::InvalidImageName(image_name.to_string(), err)),
        };

        let auth = dist_client::build_auth(&reference, anonymous)?;
        let client_config = dist_client::build_client_config(insecure)?;
        let client = Client::new(client_config);

        match client.pull_manifest_and_config(&reference, &auth).await {
            Ok((_, _, _)) => {
                return Err(BuilderError::RegistryError(
                    "manifest already exist".to_string(),
                ))
            }
            Err(err) => {
                debug!("{}", err.to_string())
            }
        };

        let m: MultiProgress = MultiProgress::new();
        let mut push_handlers: Vec<tokio::task::JoinHandle<Result<(), BuilderError>>> = Vec::new();
        let mut threads = vec![];

        println!("Trying push image {}...", reference);

        // push layers
        for layer in image_manifest.layers {
            let (tx, rx) = mpsc::channel();
            let layer_digest = utils::digest::Digest::new(&layer.digest)?;

            let spinner_bar = ProgressBar::new_spinner();
            let mspinner_bar = m.clone().add(spinner_bar);
            let style_message = format!(
                "Pushing blob {:.12} {{msg}} {{prefix:.bold}}{{spinner:.yellow}}",
                layer_digest.encoded,
            );
            let style = match ProgressStyle::with_template(&style_message) {
                Ok(st) => st,
                Err(err) => return Err(BuilderError::TerminalMultiProgressError(err.to_string())),
            };

            mspinner_bar.enable_steady_tick(Duration::from_millis(100));
            mspinner_bar.set_style(style.clone());
            mspinner_bar.set_message(format!("{} in progress", layer.size));
            threads.push(thread::spawn(move || loop {
                match rx.recv() {
                    Ok(_) => {
                        mspinner_bar.enable_steady_tick(Duration::from_millis(100));
                        mspinner_bar.set_style(style.clone());
                        mspinner_bar.set_message(format!("{} in progress", layer.size));
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
            let spawn_layer_dg = layer.digest.clone();
            let spawn_layerstore = self.layer_store().clone();
            let spawn_client = client.clone();
            let push_job = tokio::spawn(async move {
                push_blob(
                    spawn_layerstore,
                    spawn_client,
                    spawn_ref,
                    &spawn_layer_dg,
                    tx,
                )
                .await
            });

            push_handlers.push(push_job);
        }

        for phandler in push_handlers {
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

        // push config
        println!("Pushing config ...");
        match client
            .push_blob(
                &reference,
                &image_config.data,
                &image_manifest.config.digest,
            )
            .await
        {
            Ok(c) => debug!("client push config: {}", c),
            Err(err) => return Err(BuilderError::OciDistError(err)),
        }

        // push manifest
        println!("Pushing manifest ...");

        let img_manifest = self.image_store().get_manifest(&image_dg)?;
        match client.push_manifest(&reference, &img_manifest.into()).await {
            Ok(c) => debug!("client push config: {}", c),
            Err(err) => return Err(BuilderError::OciDistError(err)),
        }

        Ok(())
    }
}

async fn push_blob(
    layerstore: LayerStore,
    client: Client,
    reference: Reference,
    layer_dg: &str,
    tx: mpsc::Sender<()>,
) -> BuilderResult<()> {
    let layer_digest = utils::digest::Digest::new(layer_dg)?;
    let layer_path = layerstore.blob_path(&layer_digest);

    let blob_data = match fs::read(&layer_path) {
        Ok(n) => n,
        Err(err) => return Err(BuilderError::IoError(layer_path, err)),
    };

    match client.push_blob(&reference, &blob_data, layer_dg).await {
        Ok(c) => debug!("client push blob: {}", c),
        Err(err) => return Err(BuilderError::OciDistError(err)),
    }

    match tx.send(()) {
        Ok(_) => {}
        Err(err) => return Err(BuilderError::SpawnError(err.to_string())),
    }

    Ok(())
}
