use rustls::{ClientConfig, RootCertStore};

pub fn create_rustls_config() -> Result<ClientConfig, String> {
    let mut root_store = RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().map_err(|err| err.to_string())? {
        root_store.add(cert).map_err(|err| err.to_string())?;
    }
    Ok(ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth())
}
