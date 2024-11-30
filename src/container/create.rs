use std::fs;

use chrono::SecondsFormat;
use log::debug;

use crate::{
    container::containers::Container,
    error::{BuilderError, BuilderResult},
    utils::{self, digest},
};

use super::store::ContainerStore;

impl ContainerStore {
    pub fn create(
        &self,
        name: &str,
        image_name: &str,
        img_id: &str,
        layer_id: &str,
        layers: &Vec<String>,
    ) -> BuilderResult<digest::Digest> {
        debug!("container create");

        let new_cnt_id = utils::common::new_digest_id()?;
        let created: String = chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);

        let mut cnt_path = self.cstore_path().clone();
        cnt_path.push(&new_cnt_id.encoded);

        match fs::create_dir_all(&cnt_path) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(cnt_path, err)),
        }

        let cnt_config = Container::new(
            new_cnt_id.encoded.to_string(),
            image_name.to_string(),
            img_id.to_string(),
            layer_id.to_string(),
            created,
            layers.to_owned(),
            name.to_string(),
        )?;

        self.write_container(cnt_config)?;

        self.generate_runtime_spec(&new_cnt_id)?;

        Ok(new_cnt_id)
    }
}
