use std::{
    fs::{self, File},
    io::{self, copy, BufReader},
    path::Path,
};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use log::{debug, warn};
use oci_client::{
    manifest::{self, OciDescriptor, OciImageManifest},
    Reference,
};
use tar::Archive;

use crate::{
    container::containers::Container,
    error::{BuilderError, BuilderResult},
    image::images,
    utils::{self, digest},
};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn commit(&self, container: &str, name: Option<String>) -> BuilderResult<digest::Digest> {
        self.lock()?;

        let cnt = self.container_store().container_exist(container)?;
        let cnt_id = self.container_store().container_digest(container)?;
        let top_layer = cnt.top_layer();
        let top_layer_digest = utils::digest::Digest::new(&format!("sha256:{}", top_layer))?;
        debug!("top layer digest {}", top_layer_digest);

        let diff_path = self.layer_store().overlay_diff_path(&top_layer_digest);

        let is_empty_layer = utils::is_empty_dir(&diff_path)?;

        let mut config = self.container_store().get_builder_config(&cnt_id)?;
        config.created = Some(chrono::Utc::now());

        if cnt.image_name() != images::SCRATCH_IMAGE_NAME {
            let cnt_image_id = utils::digest::Digest::new(&format!("sha256:{}", cnt.image_id()))?;
            let img_manifest = self.image_store().get_manifest(&cnt_image_id)?;

            for blob in img_manifest.layers {
                let blob_digest = digest::Digest::new(&blob.digest)?;
                let blob_path = self.layer_store().blob_path(&blob_digest);
                if !blob_path.exists() {
                    return Err(BuilderError::IoError(
                        blob_path,
                        io::Error::from(io::ErrorKind::NotFound),
                    ));
                }

                println!("Copying blob {:.12}", blob_digest.encoded);
            }
        }

        let mut layer_archive_path = self.tmp_dir().clone();
        let mut layer_oci_desc: Option<OciDescriptor> = None;
        if !is_empty_layer {
            debug!("not an empty top layer");

            let archive_name = format!("{:.12}-top-diff.tar", cnt.id());
            layer_archive_path.push(archive_name);

            // create tar archive of top layer
            let layer_tar_id = self.create_layer_tar_archive(&diff_path, &layer_archive_path)?;

            // add compress, calculate hash and add top layer to overlay-layers
            let tmp_gz_output = self.layer_store().blob_path(&top_layer_digest);
            let layer_tar_gz_digest =
                self.compress_layer_archive(&layer_archive_path, &tmp_gz_output)?;

            println!("Copying blob {:.12}", layer_tar_gz_digest.encoded);

            // update layers.json
            let layer_gz_output = self.layer_store().blob_path(&layer_tar_gz_digest);
            let layer_size = utils::file_size(&layer_gz_output)?;
            let new_layer_oci_desc = manifest::OciDescriptor {
                size: layer_size,
                media_type: manifest::IMAGE_LAYER_GZIP_MEDIA_TYPE.to_string(),
                digest: layer_tar_gz_digest.to_string(),
                urls: None,
                annotations: None,
            };
            self.layer_store().add_layer_desc(&new_layer_oci_desc)?;
            layer_oci_desc = Some(new_layer_oci_desc.clone());

            config.rootfs.diff_ids.push(layer_tar_id.to_string());
        } else {
            debug!("empty top layer");
        }

        match serde_json::to_string(&config) {
            Ok(output) => {
                self.image_store().write_config(&cnt_id, &output)?;
            }
            Err(err) => return Err(BuilderError::SerdeJsonError(err)),
        }

        // overlay-images 3 - calc image new ID and rename
        let tmp_image_cfg_path = self.image_store().config_path(&cnt_id);
        let new_image_id = utils::compute_sha256_hash_of_file(&tmp_image_cfg_path)?;
        let new_image_id_digest = utils::digest::Digest::new(&new_image_id)?;
        println!("Copying config {:.12}", new_image_id_digest.encoded);

        let tmp_image_cfg_dir = self.image_store().config_path_dir(&cnt_id);
        let new_image_cfg_dir = self.image_store().config_path_dir(&new_image_id_digest);
        match fs::rename(tmp_image_cfg_dir, &new_image_cfg_dir) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(new_image_cfg_dir, err)),
        }

        // overlay-images 4- create image manifest
        let new_image_manifest =
            self.new_image_manifest(&cnt, &new_image_id_digest, &layer_oci_desc)?;

        println!("Writing manifest to image destination");
        self.image_store()
            .write_manifest(&new_image_id_digest, &new_image_manifest)?;

        // overlay-images 5- create image name
        let new_image_reference = self.new_image_reference(name, &new_image_id_digest)?;

        // overlay-images 6- update images.json
        self.image_store()
            .write_images(&new_image_reference, &new_image_id_digest)?;

        // remove tmp content
        if layer_archive_path.is_file() {
            match fs::remove_file(&layer_archive_path) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(layer_archive_path, err)),
            }
        }

        self.unlock()?;

        Ok(new_image_id_digest)
    }

    fn new_image_manifest(
        &self,
        cnt: &Container,
        image_id: &digest::Digest,
        new_layer: &Option<OciDescriptor>,
    ) -> BuilderResult<OciImageManifest> {
        debug!("new image manifest");

        let mut image_annotations = None;
        let mut new_image_layers = Vec::<OciDescriptor>::new();
        let img_cfg = self.image_store().config_path(image_id);
        let config_size = utils::file_size(&img_cfg)?;

        if cnt.image_name() != images::SCRATCH_IMAGE_NAME {
            let cnt_image_id = utils::digest::Digest::new(&format!("sha256:{}", cnt.image_id()))?;
            let img_manifest = self.image_store().get_manifest(&cnt_image_id)?;
            image_annotations = img_manifest.annotations;

            for layer in img_manifest.layers {
                let mut layer_type = manifest::IMAGE_LAYER_GZIP_MEDIA_TYPE.to_string();

                if layer.media_type == manifest::IMAGE_DOCKER_LAYER_GZIP_MEDIA_TYPE {
                    layer_type = manifest::IMAGE_LAYER_GZIP_MEDIA_TYPE.to_string()
                } else if layer.media_type == manifest::IMAGE_DOCKER_LAYER_TAR_MEDIA_TYPE {
                    layer_type = manifest::IMAGE_LAYER_MEDIA_TYPE.to_string()
                } else {
                    warn!("unknown layer type: {}", layer.media_type);
                }

                new_image_layers.push(OciDescriptor {
                    size: layer.size,
                    media_type: layer_type,
                    digest: layer.digest,
                    urls: layer.urls,
                    annotations: layer.annotations,
                })
            }
        }

        if new_layer.is_some() {
            new_image_layers.push(new_layer.clone().unwrap_or_default());
        }

        let new_image_config = OciDescriptor {
            media_type: manifest::IMAGE_CONFIG_MEDIA_TYPE.to_string(),
            digest: image_id.to_string(),
            size: config_size,
            urls: None,
            annotations: None,
        };

        let new_image_manifest = manifest::OciImageManifest {
            schema_version: 2,
            media_type: Some(manifest::OCI_IMAGE_MEDIA_TYPE.to_string()),
            config: new_image_config,
            layers: new_image_layers,
            artifact_type: None,
            annotations: image_annotations,
        };

        Ok(new_image_manifest)
    }

    fn new_image_reference(
        &self,
        name: Option<String>,
        img_digest: &digest::Digest,
    ) -> BuilderResult<Reference> {
        debug!("new image reference");

        let new_image_reference: Reference;
        if name.is_none() {
            new_image_reference =
                Reference::with_digest("".to_string(), "".to_string(), img_digest.to_string());
        } else {
            let mut img_name = name.unwrap_or_default();
            match img_name.split_once('/') {
                None => {
                    img_name = format!("localhost/{}", img_name);
                    new_image_reference = match img_name.parse() {
                        Ok(img_ref) => img_ref,
                        Err(err) => {
                            return Err(BuilderError::InvalidImageName(img_name.to_owned(), err))
                        }
                    };
                }
                Some((_, _)) => {
                    new_image_reference = match img_name.parse() {
                        Ok(img_ref) => img_ref,
                        Err(err) => {
                            return Err(BuilderError::InvalidImageName(img_name.to_owned(), err))
                        }
                    };
                }
            }
        }

        Ok(new_image_reference)
    }

    fn compress_layer_archive(&self, src: &Path, dest: &Path) -> BuilderResult<digest::Digest> {
        debug!("compress and archive layer");

        let layer_tar_archive = match File::open(src) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::IoError(src.to_owned(), err)),
        };

        let mut lpath = self.layer_store().lstore_path().clone();
        lpath.push("sha256/");

        match fs::create_dir_all(&lpath) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(lpath, err)),
        }

        let mut gz_input = BufReader::new(layer_tar_archive);
        let gz_output = match File::create(dest) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::IoError(dest.to_owned(), err)),
        };

        let mut gz_encoder = GzEncoder::new(gz_output, Compression::default());
        match copy(&mut gz_input, &mut gz_encoder) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(dest.to_owned(), err)),
        }

        match gz_encoder.finish() {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(dest.to_owned(), err)),
        }

        let layer_tar_gz_id = utils::compute_sha256_hash_of_file(dest)?;
        debug!("tar gz layer id: {}", layer_tar_gz_id);

        let layer_tar_gz_digest = digest::Digest::new(layer_tar_gz_id.as_str())?;
        let new_gz_output = self.layer_store().blob_path(&layer_tar_gz_digest);
        if new_gz_output.exists() {
            match fs::remove_file(dest) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(dest.to_path_buf(), err)),
            }
        } else {
            match fs::rename(dest, &new_gz_output) {
                Ok(_) => {
                    self.layer_store()
                        .create_layer_overlay_dir(&layer_tar_gz_digest)?;

                    let over_diff = self.layer_store().overlay_diff_path(&layer_tar_gz_digest);

                    let layer_archive = match File::open(new_gz_output) {
                        Ok(f) => f,
                        Err(err) => return Err(BuilderError::IoError(src.to_owned(), err)),
                    };

                    let tar = GzDecoder::new(layer_archive);
                    let mut archive = Archive::new(tar);
                    archive.set_preserve_ownerships(false);
                    match archive.unpack(over_diff) {
                        Ok(_) => {}
                        Err(err) => return Err(BuilderError::ArchiveError(err.to_string())),
                    }
                }
                Err(err) => return Err(BuilderError::IoError(new_gz_output, err)),
            }
        }

        Ok(layer_tar_gz_digest)
    }

    fn create_layer_tar_archive(&self, src: &Path, dest: &Path) -> BuilderResult<digest::Digest> {
        debug!("create tar archive from layer");

        let layer_archive = match File::create(dest) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::IoError(dest.to_owned(), err)),
        };

        let mut tar = tar::Builder::new(layer_archive);

        match tar.append_dir_all("", src) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(dest.to_owned(), err)),
        }

        match tar.finish() {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(dest.to_owned(), err)),
        }

        let layer_tar_id = utils::compute_sha256_hash_of_file(dest)?;
        debug!("tar layer id: {}", layer_tar_id);

        let layer_tar_digest = digest::Digest::new(layer_tar_id.as_str())?;

        Ok(layer_tar_digest)
    }
}
