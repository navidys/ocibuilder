use log::debug;

use crate::{error::BuilderResult, utils};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub async fn from(&self, img_name: &str, name: Option<String>) -> BuilderResult<String> {
        self.lock()?;
        let mut cnt_name = name.unwrap_or_default();

        if img_name != "scratch" {
            let img_digest = self.pull(img_name, &false).await?;
            debug!("container from image: {}", img_digest);

            let img_info = self.image_store().image_reference(&img_digest)?;
            let img_info_name = format!(
                "{}/{}:{}",
                img_info.registry(),
                img_info.repository(),
                img_info.tag().unwrap_or_default(),
            );

            let img_manifest = self.image_store().get_manifest(&img_digest)?;

            let mut img_layers: Vec<String> = Vec::new();
            for layer in img_manifest.layers {
                img_layers.push(layer.digest);
            }

            let layer_digest = utils::new_digest_id()?;
            debug!("container top layer: {}", layer_digest);

            self.layer_store().create_layer_overlay_dir(&layer_digest)?;

            if cnt_name.is_empty() {
                let img_ref = self.image_store().image_reference(&img_digest)?;
                let sp_image_name = img_ref.repository().split('/').last().unwrap_or_default();

                cnt_name = format!("{}-working-container", sp_image_name);

                let containers_list = self.container_store().containers()?;
                let mut index = 0;
                for cnt in containers_list {
                    if cnt.name() == cnt_name {
                        index += 1;
                        cnt_name = format!("{}-working-container-{}", sp_image_name, index);
                    }
                }
            }

            let cnt_id = self.container_store().create(
                &cnt_name,
                &img_info_name,
                &img_digest.encoded,
                &layer_digest.encoded,
                &img_layers,
            )?;

            let img_config = self.image_store().get_config(&img_digest)?;

            self.container_store().write_config(&cnt_id, &img_config)?;
        } else {
            let layer_digest = utils::new_digest_id()?;
            debug!("container top layer: {}", layer_digest);
            self.layer_store().create_layer_overlay_dir(&layer_digest)?;
            if cnt_name.is_empty() {
                cnt_name = "working-container".to_string();

                let containers_list = self.container_store().containers()?;
                let mut index = 0;
                for cnt in containers_list {
                    if cnt.name() == cnt_name {
                        index += 1;
                        cnt_name = format!("working-container-{}", index);
                    }
                }
            }

            self.container_store().create(
                &cnt_name,
                "scratch",
                "",
                &layer_digest.encoded,
                &Vec::new(),
            )?;
        }

        self.unlock()?;

        Ok(cnt_name)
    }
}
