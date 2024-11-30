use std::{fs::File, path::PathBuf};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

use super::store::ContainerStore;

const CONTAINERS_FILENAME: &str = "containers.json";

#[derive(Debug, Deserialize, Serialize)]
pub struct Container {
    id: String,
    image_name: String,
    image_id: String,
    top_layer: String,
    created: String,
    rootfs_diff: Vec<String>,
    name: String,
}

impl Container {
    pub fn new(
        id: String,
        image_name: String,
        image_id: String,
        top_layer: String,
        created: String,
        rootfs_diff: Vec<String>,
        name: String,
    ) -> BuilderResult<Self> {
        Ok(Self {
            id,
            image_name,
            image_id,
            top_layer,
            created,
            rootfs_diff,
            name,
        })
    }

    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    pub fn image_name(&self) -> String {
        self.image_name.to_owned()
    }

    pub fn image_id(&self) -> String {
        self.image_id.to_owned()
    }

    pub fn id(&self) -> String {
        self.id.to_owned()
    }

    pub fn top_layer(&self) -> String {
        self.top_layer.to_owned()
    }

    pub fn rootfs_diff(&self) -> Vec<String> {
        self.rootfs_diff.to_owned()
    }
}

impl ContainerStore {
    pub fn containers(&self) -> BuilderResult<Vec<Container>> {
        let cnt_file_path = self.containers_path();

        let cnt_file = match File::open(&cnt_file_path) {
            Ok(f) => f,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Ok(Vec::new());
                }

                return Err(BuilderError::ContainerStoreError(format!(
                    "{:?}: {:?}",
                    cnt_file_path,
                    err.to_string(),
                )));
            }
        };

        let containers: Vec<Container> = match serde_json::from_reader(cnt_file) {
            Ok(c) => c,
            Err(err) => return Err(BuilderError::ContainerStoreError(err.to_string())),
        };

        Ok(containers)
    }

    pub fn add_rootfs_diff(
        &self,
        cnt_id: &digest::Digest,
        layer_id: &digest::Digest,
    ) -> BuilderResult<()> {
        let containers = self.containers()?;
        let mut updated_containers: Vec<Container> = Vec::new();

        for mut cnt in containers {
            if cnt.id == cnt_id.encoded {
                cnt.rootfs_diff.insert(0, layer_id.to_string());
            }

            updated_containers.push(cnt)
        }

        self.write_containers(updated_containers)
    }

    pub fn write_containers(&self, cnts: Vec<Container>) -> BuilderResult<()> {
        debug!("write containers");

        let cnt_file_path = self.containers_path();
        let cnt_file = match File::create(&cnt_file_path) {
            Ok(f) => f,
            Err(err) => {
                return Err(BuilderError::ContainerStoreError(format!(
                    "{:?}: {:?}",
                    cnt_file_path,
                    err.to_string(),
                )));
            }
        };

        match serde_json::to_writer(cnt_file, &cnts) {
            Ok(_) => {}
            Err(err) => {
                return Err(BuilderError::ContainerStoreError(format!(
                    "{:?}: {:?}",
                    cnt_file_path,
                    err.to_string(),
                )));
            }
        }

        Ok(())
    }

    pub fn write_container(&self, cnt: Container) -> BuilderResult<()> {
        debug!("write container: {}", cnt.id);

        let mut containers = self.containers()?;
        containers.push(cnt);

        let cnt_file_path = self.containers_path();
        let cnt_file = match File::create(&cnt_file_path) {
            Ok(f) => f,
            Err(err) => {
                return Err(BuilderError::ContainerStoreError(format!(
                    "{:?}: {:?}",
                    cnt_file_path,
                    err.to_string(),
                )));
            }
        };

        match serde_json::to_writer(cnt_file, &containers) {
            Ok(_) => {}
            Err(err) => {
                return Err(BuilderError::ContainerStoreError(format!(
                    "{:?}: {:?}",
                    cnt_file_path,
                    err.to_string(),
                )));
            }
        }

        Ok(())
    }

    pub fn container_digest(&self, name_or_id: &str) -> BuilderResult<digest::Digest> {
        let cnt_list = self.containers()?;

        for cnt in cnt_list {
            let input_id = name_or_id.to_string();

            if cnt.name == input_id || (input_id.len() >= 12 && cnt.id[..12] == input_id[..12]) {
                return digest::Digest::new(&format!("sha256:{}", cnt.id));
            }
        }

        Err(BuilderError::ContainerNotFound(name_or_id.to_string()))
    }

    pub fn container_exist(&self, name_or_id: &str) -> BuilderResult<Container> {
        let cnt_id = self.container_digest(name_or_id)?;
        let cnt_list = self.containers()?;

        for cnt in cnt_list {
            if cnt_id.encoded == cnt.id {
                return Ok(cnt);
            }
        }

        Err(BuilderError::ContainerNotFound(name_or_id.to_string()))
    }

    pub fn container_by_digest(&self, dg: &digest::Digest) -> BuilderResult<Container> {
        let cnt_list = self.containers()?;
        for cnt in cnt_list {
            if cnt.id == dg.encoded {
                return Ok(cnt);
            }
        }

        Err(BuilderError::ContainerNotFound(dg.to_string()))
    }

    pub fn containers_by_image(&self, image_id: &digest::Digest) -> BuilderResult<Vec<Container>> {
        let mut containers: Vec<Container> = Vec::new();

        let cnt_list = self.containers()?;
        for cnt in cnt_list {
            if cnt.image_id() == image_id.encoded {
                containers.push(cnt);
            }
        }

        Ok(containers)
    }

    pub fn containers_path(&self) -> PathBuf {
        let mut containers_file = self.cstore_path().clone();
        containers_file.push(CONTAINERS_FILENAME);

        containers_file
    }
}
