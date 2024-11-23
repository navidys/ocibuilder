use log::debug;

use crate::{
    error::{BuilderError, BuilderResult},
    utils,
};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn rmi(&self, images: &[String], force: &bool) -> BuilderResult<()> {
        self.lock()?;

        for img in images {
            let img_id = self.image_store().image_digest(img)?;
            let cnt_list: Vec<crate::container::containers::Container> =
                self.container_store().containers_by_image(&img_id)?;
            debug!("container used by image: {:?}", cnt_list);

            if !cnt_list.is_empty() {
                if !force {
                    return Err(BuilderError::ImageUsedByContainer(cnt_list[0].id()));
                }

                for cnt in cnt_list {
                    let cnt_id = utils::digest::Digest::new(&format!("sha256:{}", cnt.id()))?;
                    let cnt_toplayer_id =
                        utils::digest::Digest::new(&format!("sha256:{}", &cnt.top_layer()))?;

                    self.container_store().remove(&cnt_id)?;
                    self.layer_store().remove_layer_overlay(&cnt_toplayer_id)?;
                }
            }

            let img_manifest = self.image_store().get_manifest(&img_id)?;

            println!("Removing image {:.12}", img_id.encoded);
            // remove overylay-images
            self.image_store().remove(&img_id)?;

            for layer in img_manifest.layers {
                let layer_id = utils::digest::Digest::new(&layer.digest)?;
                // remove overlay
                self.layer_store().remove_layer_overlay(&layer_id)?;

                // remove overlay-layers blobs
                self.layer_store().remove_blob(&layer_id)?;
            }
        }

        self.unlock()?;
        Ok(())
    }
}
