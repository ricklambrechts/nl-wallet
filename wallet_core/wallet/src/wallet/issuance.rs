use http::{header, HeaderMap, HeaderValue};
use p256::ecdsa::signature;
use tracing::{info, instrument};
use url::Url;

use error_category::{sentry_capture_error, ErrorCategory};
use nl_wallet_mdoc::utils::{cose::CoseError, issuer_auth::IssuerRegistration, x509::MdocCertificateExtension};
use openid4vc::{
    issuance_session::{HttpIssuanceSession, IssuanceSession, IssuanceSessionError},
    token::AttestationPreviewError,
};
use platform_support::hw_keystore::PlatformEcdsaKey;
use wallet_common::{jwt::JwtError, reqwest::trusted_reqwest_client_builder, urls};

use crate::{
    account_provider::AccountProviderClient,
    config::{ConfigurationRepository, UNIVERSAL_LINK_BASE_URL},
    document::{Document, DocumentMdocError, PID_DOCTYPE},
    instruction::{InstructionClient, InstructionError, RemoteEcdsaKeyError, RemoteEcdsaKeyFactory},
    issuance::{DigidSession, DigidSessionError, HttpDigidSession},
    storage::{Storage, StorageError, WalletEvent},
};

use super::{documents::DocumentsError, history::EventStorageError, Wallet};

