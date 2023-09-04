use std::{env, path::PathBuf};

use async_trait::async_trait;
use tokio::task;

use super::{PlatformUtilities, UtilitiesError};

pub struct SoftwareUtilities;

#[async_trait]
impl PlatformUtilities for SoftwareUtilities {
    async fn storage_path() -> Result<PathBuf, UtilitiesError> {
        // This should not panic and does not error,
        // so we don't need to use `spawn::blocking()`.
        let path = task::spawn_blocking(env::temp_dir)
            .await
            .expect("Could not join tokio task");

        Ok(path)
    }
}
