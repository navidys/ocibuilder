use std::path::{Path, PathBuf};

use libcontainer::{
    container::{builder::ContainerBuilder, Container},
    error::LibcontainerError,
    syscall::syscall::SyscallType,
};

use liboci_cli::Run;
use log::debug;

use crate::error::{BuilderError, BuilderResult};

use anyhow::Result;

use super::run_yuki_executer;

pub fn run_container(
    bundle_dir: &Path,
    root_path: &Path,
    cnt_id: String,
    systemd_cgroup: &bool,
) -> BuilderResult<()> {
    let args = liboci_cli::Run {
        bundle: bundle_dir.to_path_buf(),
        console_socket: None,
        pid_file: None,
        no_pivot: false,
        no_new_keyring: true,
        preserve_fds: 0,
        container_id: cnt_id.clone(),
        no_subreaper: true,
        detach: false,
        keep: false,
    };

    debug!("run yuki create container");

    match create_container(args, root_path, systemd_cgroup) {
        Ok(mut c) => start_container(&mut c),
        Err(err) => {
            if err
                .to_string()
                .contains(&LibcontainerError::Exist.to_string())
            {
                let mut cnt_root_path = root_path.to_path_buf();
                cnt_root_path.push(PathBuf::from(cnt_id));

                debug!("yuki trying to load container");
                let mut container = match Container::load(cnt_root_path) {
                    Ok(c) => c,
                    Err(err) => {
                        debug!("yuki load container failed: {:?}", err);

                        return Err(BuilderError::AnyError(err.to_string()));
                    }
                };

                if container.can_start() {
                    start_container(&mut container)?;
                }
            }

            debug!("yuki create container failed: {:?}", err);
            Err(BuilderError::AnyError(err.to_string()))
        }
    }
}

fn start_container(container: &mut Container) -> BuilderResult<()> {
    debug!("run yuki start container");
    match container.start() {
        Ok(_) => {}
        Err(err) => {
            return Err(BuilderError::YukiError(err.to_string()));
        }
    }

    //debug!("run yuki delete container");
    //match container.delete(true) {
    //   Ok(_) => {}
    //    Err(err) => return Err(BuilderError::YukiError(err.to_string())),
    //}

    Ok(())
}

fn create_container(args: Run, root_path: &Path, systemd_cgroup: &bool) -> Result<Container> {
    let container = ContainerBuilder::new(args.container_id.clone(), SyscallType::default())
        .with_executor(run_yuki_executer::default_executor())
        .with_pid_file(args.pid_file.as_ref())?
        .with_console_socket(args.console_socket.as_ref())
        .with_root_path(root_path)?
        .with_preserved_fds(args.preserve_fds)
        .validate_id()?
        .as_init(&args.bundle)
        .with_systemd(systemd_cgroup.to_owned())
        .with_detach(args.detach)
        .build()?;

    Ok(container)
}