pub(super) enum PidIssuanceSession<DS = HttpDigidSession, IS = HttpIssuanceSession> {
    Digid(DS),
    Openid4vci(IS),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum PidIssuanceError {
    #[error("wallet is not registered")]
    #[category(expected)]
    NotRegistered,
    #[error("wallet is locked")]
    #[category(expected)]
    Locked,
    #[error("issuance session is not in the correct state")]
    #[category(expected)]
    SessionState,
    #[error("PID already present")]
    #[category(expected)]
    PidAlreadyPresent,
    #[error("could not start DigiD session: {0}")]
    DigidSessionStart(#[source] DigidSessionError),
    #[error("could not finish DigiD session: {0}")]
    DigidSessionFinish(#[source] DigidSessionError),
    #[error("could not retrieve PID from issuer: {0}")]
    PidIssuer(#[from] IssuanceSessionError),
    #[error("error sending instruction to Wallet Provider: {0}")]
    Instruction(#[from] InstructionError),
    #[error("invalid signature received from Wallet Provider: {0}")]
    #[category(critical)]
    Signature(#[from] signature::Error),
    #[error("no signature received from Wallet Provider")]
    #[category(critical)]
    MissingSignature,
    #[error("could not interpret mdoc attributes: {0}")]
    MdocDocument(#[from] DocumentMdocError),
    #[error("could not insert mdocs in database: {0}")]
    MdocStorage(#[source] StorageError),
    #[error("could not store event in history database: {0}")]
    EventStorage(#[source] EventStorageError),
    #[error("key '{0}' not found in Wallet Provider")]
    #[category(pd)]
    KeyNotFound(String),
    #[error("invalid issuer certificate: {0}")]
    InvalidIssuerCertificate(#[source] CoseError),
    #[error("issuer not authenticated")]
    #[category(critical)]
    MissingIssuerRegistration,
    #[error("could not read documents from storage: {0}")]
    Document(#[source] DocumentsError),
    #[error("failed to read issuer registration from issuer certificate: {0}")]
    AttestationPreview(#[from] AttestationPreviewError),
}

impl<CR, S, PEK, APC, DS, IS, MDS> Wallet<CR, S, PEK, APC, DS, IS, MDS>
where
    CR: ConfigurationRepository,
    DS: DigidSession,
    IS: IssuanceSession,
    S: Storage,
{
    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn create_pid_issuance_auth_url(&mut self) -> Result<Url, PidIssuanceError> {
        info!("Generating DigiD auth URL, starting OpenID connect discovery");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        info!("Checking if there is an active issuance session");
        if self.issuance_session.is_some() {
            return Err(PidIssuanceError::SessionState);
        }

        info!("Checking if a pid is already present");
        let has_pid = self
            .storage
            .get_mut()
            .has_any_mdocs_with_doctype(PID_DOCTYPE)
            .await
            .map_err(PidIssuanceError::MdocStorage)?;
        if has_pid {
            return Err(PidIssuanceError::PidAlreadyPresent);
        }

        let pid_issuance_config = &self.config_repository.config().pid_issuance;
        let (session, auth_url) = DS::start(
            pid_issuance_config.clone(),
            urls::issuance_base_uri(&UNIVERSAL_LINK_BASE_URL).as_ref().to_owned(),
        )
        .await
        .map_err(PidIssuanceError::DigidSessionStart)?;

        info!("DigiD auth URL generated");
        self.issuance_session.replace(PidIssuanceSession::Digid(session));

        Ok(auth_url)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub fn has_active_pid_issuance_session(&self) -> Result<bool, PidIssuanceError> {
        info!("Checking for active PID issuance session");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        let has_active_session = self.issuance_session.is_some();

        Ok(has_active_session)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn cancel_pid_issuance(&mut self) -> Result<(), PidIssuanceError> {
        info!("PID issuance cancelled / rejected");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        info!("Checking if there is an active issuance session");
        let issuance_session = self.issuance_session.take().ok_or(PidIssuanceError::SessionState)?;

        if let PidIssuanceSession::Openid4vci(pid_issuer) = issuance_session {
            info!("Rejecting PID");
            pid_issuer.reject_issuance().await?;
        }

        Ok(())
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn continue_pid_issuance(&mut self, redirect_uri: Url) -> Result<Vec<Document>, PidIssuanceError> {
        info!("Received DigiD redirect URI, processing URI and retrieving access token");

        info!("Checking if registered");
        if self.registration.is_none() {
            return Err(PidIssuanceError::NotRegistered);
        }

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        info!("Checking if there is an active DigiD issuance session");
        if !matches!(self.issuance_session, Some(PidIssuanceSession::Digid(_))) {
            return Err(PidIssuanceError::SessionState);
        }

        // Take ownership of the active session, now that we know that it exists.
        let session = match self.issuance_session.take().unwrap() {
            PidIssuanceSession::Digid(session) => session,
            PidIssuanceSession::Openid4vci(_) => panic!(),
        };

        let token_request = session
            .into_token_request(redirect_uri)
            .await
            .map_err(PidIssuanceError::DigidSessionFinish)?;

        let pid_issuance_config = &self.config_repository.config().pid_issuance;
        let http_client = trusted_reqwest_client_builder(pid_issuance_config.digid_trust_anchors())
            .default_headers(HeaderMap::from_iter([(
                header::ACCEPT,
                HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
            )]))
            .build()
            .expect("Could not build reqwest HTTP client");
        let config = self.config_repository.config();

        let (pid_issuer, attestation_previews) = IS::start_issuance(
            http_client.into(),
            config.pid_issuance.pid_issuer_url.clone(),
            token_request,
            &config.mdoc_trust_anchors(),
        )
        .await?;

        info!("PID received successfully from issuer, returning preview documents");
        let mut documents = attestation_previews
            .into_iter()
            .map(|preview| {
                let (unsigned_mdoc, issuer) = preview.try_into()?;
                Ok(Document::from_unsigned_mdoc(unsigned_mdoc, *issuer)?)
            })
            .collect::<Result<Vec<_>, PidIssuanceError>>()?;
        documents.sort_by_key(Document::priority);

        self.issuance_session
            .replace(PidIssuanceSession::Openid4vci(pid_issuer));

        Ok(documents)
    }

    #[instrument(skip_all)]
    #[sentry_capture_error]
    pub async fn accept_pid_issuance(&mut self, pin: String) -> Result<(), PidIssuanceError>
    where
        S: Storage,
        PEK: PlatformEcdsaKey,
        APC: AccountProviderClient,
    {
        info!("Accepting PID issuance");

        info!("Checking if registered");
        let registration = self
            .registration
            .as_ref()
            .ok_or_else(|| PidIssuanceError::NotRegistered)?;

        info!("Checking if locked");
        if self.lock.is_locked() {
            return Err(PidIssuanceError::Locked);
        }

        info!("Checking if there is an active PID issuance session");
        let pid_issuer = match self.issuance_session.as_ref().ok_or(PidIssuanceError::SessionState)? {
            PidIssuanceSession::Digid(_) => Err(PidIssuanceError::SessionState)?,
            PidIssuanceSession::Openid4vci(pid_issuer) => pid_issuer,
        };

        let config = self.config_repository.config();

        let instruction_result_public_key = config.account_server.instruction_result_public_key.clone().into();

        let remote_instruction = InstructionClient::new(
            pin,
            &self.storage,
            &registration.hw_privkey,
            &self.account_provider_client,
            &registration.data,
            &config.account_server.base_url,
            &instruction_result_public_key,
        );
        let remote_key_factory = RemoteEcdsaKeyFactory::new(&remote_instruction);

        info!("Accepting PID by signing mdoc using Wallet Provider");

        let mdocs_result = pid_issuer
            .accept_issuance(
                &config.mdoc_trust_anchors(),
                &remote_key_factory,
                config.pid_issuance.pid_issuer_url.clone(),
            )
            .await
            .map_err(|error| {
                match error {
                    // We knowingly call unwrap() on the downcast to `RemoteEcdsaKeyError` here because we know
                    // that it is the error type of the `RemoteEcdsaKeyFactory` we provide above.
                    IssuanceSessionError::PrivateKeyGeneration(error)
                    | IssuanceSessionError::Jwt(JwtError::Signing(error)) => {
                        match *error.downcast::<RemoteEcdsaKeyError>().unwrap() {
                            RemoteEcdsaKeyError::Instruction(error) => PidIssuanceError::Instruction(error),
                            RemoteEcdsaKeyError::Signature(error) => PidIssuanceError::Signature(error),
                            RemoteEcdsaKeyError::KeyNotFound(identifier) => PidIssuanceError::KeyNotFound(identifier),
                            RemoteEcdsaKeyError::MissingSignature => PidIssuanceError::MissingSignature,
                        }
                    }
                    _ => PidIssuanceError::PidIssuer(error),
                }
            });

        // If the Wallet Provider returns either a PIN timeout or a permanent block,
        // wipe the contents of the wallet and return it to its initial state.
        if matches!(
            mdocs_result,
            Err(PidIssuanceError::Instruction(
                InstructionError::Timeout { .. } | InstructionError::Blocked
            ))
        ) {
            self.reset_to_initial_state().await;
        }
        let mdocs = mdocs_result?;

        info!("Isuance succeeded; removing issuance session state");
        self.issuance_session.take();

        // Prepare events before storing mdocs, to avoid cloning mdocs
        let event = {
            // Extract first copy from cred_copies
            let mdocs = mdocs
                .iter()
                .flat_map(|mdoc| mdoc.cred_copies.first())
                .cloned()
                .collect::<Vec<_>>();

            // Validate all issuer_certificates
            for mdoc in mdocs.iter() {
                let certificate = mdoc
                    .issuer_certificate()
                    .map_err(PidIssuanceError::InvalidIssuerCertificate)?;

                // Verify that the certificate contains IssuerRegistration
                if matches!(IssuerRegistration::from_certificate(&certificate), Err(_) | Ok(None)) {
                    return Err(PidIssuanceError::MissingIssuerRegistration);
                }
            }
            WalletEvent::new_issuance(mdocs.try_into().map_err(PidIssuanceError::InvalidIssuerCertificate)?)
        };

        info!("PID accepted, storing mdoc in database");
        self.storage
            .get_mut()
            .insert_mdocs(mdocs)
            .await
            .map_err(PidIssuanceError::MdocStorage)?;

        self.store_history_event(event)
            .await
            .map_err(PidIssuanceError::EventStorage)?;

        self.emit_documents().await.map_err(PidIssuanceError::Document)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use mockall::predicate::*;
    use openid4vc::{
        mock::MockIssuanceSession,
        oidc::OidcError,
        token::{AttestationPreview, TokenRequest, TokenRequestGrantType},
    };
    use rstest::rstest;
    use serial_test::serial;
    use url::Url;

    use crate::{
        document::{self, DocumentPersistence},
        issuance::MockDigidSession,
        storage::StorageState,
        wallet::history::HistoryEvent,
    };

    use super::{
        super::test::{self, WalletWithMocks, ISSUER_KEY},
        *,
    };

    #[tokio::test]
    #[serial(MockDigidSession)]
    async fn test_create_pid_issuance_auth_url() {
        const AUTH_URL: &str = "http://example.com/auth";

        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        assert!(wallet.issuance_session.is_none());

        // Set up a mock DigiD session.
        let session_start_context = MockDigidSession::start_context();
        session_start_context.expect().returning(|_, _| {
            let client = MockDigidSession::default();
            Ok((client, Url::parse(AUTH_URL).unwrap()))
        });

        // Have the `Wallet` generate a DigiD authentication URL and test it.
        let auth_url = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect("Could not generate PID issuance auth URL");

        assert_eq!(auth_url.as_str(), AUTH_URL);
        assert!(wallet.issuance_session.is_some());
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_locked() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Creating a DigiD authentication URL on
        // a locked wallet should result in an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Creating a DigiD authentication URL on an
        // unregistered wallet should result in an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_session_state_digid() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up a mock DigiD session.
        wallet.issuance_session = Some(PidIssuanceSession::Digid(MockDigidSession::default()));

        // Creating a DigiD authentication URL on a `Wallet` that
        // has an active DigiD session should return an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    #[tokio::test]
    async fn test_create_pid_issuance_auth_url_error_session_state_pid_issuer() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Setup a mock OpenID4VCI session.
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(MockIssuanceSession::default()));

        // Creating a DigiD authentication URL on a `Wallet` that has
        // an active OpenID4VCI session should return an error.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    #[tokio::test]
    #[serial(MockDigidSession)]
    async fn test_create_pid_issuance_auth_url_error_digid_session_start() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Make DigiD session starting return an error.
        let session_start_context = MockDigidSession::start_context();
        session_start_context
            .expect()
            .return_once(|_, _| Err(OidcError::NoAuthCode.into()));

        // The error should be forwarded when attempting to create a DigiD authentication URL.
        let error = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("PID issuance auth URL generation should have resulted in error");

        assert_matches!(error, PidIssuanceError::DigidSessionStart(_));
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_digid() {
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up a mock DigiD session.
        wallet.issuance_session = Some(PidIssuanceSession::Digid(MockDigidSession::default()));

        assert!(wallet.issuance_session.is_some());

        // Cancelling PID issuance should clear this session.
        wallet
            .cancel_pid_issuance()
            .await
            .expect("Could not cancel PID issuance");

        assert!(wallet.issuance_session.is_none());
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_pid() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up the `PidIssuerClient`
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client.expect_reject().return_once(|| Ok(()));
            client
        };
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Cancelling PID issuance should not fail.
        wallet
            .cancel_pid_issuance()
            .await
            .expect("Could not cancel PID issuance");

        assert!(wallet.issuance_session.is_none());
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Cancelling PID issuance on a locked wallet should result in an error.
        let error = wallet
            .cancel_pid_issuance()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Cancelling PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .cancel_pid_issuance()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Cancelling PID issuance on a wallet with no
        // active DigiD session should result in an error.
        let error = wallet
            .cancel_pid_issuance()
            .await
            .expect_err("Cancelling PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    const REDIRECT_URI: &str = "redirect://here";

    #[tokio::test]
    #[serial(MockIssuanceSession)]
    async fn test_continue_pid_issuance() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up a mock DigiD session that returns a token request.
        let issuance_session = {
            let mut session = MockDigidSession::default();

            session.expect_into_token_request().return_once(|_uri| {
                Ok(TokenRequest {
                    grant_type: TokenRequestGrantType::PreAuthorizedCode {
                        pre_authorized_code: "123".to_string().into(),
                    },
                    code_verifier: None,
                    client_id: None,
                    redirect_uri: None,
                })
            });

            session
        };
        wallet.issuance_session = Some(PidIssuanceSession::Digid(issuance_session));

        // Set up the `MockIssuanceSession` to return one `AttestationPreview`.
        let start_context = MockIssuanceSession::start_context();
        start_context.expect().return_once(|| {
            Ok((
                MockIssuanceSession::new(),
                vec![AttestationPreview::MsoMdoc {
                    unsigned_mdoc: document::create_full_unsigned_pid_mdoc(),
                    issuer: ISSUER_KEY.issuance_key.certificate().clone(),
                }],
            ))
        });

        // Continuing PID issuance should result in one preview `Document`.
        let documents = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect("Could not continue PID issuance");

        assert_eq!(documents.len(), 1);
        assert_matches!(documents[0].persistence, DocumentPersistence::InMemory);
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Continuing PID issuance on a locked wallet should result in an error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::Locked);
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Continuing PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_continue_pid_issuance_error_session_state() {
        // Prepare a registered wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Continuing PID issuance on a wallet with no active `DigidSession` should result in an error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::SessionState);
    }

    #[tokio::test]
    #[serial(MockIssuanceSession)]
    async fn test_continue_pid_issuance_error_pid_issuer() {
        // Prepare a registered wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up a mock DigiD session that returns a token request.
        let digid_session = {
            let mut session = MockDigidSession::default();

            session.expect_into_token_request().return_once(|_uri| {
                Ok(TokenRequest {
                    grant_type: TokenRequestGrantType::PreAuthorizedCode {
                        pre_authorized_code: "123".to_string().into(),
                    },
                    code_verifier: None,
                    client_id: None,
                    redirect_uri: None,
                })
            });

            session
        };
        wallet.issuance_session = Some(PidIssuanceSession::Digid(digid_session));

        // Set up the `MockIssuanceSession` to return an error.
        let start_context = MockIssuanceSession::start_context();
        start_context
            .expect()
            .return_once(|| Err(IssuanceSessionError::MissingNonce));

        // Continuing PID issuance on a wallet should forward this error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::PidIssuer(_));
    }

    #[tokio::test]
    #[serial(MockIssuanceSession)]
    async fn test_continue_pid_issuance_error_document() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up a mock DigiD session that returns a token request.
        let digid_session = {
            let mut session = MockDigidSession::default();

            session.expect_into_token_request().return_once(|_uri| {
                Ok(TokenRequest {
                    grant_type: TokenRequestGrantType::PreAuthorizedCode {
                        pre_authorized_code: "123".to_string().into(),
                    },
                    code_verifier: None,
                    client_id: None,
                    redirect_uri: None,
                })
            });

            session
        };
        wallet.issuance_session = Some(PidIssuanceSession::Digid(digid_session));

        // Set up the `MockIssuanceSession` to return an `AttestationPreview` with an unknown doctype.
        let start_context = MockIssuanceSession::start_context();
        start_context.expect().return_once(|| {
            let mut unsigned_mdoc = document::create_full_unsigned_pid_mdoc();
            unsigned_mdoc.doc_type = "foobar".to_string();

            Ok((
                MockIssuanceSession::new(),
                vec![AttestationPreview::MsoMdoc {
                    unsigned_mdoc,
                    issuer: ISSUER_KEY.issuance_key.certificate().clone(),
                }],
            ))
        });

        // Continuing PID issuance when receiving an unknown mdoc should result in an error.
        let error = wallet
            .continue_pid_issuance(Url::parse(REDIRECT_URI).unwrap())
            .await
            .expect_err("Continuing PID issuance should have resulted in error");

        assert_matches!(error, PidIssuanceError::MdocDocument(_));
    }

    #[tokio::test]
    async fn test_cancel_pid_issuance_error_pid_issuer() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Set up a mock OpenID4VCI session that expects to be rejected, which returns an error.
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client
                .expect_reject()
                .return_once(|| Err(IssuanceSessionError::MissingNonce));
            client
        };
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Canceling PID issuance on a wallet should forward this error.
        let error = wallet
            .cancel_pid_issuance()
            .await
            .expect_err("Rejecting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::PidIssuer(_));
    }

    const PIN: &str = "051097";

    #[tokio::test]
    async fn test_accept_pid_issuance() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Register mock document_callback
        let documents = test::setup_mock_documents_callback(&mut wallet).await.unwrap();

        // Register mock recent_history_callback
        let events = test::setup_mock_recent_history_callback(&mut wallet).await.unwrap();

        // Create a mock OpenID4VCI session that accepts the PID with a single
        // instance of `MdocCopies`, which contains a single valid `Mdoc`.
        let mdoc = test::create_full_pid_mdoc().await;
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client.expect_accept().return_once(|| Ok(vec![vec![mdoc].into()]));
            client
        };
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Accept the PID issuance with the PIN.
        wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect("Could not accept PID issuance");

        {
            // Test which `Document` instances we have received through the callback.
            let documents = documents.lock();

            // The first entry should be empty, because there are no mdocs in the database.
            assert_eq!(documents.len(), 2);
            assert!(documents[0].is_empty());

            // The second entry should contain a single document with the PID.
            assert_eq!(documents[1].len(), 1);
            let document = &documents[1][0];
            assert_matches!(document.persistence, DocumentPersistence::Stored(_));
            assert_eq!(document.doc_type, "com.example.pid");

            // Test that one successful issuance event is logged
            let events = events.lock();
            assert_eq!(events.len(), 2);
            assert!(events[0].is_empty());
            assert_eq!(events[1].len(), 1);
            assert_matches!(&events[1][0], HistoryEvent::Issuance { .. });

            assert!(wallet.has_registration());
            assert!(!wallet.is_locked());
        }

        // Starting another PID issuance should fail
        const AUTH_URL: &str = "http://example.com/auth";
        // Set up a mock DigiD session.
        let session_start_context = MockDigidSession::start_context();
        session_start_context.expect().returning(|_, _| {
            let client = MockDigidSession::default();
            Ok((client, Url::parse(AUTH_URL).unwrap()))
        });

        let err = wallet
            .create_pid_issuance_auth_url()
            .await
            .expect_err("creating new PID issuance auth URL when there already is a PID should fail");
        assert_matches!(err, PidIssuanceError::PidAlreadyPresent);
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_missing_issuer_registration() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Create a mock OpenID4VCI session that accepts the PID with a single instance of `MdocCopies`, which contains
        // a single valid `Mdoc`, but signed with a Certificate that is missing IssuerRegistration
        let mdoc = test::create_full_pid_mdoc_unauthenticated().await;
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client.expect_accept().return_once(|| Ok(vec![vec![mdoc].into()]));
            client
        };
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Accept the PID issuance with the PIN.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::MissingIssuerRegistration);

        // No issuance event is logged
        let events = wallet.storage.read().await.fetch_wallet_events().await.unwrap();
        assert!(events.is_empty());

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_unregistered() {
        // Prepare an unregistered wallet.
        let mut wallet = WalletWithMocks::new_unregistered().await;

        // Accepting PID issuance on an unregistered wallet should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::NotRegistered);
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_locked() {
        // Prepare a registered and locked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        wallet.lock();

        // Accepting PID issuance on a locked wallet should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::Locked);

        assert!(wallet.has_registration());
        assert!(wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_session_state() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Accepting PID issuance on a `Wallet` with a `PidIssuerClient`
        // that has no session should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::SessionState);

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    async fn test_accept_pid_issuance_error_remote_key(
        key_error: RemoteEcdsaKeyError,
    ) -> (WalletWithMocks, PidIssuanceError) {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Have the mock OpenID4VCI session return a particular `RemoteEcdsaKeyError` upon accepting.
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client
                .expect_accept()
                .return_once(|| Err(IssuanceSessionError::Jwt(JwtError::Signing(Box::new(key_error)))));
            client
        };
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        (wallet, error)
    }

    #[rstest]
    #[case(InstructionError::IncorrectPin { attempts_left_in_round: 1, is_final_round: false }, false)]
    #[case(InstructionError::Timeout { timeout_millis: 10_000 }, true)]
    #[case(InstructionError::Blocked, true)]
    #[case(InstructionError::InstructionValidation, false)]
    #[tokio::test]
    async fn test_accept_pid_issuance_error_instruction(
        #[case] instruction_error: InstructionError,
        #[case] expect_reset: bool,
    ) {
        let (mut wallet, error) =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::from(instruction_error)).await;

        // Test that this error is converted to the appropriate variant of `PidIssuanceError`.
        assert_matches!(error, PidIssuanceError::Instruction(_));

        // Test the state of the Wallet, based on if we expect a reset for this InstructionError.
        if expect_reset {
            assert!(!wallet.has_registration());
            assert!(wallet.is_locked());
            assert_matches!(
                wallet.storage.get_mut().state().await.unwrap(),
                StorageState::Uninitialized
            );
        } else {
            assert!(wallet.has_registration());
            assert!(!wallet.is_locked());
            assert_matches!(wallet.storage.get_mut().state().await.unwrap(), StorageState::Opened);
        }
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_signature() {
        let (wallet, error) =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::from(signature::Error::default())).await;

        // Test that this error is converted to the appropriate variant of `PidIssuanceError`.
        assert_matches!(error, PidIssuanceError::Signature(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_key_not_found() {
        let (wallet, error) =
            test_accept_pid_issuance_error_remote_key(RemoteEcdsaKeyError::KeyNotFound("not found".to_string())).await;

        // Test that this error is converted to the appropriate variant of `PidIssuanceError`.
        assert_matches!(error, PidIssuanceError::KeyNotFound(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_pid_issuer() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Have the mock OpenID4VCI session return an error upon accepting.
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client
                .expect_accept()
                .return_once(|| Err(IssuanceSessionError::MissingNonce));
            client
        };
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::PidIssuer(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }

    #[tokio::test]
    async fn test_accept_pid_issuance_error_storage() {
        // Prepare a registered and unlocked wallet.
        let mut wallet = WalletWithMocks::new_registered_and_unlocked().await;

        // Have the mock OpenID4VCI session report some mdocs upon accepting.
        let mdoc = test::create_full_pid_mdoc().await;
        let pid_issuer = {
            let mut client = MockIssuanceSession::new();
            client.expect_accept().return_once(|| Ok(vec![vec![mdoc].into()]));
            client
        };
        wallet.issuance_session = Some(PidIssuanceSession::Openid4vci(pid_issuer));

        // Have the mdoc storage return an error on query.
        wallet.storage.get_mut().has_query_error = true;

        // Accepting PID issuance should result in an error.
        let error = wallet
            .accept_pid_issuance(PIN.to_string())
            .await
            .expect_err("Accepting PID issuance should have resulted in an error");

        assert_matches!(error, PidIssuanceError::MdocStorage(_));

        assert!(wallet.has_registration());
        assert!(!wallet.is_locked());
    }
}
