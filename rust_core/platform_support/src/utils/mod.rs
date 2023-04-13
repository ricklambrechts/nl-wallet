#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "software")]
pub mod software;

use std::path::PathBuf;
use thiserror::Error;

// implementation of UtilitiesError from UDL, only with "hardware" flag
#[derive(Debug, Error)]
pub enum UtilitiesError {
    #[error("Platform error: {reason}")]
    PlatformError { reason: String },
    #[error("Bridging error: {reason}")]
    BridgingError { reason: String },
}

pub trait PlatformUtilities {
    fn storage_path() -> Result<PathBuf, UtilitiesError>;
}

// if the hardware feature is enabled, prefer HardwareUtilities
#[cfg(feature = "hardware")]
pub type PreferredPlatformUtilities = self::hardware::HardwareUtilities;

// otherwise if the software feature is enabled, prefer SoftwareUtilities
#[cfg(all(not(feature = "hardware"), feature = "software"))]
pub type PreferredPlatformUtilities = self::software::SoftwareUtilities;

// otherwise just just alias the Never type
#[cfg(not(any(feature = "hardware", feature = "software")))]
pub type PreferredPlatformUtilities = never::Never;
