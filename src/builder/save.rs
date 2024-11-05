use log::debug;
use oci_client::{
    annotations,
    manifest::{self, ImageIndexEntry, OciDescriptor, OciImageIndex},
};
use oci_spec::image::{version, OciLayoutBuilder};

use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::{
    error::{BuilderError, BuilderResult},
    utils::{self, digest},
};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn save(&self, image: &str, output: &str) -> BuilderResult<()> {
        let output_path = Path::new(output);
        if output_path.exists() {
            return Err(BuilderError::ImageArchiveExits(output_path.to_path_buf()));
        }

        self.lock()?;

        let image_digest = self.image_store().image_digest(image)?;
        let image_manifest = self.image_store().get_manifest(&image_digest)?;
        let image_manifest_path = self.image_store().manifest_path(&image_digest);

        let (output_tmp_dir, output_tmp_blobs_dir) = self.create_oci_layout(&image_digest)?;

        // write image manifest
        self.write_image_manifest(&output_tmp_blobs_dir, &image_digest)?;

        // write image config
        let image_config_digest = utils::digest::Digest::new(&image_manifest.config.digest)?;
        self.write_image_config(&output_tmp_blobs_dir, &image_config_digest)?;

        // write layers
        self.write_layers(&output_tmp_blobs_dir, image_manifest.layers.to_owned())?;

        // write image index
        self.write_index(&output_tmp_dir, &image_digest, &image_manifest_path)?;

        // archive content
        self.create_tar_archive(&output_tmp_dir, output)?;

        // remove tmp directory
        match fs::remove_dir_all(&output_tmp_dir) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(output_tmp_dir, err)),
        }

        self.unlock()?;

        Ok(())
    }

    fn create_tar_archive(&self, src: &Path, output: &str) -> BuilderResult<()> {
        let output_path = PathBuf::from(output);
        let archive_file = match File::create(&output_path) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::IoError(output_path, err)),
        };

        let mut tar: tar::Builder<File> = tar::Builder::new(archive_file);

        match tar.append_dir_all("", src) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(output_path.to_owned(), err)),
        }

        match tar.finish() {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(output_path.to_owned(), err)),
        }

        Ok(())
    }

    fn create_oci_layout(
        &self,
        image_digest: &digest::Digest,
    ) -> BuilderResult<(PathBuf, PathBuf)> {
        let mut output_tmp_dir = self.tmp_dir().to_owned();
        output_tmp_dir.push(&image_digest.encoded);

        let mut output_tmp_blobs_dir = output_tmp_dir.clone();
        output_tmp_blobs_dir.push("blobs/");

        // first remove if exists
        if output_tmp_dir.exists() {
            match fs::remove_dir_all(&output_tmp_dir) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(output_tmp_dir, err)),
            }
        }

        match fs::create_dir_all(&output_tmp_blobs_dir) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(output_tmp_blobs_dir, err)),
        }

        let mut oci_layout_path = output_tmp_dir.clone();
        oci_layout_path.push("oci-layout");

        let oci_layout = match OciLayoutBuilder::default()
            .image_layout_version(version())
            .build()
        {
            Ok(l) => l,
            Err(err) => return Err(BuilderError::OciSpecError(err)),
        };

        match oci_layout.to_file(oci_layout_path) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::OciSpecError(err)),
        }

        Ok((output_tmp_dir, output_tmp_blobs_dir))
    }

    fn write_index(
        &self,
        dest: &Path,
        img_digest: &digest::Digest,
        manifest_path: &Path,
    ) -> BuilderResult<()> {
        let manifest_digest_str = utils::compute_sha256_hash_of_file(manifest_path)?;
        let manifest_digest = utils::digest::Digest::new(&manifest_digest_str)?;
        let manifest_size = utils::file_size(manifest_path)?;

        let image = self.image_store().image(img_digest)?;
        let mut index_annotations: BTreeMap<String, String> = BTreeMap::new();
        let mut index_annotations_opt: Option<BTreeMap<String, String>> = None;

        if !image.tag().is_empty() && image.tag() != "latest" {
            index_annotations.insert(
                annotations::ORG_OPENCONTAINERS_IMAGE_REF_NAME.to_string(),
                image.tag(),
            );
            index_annotations_opt = Some(index_annotations);
        }

        let image_index_entry = ImageIndexEntry {
            media_type: manifest::OCI_IMAGE_MEDIA_TYPE.to_string(),
            digest: manifest_digest.to_string(),
            size: manifest_size,
            platform: None,
            annotations: index_annotations_opt,
        };

        let image_index = OciImageIndex {
            schema_version: 2,
            media_type: Some(manifest::OCI_IMAGE_INDEX_MEDIA_TYPE.to_string()),
            manifests: vec![image_index_entry],
            annotations: None,
        };

        let mut image_index_path = dest.to_path_buf();
        image_index_path.push("index.json");

        let image_index_file = match File::create(&image_index_path) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::IoError(image_index_path, err)),
        };

        match serde_json::to_writer(image_index_file, &image_index) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::SerdeJsonError(err)),
        }

        debug!("writing image index to oci archive");

        Ok(())
    }

    fn write_layers(&self, dest: &Path, layers: Vec<OciDescriptor>) -> BuilderResult<()> {
        for layer in layers {
            let layer_dg = utils::digest::Digest::new(&layer.digest)?;
            let layer_src_path = self.layer_store().blob_path(&layer_dg);
            let mut layer_output_dest = dest.to_path_buf();
            layer_output_dest.push(&layer_dg.encoded);

            debug!("copy image layer {:.12}", layer_dg.encoded);
            match fs::copy(&layer_src_path, &layer_output_dest) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(layer_output_dest, err)),
            }
        }

        Ok(())
    }

    fn write_image_config(&self, dest: &Path, dg: &digest::Digest) -> BuilderResult<()> {
        let image_config_src_path = self.image_store().config_path(dg);

        let mut config_output_file = dest.to_path_buf();
        config_output_file.push(&dg.encoded);

        debug!("copy image config {:.12}", dg.encoded);
        match fs::copy(&image_config_src_path, &config_output_file) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(config_output_file, err)),
        }

        Ok(())
    }

    fn write_image_manifest(&self, dest: &Path, dg: &digest::Digest) -> BuilderResult<()> {
        let image_manifest_src_path = self.image_store().manifest_path(dg);
        let image_manifest_id = utils::compute_sha256_hash_of_file(&image_manifest_src_path)?;
        let image_manifest_digest = utils::digest::Digest::new(&image_manifest_id)?;

        let mut manifest_output_file = dest.to_path_buf();
        manifest_output_file.push(image_manifest_digest.encoded);

        debug!("copy image manifest {:.12}", dg.encoded);
        match fs::copy(&image_manifest_src_path, &manifest_output_file) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(manifest_output_file, err)),
        }

        Ok(())
    }
}
