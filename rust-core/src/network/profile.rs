use anyhow::Result;
use iroh::SecretKey;
use std::sync::Arc;
use crate::ffi_error::FfiError;
use crate::network::NwProfile;
use crate::database::DbClient;


#[uniffi::export]
fn set_secret_key(db_client: Arc<DbClient>) -> Result<(), FfiError> {
    
    let secret_key = SecretKey::generate();
    tokio::runtime::Runtime::new()
        .map_err(anyhow::Error::from)?
        .block_on(db_client
        .set_profile(NwProfile {
                secret_key
            }
        )
    )?;
    
    Ok(())
}