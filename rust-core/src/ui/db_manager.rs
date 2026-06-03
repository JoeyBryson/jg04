use std::{path::PathBuf, str::FromStr};
use crate::ui::Arc;
use super::{UiDbManager, UiDbError, UiDbRequest, UiDbClient};
use crate::db::DbManager;

impl UiDbManager {
    fn parse_path(path_str: &str) -> Result<PathBuf, UiDbError> {
        PathBuf::from_str(path_str).map_err(|e| UiDbError::InternalError {
            msg: format!("Invalid path: {}", e),
        })
    }
}

#[uniffi::export]
impl UiDbManager {
    
    #[uniffi::constructor]
    pub fn spawn(db_path_string: String) -> Result<Arc<Self>, UiDbError> {
        let db_path = Self::parse_path(&db_path_string)?;
        let inner = DbManager::<UiDbRequest>::spawn(db_path)?;
        
        Ok(Arc::new(Self { inner }))
    }

    #[uniffi::constructor]
    pub fn delete_db_then_spawn(db_path_string: String) -> Result<Arc<Self>, UiDbError> {
        let db_path = Self::parse_path(&db_path_string)?;
        let inner = DbManager::<UiDbRequest>::delete_db_then_spawn(db_path)?;
        
        Ok(Arc::new(Self { inner }))
    }

    pub fn get_client(&self) -> Arc<UiDbClient> {
        let worker_tx = self.inner.worker_tx();
        
        Arc::new(UiDbClient {
            worker_tx
        })
    }
}