use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use crate::error::{BuilderError, BuilderResult};

use super::reference::Reference;

lazy_static::lazy_static! {
    static ref NAME_RE: Regex = Regex::new(r"^[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*(\/[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*)*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageName {
    pub hostname: String,
    pub port: Option<u16>,
    pub name: Name,
    pub reference: Reference,
}

impl std::ops::Deref for Name {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Name {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn new(name: &str) -> BuilderResult<Self> {
        if NAME_RE.is_match(name) {
            Ok(Name(name.to_string()))
        } else {
            Err(BuilderError::InvalidImageName(name.to_string()))
        }
    }
}

impl fmt::Display for ImageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(port) = self.port {
            write!(
                f,
                "{}:{}/{}:{}",
                self.hostname, port, self.name, self.reference
            )
        } else {
            write!(f, "{}/{}:{}", self.hostname, self.name, self.reference)
        }
    }
}

impl FromStr for ImageName {
    type Err = BuilderError;
    fn from_str(name: &str) -> BuilderResult<Self> {
        let (hostname, name) = name
            .split_once('/')
            .unwrap_or(("registry-1.docker.io", name));
        let (hostname, port) = if let Some((hostname, port)) = hostname.split_once(':') {
            let p = str::parse(port);
            if p.is_err() {
                (hostname, None)
            } else {
                (hostname, Some(p.unwrap_or_default()))
            }
        } else {
            (hostname, None)
        };
        let (name, reference) = name.split_once(':').unwrap_or((name, "latest"));
        Ok(ImageName {
            hostname: hostname.to_string(),
            port,
            name: Name::new(name)?,
            reference: Reference::new(reference)?,
        })
    }
}

impl Serialize for ImageName {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ImageName {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ImageName::parse(&s).map_err(serde::de::Error::custom)
    }
}

impl ImageName {
    pub fn parse(name: &str) -> BuilderResult<Self> {
        Self::from_str(name)
    }
}
