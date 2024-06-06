mod uri;

use openid4vc::disclosure_session::{HttpVpMessageClient, VpClientError};
use url::Url;
use uuid::Uuid;

use nl_wallet_mdoc::{
    holder::{
        CborHttpClient, DisclosureMissingAttributes, DisclosureProposal, DisclosureResult, DisclosureSession,
        MdocDataSource, ProposedAttributes, TrustAnchor,
    },
    identifiers::AttributeIdentifier,
    utils::{
        keys::{KeyFactory, MdocEcdsaKey},
        reader_auth::ReaderRegistration,
        x509::Certificate,
    },
    verifier::SessionType,
};
use wallet_common::{config::wallet_config::BaseUrl, reqwest::default_reqwest_client_builder};

pub use nl_wallet_mdoc::holder::DisclosureUriSource;

pub use self::uri::{DisclosureUriError, IsoDisclosureUriData, VpDisclosureUriData};

#[cfg(any(test, feature = "mock"))]
pub use self::mock::{MockMdocDisclosureProposal, MockMdocDisclosureSession};

#[derive(Debug)]
pub enum MdocDisclosureSessionState<M, P> {
    MissingAttributes(M),
    Proposal(P),
}

pub trait MdocDisclosureSession<D> {
    type MissingAttributes: MdocDisclosureMissingAttributes;
    type Proposal: MdocDisclosureProposal;
    type DisclosureUriData;
    type Error: std::error::Error + Into<crate::wallet::DisclosureError>;

    fn parse_url(uri: &Url, base_uri: &Url) -> Result<Self::DisclosureUriData, DisclosureUriError>;

    async fn start<'a>(
        disclosure_uri: Self::DisclosureUriData,
        reader_engagement_source: DisclosureUriSource,
        mdoc_data_source: &D,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> nl_wallet_mdoc::Result<Self, Self::Error>
    where
        Self: Sized;

    fn rp_certificate(&self) -> &Certificate;
    fn reader_registration(&self) -> &ReaderRegistration;
    fn session_state(&self) -> MdocDisclosureSessionState<&Self::MissingAttributes, &Self::Proposal>;
    fn session_type(&self) -> SessionType;

    async fn terminate(self) -> Result<(), Self::Error>;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MdocDisclosureMissingAttributes {
    fn missing_attributes(&self) -> &[AttributeIdentifier];
}

pub trait MdocDisclosureProposal {
    fn return_url(&self) -> Option<&Url>;
    fn proposed_source_identifiers(&self) -> Vec<Uuid>;
    fn proposed_attributes(&self) -> ProposedAttributes;

    async fn disclose<KF, K>(&self, key_factory: &KF) -> DisclosureResult<Option<BaseUrl>>
    where
        KF: KeyFactory<Key = K>,
        K: MdocEcdsaKey;
}

type VpDisclosureSession = openid4vc::disclosure_session::DisclosureSession<HttpVpMessageClient, Uuid>;
type VpDisclosureMissingAttributes = openid4vc::disclosure_session::DisclosureMissingAttributes<HttpVpMessageClient>;
type VpDisclosureProposal = openid4vc::disclosure_session::DisclosureProposal<HttpVpMessageClient, Uuid>;

impl<D> MdocDisclosureSession<D> for VpDisclosureSession
where
    D: MdocDataSource<MdocIdentifier = Uuid>,
{
    type MissingAttributes = VpDisclosureMissingAttributes;
    type Proposal = VpDisclosureProposal;
    type DisclosureUriData = VpDisclosureUriData;
    type Error = VpClientError;

    fn parse_url(uri: &Url, base_uri: &Url) -> Result<Self::DisclosureUriData, DisclosureUriError> {
        VpDisclosureUriData::parse_from_uri(uri, base_uri)
    }

    async fn start<'a>(
        disclosure_uri: VpDisclosureUriData,
        uri_source: DisclosureUriSource,
        mdoc_data_source: &D,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let client = default_reqwest_client_builder()
            .build()
            .expect("Could not build reqwest HTTP client")
            .into();
        Self::start(
            client,
            &disclosure_uri.query,
            uri_source,
            mdoc_data_source,
            trust_anchors,
        )
        .await
    }

    fn rp_certificate(&self) -> &Certificate {
        self.verifier_certificate()
    }

    fn reader_registration(&self) -> &ReaderRegistration {
        self.reader_registration()
    }

    fn session_state(&self) -> MdocDisclosureSessionState<&VpDisclosureMissingAttributes, &VpDisclosureProposal> {
        match self {
            Self::MissingAttributes(session) => MdocDisclosureSessionState::MissingAttributes(session),
            Self::Proposal(session) => MdocDisclosureSessionState::Proposal(session),
        }
    }

    fn session_type(&self) -> SessionType {
        self.session_type()
    }

    async fn terminate(self) -> Result<(), Self::Error> {
        self.terminate().await
    }
}

impl MdocDisclosureMissingAttributes for VpDisclosureMissingAttributes {
    fn missing_attributes(&self) -> &[AttributeIdentifier] {
        self.missing_attributes()
    }
}

