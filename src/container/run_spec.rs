use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use libcontainer::oci_spec::runtime::{
    LinuxBuilder, LinuxIdMappingBuilder, LinuxNamespace, LinuxNamespaceBuilder, LinuxNamespaceType,
    Mount, Spec,
};

use anyhow::Result;
use log::debug;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

use super::store::ContainerStore;

pub const SPEC_FILENAME: &str = "config.json";

impl ContainerStore {
    pub fn runtime_spec(&self, cnt_id: &digest::Digest) -> BuilderResult<Spec> {
        let mut run_config_path = self.cstore_path().clone();
        run_config_path.push(&cnt_id.encoded);
        run_config_path.push(SPEC_FILENAME);

        let spec_file = match File::open(&run_config_path) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::IoError(run_config_path, err)),
        };

        let runtime_spec: Spec = match serde_json::from_reader(spec_file) {
            Ok(m) => m,
            Err(err) => return Err(BuilderError::SerdeJsonError(err)),
        };

        Ok(runtime_spec)
    }

    pub fn generate_runtime_spec(&self, cnt_id: &digest::Digest) -> BuilderResult<()> {
        let config_spec = if nix::unistd::geteuid().as_raw() == 0 {
            match get_default_config() {
                Ok(c) => c,
                Err(err) => return Err(BuilderError::AnyError(err.to_string())),
            }
        } else {
            match get_rootless_config() {
                Ok(c) => c,
                Err(err) => return Err(BuilderError::AnyError(err.to_string())),
            }
        };

        debug!("write  runtime config");

        let mut run_config_path = self.cstore_path().clone();
        run_config_path.push(&cnt_id.encoded);
        run_config_path.push(SPEC_FILENAME);

        let spec_file = match File::create(&run_config_path) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::IoError(run_config_path, err)),
        };

        let mut writer = BufWriter::new(spec_file);
        match serde_json::to_writer_pretty(&mut writer, &config_spec) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::SerdeJsonError(err)),
        }

        match writer.flush() {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(run_config_path, err)),
        }

        Ok(())
    }
}

fn get_default_config() -> Result<Spec> {
    let spec = Spec::default();

    Ok(spec)
}

fn get_rootless_config() -> Result<Spec> {
    let mut namespaces: Vec<LinuxNamespace> =
        libcontainer::oci_spec::runtime::get_default_namespaces()
            .into_iter()
            .filter(|ns| {
                ns.typ() != LinuxNamespaceType::Network && ns.typ() != LinuxNamespaceType::User
            })
            .collect();

    namespaces.push(
        LinuxNamespaceBuilder::default()
            .typ(LinuxNamespaceType::User)
            .build()?,
    );

    let uid = nix::unistd::geteuid().as_raw();
    let gid = nix::unistd::getegid().as_raw();

    let linux = LinuxBuilder::default()
        .namespaces(namespaces)
        .uid_mappings(vec![LinuxIdMappingBuilder::default()
            .host_id(uid)
            .container_id(0_u32)
            .size(1_u32)
            .build()?])
        .gid_mappings(vec![LinuxIdMappingBuilder::default()
            .host_id(gid)
            .container_id(0_u32)
            .size(1_u32)
            .build()?])
        .build()?;

    let mut mounts: Vec<Mount> = libcontainer::oci_spec::runtime::get_default_mounts();
    for mount in &mut mounts {
        if mount.destination().eq(Path::new("/sys")) {
            mount
                .set_source(Some(PathBuf::from("/sys")))
                .set_typ(Some(String::from("none")))
                .set_options(Some(vec![
                    "rbind".to_string(),
                    "nosuid".to_string(),
                    "noexec".to_string(),
                    "nodev".to_string(),
                    "ro".to_string(),
                ]));
        } else {
            let options: Vec<String> = mount
                .options()
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .filter(|&o| !o.starts_with("gid=") && !o.starts_with("uid="))
                .map(|o| o.to_string())
                .collect();
            mount.set_options(Some(options));
        }
    }

    let mut spec = get_default_config()?;
    spec.set_linux(Some(linux)).set_mounts(Some(mounts));
    Ok(spec)
}
