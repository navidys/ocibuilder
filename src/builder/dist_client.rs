use docker_credential::{CredentialRetrievalError, DockerCredential};
use log::{debug, warn};
use oci_client::{client, secrets::RegistryAuth, Reference};

use crate::error::BuilderResult;

pub fn build_auth(reference: &Reference, anon: bool) -> BuilderResult<RegistryAuth> {
    let server = reference
        .resolve_registry()
        .strip_suffix('/')
        .unwrap_or_else(|| reference.resolve_registry());

    if anon {
        return Ok(RegistryAuth::Anonymous);
    }

    match docker_credential::get_credential(server) {
        Err(CredentialRetrievalError::ConfigNotFound) => Ok(RegistryAuth::Anonymous),
        Err(CredentialRetrievalError::NoCredentialConfigured) => Ok(RegistryAuth::Anonymous),
        Err(e) => panic!("Error handling docker configuration file: {}", e),
        Ok(DockerCredential::UsernamePassword(username, password)) => {
            debug!("Found docker credentials");
            Ok(RegistryAuth::Basic(username, password))
        }
        Ok(DockerCredential::IdentityToken(_)) => {
            warn!("Cannot use contents of docker config, identity token not supported. Using anonymous auth");
            Ok(RegistryAuth::Anonymous)
        }
    }
}

pub fn build_client_config(insecure: bool) -> BuilderResult<client::ClientConfig> {
    let protocol = if insecure {
        client::ClientProtocol::Http
    } else {
        client::ClientProtocol::Https
    };

    Ok(client::ClientConfig {
        protocol,
        ..Default::default()
    })
}
