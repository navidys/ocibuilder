use std::{
    ffi::OsString,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use libcontainer::oci_spec::runtime::Spec;
use log::debug;
use oci_client::config::History;

use crate::{
    builder::run_yuki,
    container::run_spec,
    error::{BuilderError, BuilderResult},
    utils,
};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn run(
        &self,
        container: &str,
        cmd: &Vec<String>,
        rundir: &Option<OsString>,
        systemd_cgroup: &bool,
    ) -> BuilderResult<()> {
        self.lock()?;

        let cnt = self.container_store().container_exist(container)?;
        let cnt_id = utils::digest::Digest::new(&format!("sha256:{}", cnt.id()))?;
        let cnt_top_layer_digest =
            utils::digest::Digest::new(&format!("sha256:{}", cnt.top_layer()))?;

        // update config process args
        let runtime_spec = self.container_store().runtime_spec(&cnt_id)?;
        let runtime_path = self.layer_store().overlay_dir_path(&cnt_top_layer_digest);

        update_and_write_runtime_spec(&runtime_path, runtime_spec, cmd)?;
        self.mount(container)?;

        let yuki_run_dir = match utils::get_run_dir(rundir) {
            Ok(rund) => rund,
            Err(err) => {
                self.umount(container)?;
                return Err(err);
            }
        };

        debug!("yuki runtime dir: {:?}", yuki_run_dir.clone());
        debug!("yuki runtime systemd: {:?}", systemd_cgroup);
        debug!("yuki running cmd: {:?}", cmd);

        match run_yuki::run_container(&runtime_path, &yuki_run_dir, &cnt_id.encoded, systemd_cgroup)
        {
            Ok(_) => {}
            Err(err) => {
                debug!("yuki run error: {:?}", err);

                match fs::remove_dir_all(&yuki_run_dir) {
                    Ok(_) => {}
                    Err(err) => {
                        self.umount(container)?;
                        return Err(BuilderError::IoError(yuki_run_dir, err));
                    }
                }

                self.umount(container)?;
                return Err(err);
            }
        }

        match fs::remove_dir_all(&yuki_run_dir) {
            Ok(_) => {}
            Err(err) => {
                self.umount(container)?;
                return Err(BuilderError::IoError(yuki_run_dir, err));
            }
        }

        self.umount(container)?;

        // update history
        let diff_path = self.layer_store().overlay_diff_path(&cnt_top_layer_digest);
        let is_empty_layer = utils::is_empty_dir(&diff_path)?;

        let mut img_cfg = self.container_store().get_builder_config(&cnt_id)?;

        let mut img_history = img_cfg.history.to_owned().unwrap_or_default();
        let mut change_history = History {
            created: Some(chrono::Utc::now()),
            author: None,
            created_by: Some(cmd.join(" ")),
            comment: None,
            empty_layer: None,
        };

        if is_empty_layer {
            change_history.empty_layer = Some(is_empty_layer)
        }

        img_history.insert(0, change_history);
        img_cfg.history = Some(img_history);

        self.container_store()
            .write_builder_config(&cnt_id, &img_cfg)?;

        if !is_empty_layer {
            self.commit(container, None)?;
        }

        self.unlock()?;
        Ok(())
    }
}

fn update_and_write_runtime_spec(
    runtime_dir: &Path,
    mut runtime_spec: Spec,
    cmd: &Vec<String>,
) -> BuilderResult<()> {
    let mut process = runtime_spec.process().to_owned().unwrap_or_default();
    let mut args: Vec<String> = Vec::new();

    for c in cmd {
        args.push(c.to_string());
    }

    process.set_args(Some(args));
    runtime_spec.set_process(Some(process));

    // enable read-write of root fs
    let mut runtime_root = runtime_spec.root().to_owned().unwrap_or_default();
    runtime_root.set_readonly(Some(false));

    runtime_spec.set_root(Some(runtime_root));

    let mut runtime_spec_path = runtime_dir.to_path_buf();
    runtime_spec_path.push(run_spec::SPEC_FILENAME);

    let spec_file = match File::create(&runtime_spec_path) {
        Ok(f) => f,
        Err(err) => return Err(BuilderError::IoError(runtime_spec_path, err)),
    };

    let mut writer = BufWriter::new(spec_file);
    match serde_json::to_writer_pretty(&mut writer, &runtime_spec) {
        Ok(_) => {}
        Err(err) => return Err(BuilderError::SerdeJsonError(err)),
    }

    match writer.flush() {
        Ok(_) => {}
        Err(err) => return Err(BuilderError::IoError(runtime_spec_path, err)),
    }

    Ok(())
}
