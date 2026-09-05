use anyhow::Result;
use iroh::SecretKey;
use std::sync::Arc;
use crate::ffi_error::FfiError;
use crate::network::NwProfile;
use crate::database::client::DbClient;


#[uniffi::export]
fn set_secret_key(db_client: Arc<DbClient>) -> Result<(), FfiError> {
    
    let secret_key = SecretKey::generate();
    db_client
        .set_profile(NwProfile {
                secret_key})?;
    
    Ok(())
}