impl MdocDisclosureProposal for VpDisclosureProposal {
    fn return_url(&self) -> Option<&Url> {
        None
    }

    fn proposed_source_identifiers(&self) -> Vec<Uuid> {
        self.proposed_source_identifiers().into_iter().copied().collect()
    }

    fn proposed_attributes(&self) -> ProposedAttributes {
        self.proposed_attributes()
    }

    async fn disclose<KF, K>(&self, key_factory: &KF) -> DisclosureResult<Option<BaseUrl>>
    where
        KF: KeyFactory<Key = K>,
        K: MdocEcdsaKey,
    {
        Ok(self.disclose(key_factory).await.unwrap())
    }
}

impl<D> MdocDisclosureSession<D> for DisclosureSession<CborHttpClient, Uuid>
where
    D: MdocDataSource<MdocIdentifier = Uuid>,
{
    type MissingAttributes = DisclosureMissingAttributes<CborHttpClient>;
    type Proposal = DisclosureProposal<CborHttpClient, Uuid>;
    type DisclosureUriData = IsoDisclosureUriData;
    type Error = nl_wallet_mdoc::Error;

    fn parse_url(uri: &Url, base_uri: &Url) -> Result<Self::DisclosureUriData, DisclosureUriError> {
        IsoDisclosureUriData::parse_from_uri(uri, base_uri)
    }

    async fn start<'a>(
        disclosure_uri: Self::DisclosureUriData,
        reader_engagement_source: DisclosureUriSource,
        mdoc_data_source: &D,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> nl_wallet_mdoc::Result<Self> {
        let http_client = default_reqwest_client_builder()
            .build()
            .expect("Could not build reqwest HTTP client");

        Self::start(
            CborHttpClient(http_client),
            &disclosure_uri.reader_engagement_bytes,
            reader_engagement_source,
            mdoc_data_source,
            trust_anchors,
        )
        .await
    }

    fn rp_certificate(&self) -> &Certificate {
        self.verifier_certificate()
    }

    fn reader_registration(&self) -> &ReaderRegistration {
        self.reader_registration()
    }

    fn session_state(
        &self,
    ) -> MdocDisclosureSessionState<
        &DisclosureMissingAttributes<CborHttpClient>,
        &DisclosureProposal<CborHttpClient, Uuid>,
    > {
        match self {
            DisclosureSession::MissingAttributes(session) => MdocDisclosureSessionState::MissingAttributes(session),
            DisclosureSession::Proposal(session) => MdocDisclosureSessionState::Proposal(session),
        }
    }

    async fn terminate(self) -> nl_wallet_mdoc::Result<()> {
        self.terminate().await
    }

    fn session_type(&self) -> SessionType {
        self.session_type()
    }
}

impl MdocDisclosureMissingAttributes for DisclosureMissingAttributes<CborHttpClient> {
    fn missing_attributes(&self) -> &[AttributeIdentifier] {
        self.missing_attributes()
    }
}

impl MdocDisclosureProposal for DisclosureProposal<CborHttpClient, Uuid> {
    fn return_url(&self) -> Option<&Url> {
        self.return_url()
    }

    fn proposed_source_identifiers(&self) -> Vec<Uuid> {
        self.proposed_source_identifiers().into_iter().copied().collect()
    }

    fn proposed_attributes(&self) -> ProposedAttributes {
        self.proposed_attributes()
    }

    async fn disclose<KF, K>(&self, key_factory: &KF) -> DisclosureResult<Option<BaseUrl>>
    where
        KF: KeyFactory<Key = K>,
        K: MdocEcdsaKey,
    {
        self.disclose(key_factory).await?;
        Ok(None)
    }
}

