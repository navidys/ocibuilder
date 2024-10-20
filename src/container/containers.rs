use std::{fs::File, path::PathBuf};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::error::{BuilderError, BuilderResult};

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

    pub fn write_containers(&self, cnt: Container) -> BuilderResult<()> {
        debug!("write containers: {}", cnt.id);

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

    pub fn containers_path(&self) -> PathBuf {
        let mut containers_file = self.cstore_path().clone();
        containers_file.push(CONTAINERS_FILENAME);

        containers_file
    }
}
