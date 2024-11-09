use crate::{error::BuilderResult, utils};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn rm(&self, containers: &Vec<String>) -> BuilderResult<()> {
        self.lock()?;

        for cnt in containers {
            let cnt = self.container_store().container_exist(cnt)?;
            let cnt_id = utils::digest::Digest::new(&format!("sha256:{}", &cnt.id()))?;
            let cnt_toplayer_id =
                utils::digest::Digest::new(&format!("sha256:{}", &cnt.top_layer()))?;

            println!("Removing container {:.12}", cnt_id.encoded);
            self.container_store().remove(&cnt_id)?;
            self.layer_store().remove_layer_overlay(&cnt_toplayer_id)?;
        }

        self.unlock()?;
        Ok(())
    }
}
