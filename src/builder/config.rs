use std::collections::HashMap;

use log::debug;
use oci_client::config::{ConfigFile, History};

use crate::{commands::config::Config, error::BuilderResult};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn update_config(&self, cfg: &Config) -> BuilderResult<()> {
        self.lock()?;

        let cnt_id = &self.container_store().container_digest(&cfg.container_id)?;

        debug!("update container {} config", cnt_id);

        let mut img_cfg = self.container_store().get_builder_config(cnt_id)?;

        if cfg.author.is_some() {
            self.set_author(&mut img_cfg, &cfg.author, cfg.add_history)?;
        }

        if cfg.user.is_some() {
            self.set_user(&mut img_cfg, &cfg.user, cfg.add_history)?;
        }

        if cfg.working_dir.is_some() {
            self.set_working_dir(&mut img_cfg, &cfg.working_dir, cfg.add_history)?;
        }

        if cfg.stop_signal.is_some() {
            self.set_stop_signal(&mut img_cfg, &cfg.stop_signal, cfg.add_history)?;
        }

        if cfg.created_by.is_some() {
            // created_by will be part of history so not passing add_history
            self.set_created_by(&mut img_cfg, &cfg.created_by)?;
        }

        if cfg.cmd.is_some() {
            self.set_cmd(&mut img_cfg, &cfg.cmd, cfg.add_history)?;
        }

        if cfg.entrypoint.is_some() {
            self.set_entrypoint(&mut img_cfg, &cfg.entrypoint, cfg.add_history)?;
        }

        if cfg.env.is_some() {
            self.set_env(&mut img_cfg, &cfg.env, cfg.add_history)?;
        }

        if cfg.label.is_some() {
            self.set_label(&mut img_cfg, &cfg.label, cfg.add_history)?;
        }

        if cfg.port.is_some() {
            self.set_port(&mut img_cfg, &cfg.port, cfg.add_history)?;
        }

        self.container_store()
            .write_builder_config(cnt_id, &img_cfg)?;
        self.unlock()?;
        Ok(())
    }

    fn set_port(
        &self,
        cfg: &mut ConfigFile,
        port: &Option<String>,
        add_history: bool,
    ) -> BuilderResult<()> {
        debug!("set container config port: {:?}", port);

        let mut cfg_config = cfg.config.to_owned().unwrap_or_default();
        let mut history_exposed_port: Vec<String> = Vec::new();
        let mut image_exposed_port = cfg_config.exposed_ports.unwrap_or_default();

        for port_item in port.to_owned().unwrap_or_default().split(',') {
            let input_port = match port_item.find('/') {
                Some(_) => port_item.to_owned(),
                None => format!("{}/tcp", port_item),
            };

            if !image_exposed_port.contains(&input_port) {
                history_exposed_port.push(input_port.to_owned());
                image_exposed_port.insert(input_port);
            }
        }

        if image_exposed_port.is_empty() {
            cfg_config.exposed_ports = None;
        } else {
            cfg_config.exposed_ports = Some(image_exposed_port);

            if !history_exposed_port.is_empty() && add_history {
                let mut img_history = cfg.history.to_owned().unwrap_or_default();
                let change_history = History {
                    created: Some(chrono::Utc::now()),
                    author: None,
                    created_by: Some(format!(
                        "/bin/sh -c #(nop) EXPOSE {}",
                        history_exposed_port.join(" ")
                    )),
                    comment: None,
                    empty_layer: Some(true),
                };

                img_history.insert(0, change_history);
                cfg.history = Some(img_history);
            }
        }

        cfg.config = Some(cfg_config);

        Ok(())
    }

    fn set_label(
        &self,
        cfg: &mut ConfigFile,
        label: &Option<String>,
        add_history: bool,
    ) -> BuilderResult<()> {
        debug!("set container config label: {:?}", label);

        let mut cfg_config = cfg.config.to_owned().unwrap_or_default();
        let mut history_label_list: Vec<String> = Vec::new();
        let mut image_labels = cfg_config.labels.unwrap_or_default();

        for label_item in label.to_owned().unwrap_or_default().split(',') {
            let label_key_value = label_item.split('=').collect::<Vec<&str>>();

            if label_key_value.len() != 2 {
                continue;
            }

            let key = label_key_value[0].to_string();
            let value = label_key_value[1].to_string();

            if image_labels.contains_key(&key) {
                image_labels.remove(&key);
            }

            history_label_list.push(format!("{}={}", key, value));
            image_labels.insert(key, value);
        }

        if image_labels.is_empty() {
            cfg_config.labels = None;
        } else {
            cfg_config.labels = Some(image_labels);

            if !history_label_list.is_empty() && add_history {
                let mut img_history = cfg.history.to_owned().unwrap_or_default();
                let change_history = History {
                    created: Some(chrono::Utc::now()),
                    author: None,
                    created_by: Some(format!(
                        "/bin/sh -c #(nop) LABEL {}",
                        history_label_list.join(" ")
                    )),
                    comment: None,
                    empty_layer: Some(true),
                };

                img_history.insert(0, change_history);
                cfg.history = Some(img_history);
            }
        }

        cfg.config = Some(cfg_config);

        Ok(())
    }

    fn set_env(
        &self,
        cfg: &mut ConfigFile,
        env: &Option<String>,
        add_history: bool,
    ) -> BuilderResult<()> {
        debug!("set container config env: {:?}", env);

        let mut cfg_config = cfg.config.to_owned().unwrap_or_default();
        let mut history_env_list: Vec<String> = Vec::new();
        let mut env_hashmap = create_env_hash(&cfg_config.env);

        for env_item in env.to_owned().unwrap_or_default().split(',') {
            let env_key_value = env_item.split('=').collect::<Vec<&str>>();
            if env_key_value.len() != 2 {
                continue;
            }

            let key = env_key_value[0].to_string();
            let value = env_key_value[1].to_string();

            if env_hashmap.contains_key(&key) {
                env_hashmap.remove(&key);
            }

            history_env_list.push(format!("{}={}", key, value));
            env_hashmap.insert(key, value);
        }

        let mut env_list: Vec<String> = Vec::new();
        for env_item in env_hashmap.into_iter() {
            env_list.push(format!("{}={}", env_item.0, env_item.1));
        }

        if env_list.is_empty() {
            cfg_config.env = None
        } else {
            cfg_config.env = Some(env_list);

            if !history_env_list.is_empty() && add_history {
                let mut img_history = cfg.history.to_owned().unwrap_or_default();
                let change_history = History {
                    created: Some(chrono::Utc::now()),
                    author: None,
                    created_by: Some(format!(
                        "/bin/sh -c #(nop) ENV {}",
                        history_env_list.join(" ")
                    )),
                    comment: None,
                    empty_layer: Some(true),
                };

                img_history.insert(0, change_history);
                cfg.history = Some(img_history);
            }
        }

        cfg.config = Some(cfg_config);

        Ok(())
    }

    fn set_entrypoint(
        &self,
        cfg: &mut ConfigFile,
        entrypoint: &Option<String>,
        add_history: bool,
    ) -> BuilderResult<()> {
        debug!("set container config entrypoint: {:?}", entrypoint);

        let mut entry_list: Vec<String> = vec!["/bin/sh".to_string(), "-c".to_string()];
        for entry_item in entrypoint.to_owned().unwrap_or_default().split_whitespace() {
            entry_list.push(entry_item.to_string())
        }

        if entry_list.len() > 2 {
            let mut cfg_config = cfg.config.to_owned().unwrap_or_default();
            cfg_config.entrypoint = Some(entry_list.to_owned());
            cfg.config = Some(cfg_config);

            if add_history {
                let mut img_history = cfg.history.to_owned().unwrap_or_default();
                let change_history = History {
                    created: Some(chrono::Utc::now()),
                    author: None,
                    created_by: Some(format!(
                        "/bin/sh -c #(nop) ENTRYPOINT {:?}",
                        entry_list.to_owned()
                    )),
                    comment: None,
                    empty_layer: Some(true),
                };

                img_history.insert(0, change_history);
                cfg.history = Some(img_history);
            }
        }

        Ok(())
    }

    fn set_cmd(
        &self,
        cfg: &mut ConfigFile,
        cmd: &Option<String>,
        add_history: bool,
    ) -> BuilderResult<()> {
        debug!("set container config cmd: {:?}", cmd);

        let mut cmd_list: Vec<String> = Vec::new();
        for cmd_item in cmd.to_owned().unwrap_or_default().split_whitespace() {
            cmd_list.push(cmd_item.to_string())
        }

        if !cmd_list.is_empty() {
            let mut cfg_config = cfg.config.to_owned().unwrap_or_default();
            cfg_config.cmd = Some(cmd_list.clone());
            cfg.config = Some(cfg_config);

            if add_history {
                let mut img_history = cfg.history.to_owned().unwrap_or_default();
                let change_history = History {
                    created: Some(chrono::Utc::now()),
                    author: None,
                    created_by: Some(format!("/bin/sh -c #(nop) CMD {:?}", cmd_list.to_owned())),
                    comment: None,
                    empty_layer: Some(true),
                };

                img_history.insert(0, change_history);
                cfg.history = Some(img_history);
            }
        }

        Ok(())
    }

    fn set_author(
        &self,
        cfg: &mut ConfigFile,
        author: &Option<String>,
        add_history: bool,
    ) -> BuilderResult<()> {
        debug!("set container config author: {:?}", author);

        cfg.author = author.to_owned();

        if add_history {
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
        }

        Ok(())
    }

    fn set_user(
        &self,
        cfg: &mut ConfigFile,
        user: &Option<String>,
        add_history: bool,
    ) -> BuilderResult<()> {
        debug!("set container config user: {:?}", user);

        let mut cfg_config = cfg.config.to_owned().unwrap_or_default();
        cfg_config.user = user.to_owned();

        cfg.config = Some(cfg_config);

        if add_history {
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
        }

        Ok(())
    }

    fn set_working_dir(
        &self,
        cfg: &mut ConfigFile,
        working_dir: &Option<String>,
        add_history: bool,
    ) -> BuilderResult<()> {
        debug!("set container config working dir: {:?}", working_dir);

        let mut cfg_config = cfg.to_owned().config.unwrap_or_default();
        cfg_config.working_dir = working_dir.to_owned();

        cfg.config = Some(cfg_config);

        if add_history {
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
        }

        Ok(())
    }

    pub fn set_stop_signal(
        &self,
        cfg: &mut ConfigFile,
        stop_signal: &Option<String>,
        add_history: bool,
    ) -> BuilderResult<()> {
        debug!("set container config stop signal: {:?}", stop_signal);

        let mut cfg_config = cfg.to_owned().config.unwrap_or_default();
        cfg_config.stop_signal = stop_signal.to_owned();

        cfg.config = Some(cfg_config);

        if add_history {
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
        }

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

fn create_env_hash(env_list: &Option<Vec<String>>) -> HashMap<String, String> {
    let mut env_hash: HashMap<String, String> = HashMap::new();

    if env_list.is_none() {
        return env_hash;
    }

    for env_item in env_list.to_owned().unwrap_or_default() {
        let env_key_value = env_item.split('=').collect::<Vec<&str>>();
        if env_key_value.len() != 2 {
            continue;
        }

        let key = env_key_value[0].to_string();
        let value = env_key_value[1].to_string();
        env_hash.insert(key, value);
    }

    env_hash
}
