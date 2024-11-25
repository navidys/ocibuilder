use docker_credential::{CredentialRetrievalError, DockerCredential};
use log::{debug, warn};
use oci_client::{client, secrets::RegistryAuth, Reference};

use crate::error::BuilderResult;

pub fn build_auth(reference: &Reference, anon: &bool) -> BuilderResult<RegistryAuth> {
    if anon.to_owned() {
        return Ok(RegistryAuth::Anonymous);
    }

    match docker_credential::get_podman_credential(reference.registry()) {
        Err(CredentialRetrievalError::ConfigNotFound) => {
            debug!("credential config file not found, using anonymous");
            Ok(RegistryAuth::Anonymous)
        }
        Err(CredentialRetrievalError::NoCredentialConfigured) => {
            debug!("no credential found, using anonymous");
            Ok(RegistryAuth::Anonymous)
        }
        Err(e) => {
            warn!("credential retrieval: {}", e.to_string());
            Ok(RegistryAuth::Anonymous)
        }
        Ok(DockerCredential::UsernamePassword(username, password)) => {
            debug!("found login username/password credentials");
            Ok(RegistryAuth::Basic(username, password))
        }
        Ok(DockerCredential::IdentityToken(_)) => {
            warn!("cannot use contents of docker config, identity token not supported. using anonymous auth");
            Ok(RegistryAuth::Anonymous)
        }
    }
}

pub fn build_client_config(insecure: &bool) -> BuilderResult<client::ClientConfig> {
    let protocol = if *insecure {
        client::ClientProtocol::Http
    } else {
        client::ClientProtocol::Https
    };

    Ok(client::ClientConfig {
        protocol,
        ..Default::default()
    })
}
