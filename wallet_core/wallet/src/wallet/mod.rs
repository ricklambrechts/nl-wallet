mod config;
mod disclosure;
mod documents;
mod history;
mod init;
mod issuance;
mod lock;
mod registration;
mod uri;

#[cfg(test)]
mod tests;

use openid4vc::issuance_client::HttpIssuerClient;
use tokio::sync::RwLock;
use uuid::Uuid;

use nl_wallet_mdoc::holder::{CborHttpClient, DisclosureSession};
use platform_support::hw_keystore::hardware::{HardwareEcdsaKey, HardwareEncryptionKey};

use crate::{
    account_provider::HttpAccountProviderClient,
    config::UpdatingFileHttpConfigurationRepository,
    digid::HttpDigidSession,
    lock::WalletLock,
    storage::{DatabaseStorage, RegistrationData},
};

pub use self::{
    disclosure::{DisclosureError, DisclosureProposal},
    history::{EventStatus, HistoryError, HistoryEvent},
    init::WalletInitError,
    issuance::PidIssuanceError,
    lock::WalletUnlockError,
    registration::WalletRegistrationError,
    uri::{UriIdentificationError, UriType},
};

use self::documents::DocumentsCallback;

pub struct Wallet<
    CR = UpdatingFileHttpConfigurationRepository,  // ConfigurationRepository
    S = DatabaseStorage<HardwareEncryptionKey>,    // Storage
    PEK = HardwareEcdsaKey,                        // PlatformEcdsaKey
    APC = HttpAccountProviderClient,               // AccountProviderClient
    DGS = HttpDigidSession,                        // DigidSession
    IC = HttpIssuerClient,                         // IssuerClient
    MDS = DisclosureSession<CborHttpClient, Uuid>, // MdocDisclosureSession
> {
    config_repository: CR,
    storage: RwLock<S>,
    hw_privkey: PEK,
    account_provider_client: APC,
    digid_session: Option<DGS>,
    pid_issuer: IC,
    disclosure_session: Option<MDS>,
    lock: WalletLock,
    registration: Option<RegistrationData>,
    documents_callback: Option<DocumentsCallback>,
}