#[cfg(any(test, feature = "mock"))]
mod mock {
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    };

    use once_cell::sync::Lazy;
    use parking_lot::Mutex;

    use nl_wallet_mdoc::{holder::DisclosureError, server_keys::KeyPair, verifier::SessionType};

    use super::*;

    type SessionState = MdocDisclosureSessionState<MockMdocDisclosureMissingAttributes, MockMdocDisclosureProposal>;
    type MockFields = (ReaderRegistration, SessionState);

    pub static NEXT_START_ERROR: Lazy<Mutex<Option<nl_wallet_mdoc::Error>>> = Lazy::new(|| Mutex::new(None));
    pub static NEXT_MOCK_FIELDS: Lazy<Mutex<Option<MockFields>>> = Lazy::new(|| Mutex::new(None));

    // For convenience, the default `SessionState` is a proposal.
    impl Default for SessionState {
        fn default() -> Self {
            MdocDisclosureSessionState::Proposal(MockMdocDisclosureProposal::default())
        }
    }

    #[derive(Debug)]
    pub struct MockMdocDisclosureProposal {
        pub return_url: Option<Url>,
        pub proposed_source_identifiers: Vec<Uuid>,
        pub proposed_attributes: ProposedAttributes,
        pub disclosure_count: Arc<AtomicUsize>,
        pub next_error: Mutex<Option<nl_wallet_mdoc::Error>>,
        pub attributes_shared: bool,
        pub session_type: SessionType,
    }

    impl Default for MockMdocDisclosureProposal {
        fn default() -> Self {
            Self {
                return_url: Default::default(),
                proposed_source_identifiers: Default::default(),
                proposed_attributes: Default::default(),
                disclosure_count: Default::default(),
                next_error: Default::default(),
                attributes_shared: Default::default(),
                session_type: SessionType::SameDevice,
            }
        }
    }

    impl MdocDisclosureProposal for MockMdocDisclosureProposal {
        fn return_url(&self) -> Option<&Url> {
            self.return_url.as_ref()
        }

        fn proposed_source_identifiers(&self) -> Vec<Uuid> {
            self.proposed_source_identifiers.clone()
        }

        fn proposed_attributes(&self) -> ProposedAttributes {
            self.proposed_attributes.clone()
        }

        async fn disclose<KF, K>(&self, _key_factory: &KF) -> DisclosureResult<Option<BaseUrl>>
        where
            KF: KeyFactory<Key = K>,
            K: MdocEcdsaKey,
        {
            if let Some(error) = self.next_error.lock().take() {
                return Err(DisclosureError::new(self.attributes_shared, error));
            }

            self.disclosure_count
                .store(self.disclosure_count.load(Ordering::Relaxed) + 1, Ordering::Relaxed);

            Ok(None)
        }
    }

    #[derive(Debug)]
    pub struct MockMdocDisclosureSession {
        pub disclosure_uri: IsoDisclosureUriData,
        pub reader_engagement_source: DisclosureUriSource,
        pub certificate: Certificate,
        pub reader_registration: ReaderRegistration,
        pub session_state: SessionState,
        pub was_terminated: Arc<AtomicBool>,
        pub session_type: SessionType,
    }

    impl MockMdocDisclosureSession {
        pub fn next_fields(reader_registration: ReaderRegistration, session_state: SessionState) {
            NEXT_MOCK_FIELDS.lock().replace((reader_registration, session_state));
        }

        pub fn next_start_error(error: nl_wallet_mdoc::Error) {
            NEXT_START_ERROR.lock().replace(error);
        }
    }

    /// The reader key, generated once for testing.
    static READER_KEY: Lazy<KeyPair> = Lazy::new(|| {
        let reader_ca = KeyPair::generate_reader_mock_ca().unwrap();
        reader_ca
            .generate_reader_mock(ReaderRegistration::new_mock().into())
            .unwrap()
    });

    impl Default for MockMdocDisclosureSession {
        fn default() -> Self {
            Self {
                disclosure_uri: IsoDisclosureUriData {
                    reader_engagement_bytes: Default::default(),
                },
                reader_engagement_source: DisclosureUriSource::Link,
                certificate: READER_KEY.certificate().clone(),
                reader_registration: ReaderRegistration::new_mock(),
                session_state: Default::default(),
                was_terminated: Default::default(),
                session_type: SessionType::SameDevice,
            }
        }
    }

    impl<D> MdocDisclosureSession<D> for MockMdocDisclosureSession {
        type MissingAttributes = MockMdocDisclosureMissingAttributes;
        type Proposal = MockMdocDisclosureProposal;
        type DisclosureUriData = IsoDisclosureUriData;
        type Error = nl_wallet_mdoc::Error;

        fn parse_url(uri: &Url, base_uri: &Url) -> Result<Self::DisclosureUriData, DisclosureUriError> {
            IsoDisclosureUriData::parse_from_uri(uri, base_uri)
        }

        async fn start<'a>(
            disclosure_uri: Self::DisclosureUriData,
            reader_engagement_source: DisclosureUriSource,
            _mdoc_data_source: &D,
            _trust_anchors: &[TrustAnchor<'a>],
        ) -> nl_wallet_mdoc::Result<Self> {
            if let Some(error) = NEXT_START_ERROR.lock().take() {
                return Err(error);
            }

            let (reader_registration, session_state) = NEXT_MOCK_FIELDS
                .lock()
                .take()
                .unwrap_or_else(|| (ReaderRegistration::new_mock(), SessionState::default()));

            let session = MockMdocDisclosureSession {
                disclosure_uri,
                reader_engagement_source,
                reader_registration,
                session_state,
                ..Default::default()
            };

            Ok(session)
        }

        fn session_state(&self) -> MdocDisclosureSessionState<&Self::MissingAttributes, &Self::Proposal> {
            match self.session_state {
                MdocDisclosureSessionState::MissingAttributes(ref session) => {
                    MdocDisclosureSessionState::MissingAttributes(session)
                }
                MdocDisclosureSessionState::Proposal(ref session) => MdocDisclosureSessionState::Proposal(session),
            }
        }

        fn reader_registration(&self) -> &ReaderRegistration {
            &self.reader_registration
        }

        async fn terminate(self) -> nl_wallet_mdoc::Result<()> {
            self.was_terminated.store(true, Ordering::Relaxed);

            Ok(())
        }

        fn rp_certificate(&self) -> &Certificate {
            &self.certificate
        }

        fn session_type(&self) -> SessionType {
            self.session_type
        }
    }
}
