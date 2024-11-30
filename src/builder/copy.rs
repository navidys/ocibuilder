use std::path::{Path, PathBuf};

use log::debug;
use oci_client::config::History;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::{self, digest},
};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn copy(
        &self,
        cnt_name: &str,
        src: &str,
        dest: &str,
        add_history: &bool,
    ) -> BuilderResult<()> {
        self.lock()?;

        // check if container exist
        let cnt = self.container_store().container_exist(cnt_name)?;
        let cnt_id = &utils::digest::Digest::new(&format!("sha256:{}", cnt.id()))?;
        let cnt_toplayer = format!("sha256:{}", cnt.top_layer());
        let cnt_toplayer_digest = digest::Digest::new(&cnt_toplayer)?;
        let mut history_copy_type = "file";

        let cnt_toplayer_rootfs_path = self.layer_store().overlay_rootfs_path(&cnt_toplayer_digest);
        let cnt_rootfs_path =
            PathBuf::from(format!("{}/{}", cnt_toplayer_rootfs_path.display(), dest));

        debug!("cnt {} toplayer rootfs: {:?}", cnt_name, cnt_rootfs_path);

        let src_path = Path::new(src);
        // check if source exits and its type
        if !src_path.exists() {
            self.umount(cnt_name)?;

            return Err(BuilderError::CopyError(format!(
                "source path not found: {}",
                src
            )));
        }

        debug!("copy {} from  {} to {:?}", cnt_name, src, cnt_rootfs_path);

        self.mount(cnt_name)?;

        // based on the type copy to container rootfs
        let copy_id = match utils::copy::copy_content(src_path, &cnt_rootfs_path) {
            Ok(dg) => dg,
            Err(err) => {
                self.umount(cnt_name)?;
                return Err(err);
            }
        };

        let copy_id_digest = utils::digest::Digest::new(&copy_id)?;

        self.umount(cnt_name)?;

        if src_path.is_dir() {
            history_copy_type = "dir";
        }

        // add history
        if add_history.to_owned() {
            let mut img_cfg = self.container_store().get_builder_config(cnt_id)?;

            let mut img_history = img_cfg.history.to_owned().unwrap_or_default();
            let change_history = History {
                created: Some(chrono::Utc::now()),
                author: None,
                created_by: Some(format!(
                    "/bin/sh -c #(nop) COPY {}:{}",
                    history_copy_type, copy_id_digest.encoded,
                )),
                comment: None,
                empty_layer: Some(true),
            };

            img_history.insert(0, change_history);
            img_cfg.history = Some(img_history);

            self.container_store()
                .write_builder_config(cnt_id, &img_cfg)?;
        }

        self.unlock()?;

        Ok(())
    }
}
