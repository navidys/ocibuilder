use oci_client::config::ConfigFile;

use crate::error::{BuilderError, BuilderResult};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn inspect(&self, name_or_id: &str) -> BuilderResult<ConfigFile> {
        let mut errors = Vec::new();
        // first try image inspect
        match self.image_store().image_digest(name_or_id) {
            Ok(image_dg) => {
                let img_config = self.image_store().get_config(&image_dg)?;
                return Ok(img_config);
            }
            Err(err) => errors.push(err),
        }

        // try container
        match self.container_store().container_digest(name_or_id) {
            Ok(cnt_dg) => {
                let cnt_config = self.container_store().get_builder_config(&cnt_dg)?;
                return Ok(cnt_config);
            }
            Err(err) => errors.push(err),
        }

        Err(BuilderError::ContainerOrImageNotFound(
            name_or_id.to_string(),
        ))
    }
}
