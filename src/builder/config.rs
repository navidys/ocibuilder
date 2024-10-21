use log::debug;
use oci_client::config::{ConfigFile, History};

use crate::{commands::config::Config, error::BuilderResult};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn update_config(&self, cfg: &Config) -> BuilderResult<()> {
        self.lock()?;

        let cnt_id = &self.container_store().container_digest(&cfg.container_id)?;

        debug!("update container {} config", cnt_id);

        let mut img_cfg = self.container_store().get_config(cnt_id)?;

        if cfg.author.is_some() {
            self.set_author(&mut img_cfg, &cfg.author)?;
        }

        if cfg.user.is_some() {
            self.set_user(&mut img_cfg, &cfg.user)?;
        }

        if cfg.working_dir.is_some() {
            self.set_working_dir(&mut img_cfg, &cfg.working_dir)?;
        }

        if cfg.stop_signal.is_some() {
            self.set_stop_signal(&mut img_cfg, &cfg.stop_signal)?;
        }

        if cfg.created_by.is_some() {
            self.set_created_by(&mut img_cfg, &cfg.created_by)?;
        }

        self.container_store().write_config(cnt_id, &img_cfg)?;
        self.unlock()?;
        Ok(())
    }

    fn set_author(&self, cfg: &mut ConfigFile, author: &Option<String>) -> BuilderResult<()> {
        debug!("set container config author: {:?}", author);

        cfg.author = author.to_owned();

        let mut img_history = cfg.history.to_owned().unwrap_or_default();
        let change_history = History {
            created: Some(chrono::Utc::now()),
            author: author.to_owned(),
            created_by: Some(format!(
                "/bin/sh -c #(nop) MAINTAINER {}",
                author.to_owned().unwrap_or_default()
            )),
            comment: None,
            empty_layer: Some(true),
        };

        img_history.insert(0, change_history);
        cfg.history = Some(img_history);

        Ok(())
    }

    fn set_user(&self, cfg: &mut ConfigFile, user: &Option<String>) -> BuilderResult<()> {
        debug!("set container config user: {:?}", user);

        let mut cfg_config = cfg.config.to_owned().unwrap_or_default();
        cfg_config.user = user.to_owned();

        cfg.config = Some(cfg_config);

        let mut img_history = cfg.history.to_owned().unwrap_or_default();
        let change_history = History {
            created: Some(chrono::Utc::now()),
            author: None,
            created_by: Some(format!(
                "/bin/sh -c #(nop) USER {}",
                user.to_owned().unwrap_or_default()
            )),
            comment: None,
            empty_layer: Some(true),
        };

        img_history.insert(0, change_history);
        cfg.history = Some(img_history);

        Ok(())
    }

    fn set_working_dir(
        &self,
        cfg: &mut ConfigFile,
        working_dir: &Option<String>,
    ) -> BuilderResult<()> {
        debug!("set container config working dir: {:?}", working_dir);

        let mut cfg_config = cfg.to_owned().config.unwrap_or_default();
        cfg_config.working_dir = working_dir.to_owned();

        cfg.config = Some(cfg_config);

        let mut img_history = cfg.history.to_owned().unwrap_or_default();
        let change_history = History {
            created: Some(chrono::Utc::now()),
            author: None,
            created_by: Some(format!(
                "/bin/sh -c #(nop) WORKDIR {}",
                working_dir.to_owned().unwrap_or_default()
            )),
            comment: None,
            empty_layer: Some(true),
        };

        img_history.insert(0, change_history);
        cfg.history = Some(img_history);

        Ok(())
    }

    pub fn set_stop_signal(
        &self,
        cfg: &mut ConfigFile,
        stop_signal: &Option<String>,
    ) -> BuilderResult<()> {
        debug!("set container config stop signal: {:?}", stop_signal);

        let mut cfg_config = cfg.to_owned().config.unwrap_or_default();
        cfg_config.stop_signal = stop_signal.to_owned();

        cfg.config = Some(cfg_config);

        let mut img_history = cfg.history.to_owned().unwrap_or_default();
        let change_history = History {
            created: Some(chrono::Utc::now()),
            author: None,
            created_by: Some(format!(
                "/bin/sh -c #(nop) STOPSIGNAL {}",
                stop_signal.to_owned().unwrap_or_default()
            )),
            comment: None,
            empty_layer: Some(true),
        };

        img_history.insert(0, change_history);
        cfg.history = Some(img_history);

        Ok(())
    }

    pub fn set_created_by(
        &self,
        cfg: &mut ConfigFile,
        created_by: &Option<String>,
    ) -> BuilderResult<()> {
        debug!("set container config created by: {:?}", created_by);

        let mut img_history = cfg.history.to_owned().unwrap_or_default();
        let change_history = History {
            created: Some(chrono::Utc::now()),
            author: None,
            created_by: created_by.to_owned(),
            comment: None,
            empty_layer: Some(true),
        };

        img_history.insert(0, change_history);
        cfg.history = Some(img_history);

        Ok(())
    }
}
