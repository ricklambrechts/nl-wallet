mod client;

#[cfg(any(test, feature = "mock"))]
mod mock;

use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::{MdocCopies, TrustAnchor},
    utils::keys::{KeyFactory, MdocEcdsaKey},
};

#[cfg(feature = "wallet_deps")]
pub use client::HttpPidIssuerClient;

pub use client::HttpOpenidPidIssuerClient;

use crate::digid::DigidSession;

#[cfg(any(test, feature = "mock"))]
pub use self::mock::MockPidIssuerClient;

#[derive(Debug, thiserror::Error)]
pub enum PidIssuerError {
    #[error("could not get BSN from PID issuer: {0}")]
    Networking(#[from] reqwest::Error),
    #[error("could not get BSN from PID issuer: {0} - Response body: {1}")]
    Response(#[source] reqwest::Error, String),
    #[error("mdoc error: {0}")]
    MdocError(#[from] nl_wallet_mdoc::Error),
    #[error("openid4vci error: {0}")]
    Openid(#[from] openid4vc::Error),
}

pub trait OpenidPidIssuerClient {
    fn has_session(&self) -> bool;

    async fn start_retrieve_pid<DGS: DigidSession>(
        &mut self,
        digid_session: DGS,
        base_url: &Url,
        pre_authorized_code: String,
    ) -> Result<Vec<UnsignedMdoc>, PidIssuerError>;

    async fn accept_pid<K: MdocEcdsaKey>(
        &mut self,
        mdoc_trust_anchors: &[TrustAnchor<'_>],
        key_factory: impl KeyFactory<Key = K>,
        credential_issuer_identifier: &Url,
    ) -> Result<Vec<MdocCopies>, PidIssuerError>;

    async fn reject_pid(&mut self) -> Result<(), PidIssuerError>;
}

pub trait PidIssuerClient {
    fn has_session(&self) -> bool;

    async fn start_retrieve_pid(
        &mut self,
        base_url: &Url,
        access_token: &str,
    ) -> Result<Vec<UnsignedMdoc>, PidIssuerError>;

    async fn accept_pid<K: MdocEcdsaKey>(
        &mut self,
        mdoc_trust_anchors: &[TrustAnchor<'_>],
        key_factory: &impl KeyFactory<Key = K>,
    ) -> Result<Vec<MdocCopies>, PidIssuerError>;

    async fn reject_pid(&mut self) -> Result<(), PidIssuerError>;
}
