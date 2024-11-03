use std::{
    ffi::CStr,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
};

use log::debug;

use crate::{
    error::{BuilderError, BuilderResult},
    utils,
};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn mount(&self, container: &str) -> BuilderResult<PathBuf> {
        self.lock()?;

        let cnt: crate::container::containers::Container =
            self.container_store().container_exist(container)?;

        let top_layer = cnt.top_layer();
        let top_layer_digest = utils::digest::Digest::new(&format!("sha256:{}", top_layer))?;
        let mount_point = self.layer_store().overlay_merged_path(&top_layer_digest);
        debug!("container {:.12} mount point: {:?}", cnt.id(), mount_point);

        let workdir_path = self.layer_store().overlay_work_path(&top_layer_digest);
        debug!(
            "container {:.12} work directory: {:?}",
            cnt.id(),
            workdir_path
        );

        let upperdir_path = self.layer_store().overlay_diff_path(&top_layer_digest);
        debug!(
            "container {:.12} upper directory: {:?}",
            cnt.id(),
            upperdir_path
        );

        let mut lowerdir_paths: Vec<String> = Vec::new();

        for layer in cnt.rootfs_diff() {
            let layer_digest = utils::digest::Digest::new(&layer)?;
            let layer_diff_path = self
                .layer_store()
                .overlay_diff_path(&layer_digest)
                .display()
                .to_string();
            lowerdir_paths.push(layer_diff_path);
        }

        if is_mounted(&mount_point)? {
            self.umount(container)?;
        }

        let mount_options = format!(
            "lowerdir={},upperdir={},workdir={}",
            lowerdir_paths.join(":"),
            upperdir_path.display(),
            workdir_path.display(),
        );

        debug!(
            "container {:.12} mount options: {:?}",
            cnt.id(),
            mount_options
        );

        if nix::unistd::geteuid().as_raw() != 0 {
            mount_rootless(&mount_point, lowerdir_paths, &upperdir_path, &workdir_path)?
        } else {
            mount(&mount_point, &mount_options)?;
        }

        self.unlock()?;
        Ok(mount_point)
    }

    pub fn umount(&self, container: &str) -> BuilderResult<()> {
        self.lock()?;

        let cnt = self.container_store().container_exist(container)?;
        let top_layer = cnt.top_layer();
        let top_layer_digest = utils::digest::Digest::new(&format!("sha256:{}", top_layer))?;
        let mount_point = self.layer_store().overlay_merged_path(&top_layer_digest);

        if is_mounted(&mount_point)? {
            debug!(
                "container {:.12} filesystem umount from {:?}",
                cnt.id(),
                mount_point
            );

            if nix::unistd::geteuid().as_raw() != 0 {
                umount_rootless(&mount_point)?;
            } else {
                umount(&mount_point)?;
            }
        }

        self.unlock()?;
        Ok(())
    }
}

fn is_mounted(source: &Path) -> BuilderResult<bool> {
    let proc_mounts_file = "/proc/mounts";
    let proc_mounts = match File::open(proc_mounts_file) {
        Ok(pm) => pm,
        Err(err) => return Err(BuilderError::IoError(PathBuf::from(proc_mounts_file), err)),
    };

    for line_result in BufReader::new(proc_mounts).lines() {
        match line_result {
            Ok(line) => {
                let mount_info: Vec<&str> = line.split_whitespace().collect();
                if !mount_info.is_empty()
                    && mount_info.len() == 6
                    && mount_info[1] == source.display().to_string()
                {
                    return Ok(true);
                }
            }
            Err(err) => return Err(BuilderError::IoError(PathBuf::from(proc_mounts_file), err)),
        }
    }

    Ok(false)
}

fn mount(mount_point: &Path, mount_options: &str) -> BuilderResult<()> {
    match nix::mount::mount(
        Some(CStr::from_bytes_with_nul(b"overlay\0").unwrap()),
        mount_point.display().to_string().as_bytes(),
        Some(CStr::from_bytes_with_nul(b"overlay\0").unwrap()),
        nix::mount::MsFlags::empty(),
        Some(mount_options.as_bytes()),
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(BuilderError::MountUmountError(err.to_string())),
    }
}

fn umount(mount_point: &Path) -> BuilderResult<()> {
    match nix::mount::umount(mount_point.display().to_string().as_bytes()) {
        Ok(_) => Ok(()),
        Err(err) => Err(BuilderError::MountUmountError(err.to_string())),
    }
}

fn mount_rootless(
    mount_point: &Path,
    lower_dir: Vec<String>,
    upper_dir: &Path,
    work_dir: &Path,
) -> BuilderResult<()> {
    let mut cmd_options = String::new();
    for ldir in lower_dir {
        if cmd_options.is_empty() {
            cmd_options = format!("lowerdir={}", ldir)
        } else {
            cmd_options = format!("{}:{}", cmd_options, ldir)
        }
    }

    cmd_options = format!(
        "{},upperdir={},workdir={}",
        cmd_options,
        upper_dir.display(),
        work_dir.display(),
    );

    match Command::new("/usr/bin/fuse-overlayfs")
        .arg("-o")
        .arg(cmd_options)
        .arg(mount_point.display().to_string())
        .output()
    {
        Ok(output) => {
            if !output.stderr.is_empty() {
                return Err(BuilderError::MountUmountError(
                    String::from_utf8(output.stderr).unwrap_or_default(),
                ));
            }

            Ok(())
        }
        Err(err) => Err(BuilderError::MountUmountError(err.to_string())),
    }
}

fn umount_rootless(mount_point: &Path) -> BuilderResult<()> {
    match Command::new("/usr/bin/fusermount")
        .arg("-u")
        .arg(mount_point.display().to_string())
        .output()
    {
        Ok(output) => {
            if !output.stderr.is_empty() {
                return Err(BuilderError::MountUmountError(
                    String::from_utf8(output.stderr).unwrap_or_default(),
                ));
            }

            Ok(())
        }
        Err(err) => Err(BuilderError::MountUmountError(err.to_string())),
    }
}
