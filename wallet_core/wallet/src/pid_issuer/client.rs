use std::sync::Arc;

use async_trait::async_trait;
use futures::future::TryFutureExt;
use http::{header, HeaderMap, HeaderValue};
use url::Url;

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    holder::{CborHttpClient, IssuanceUserConsent, TrustAnchor, Wallet as MdocWallet},
    utils::mdocs_map::MdocsMap,
    ServiceEngagement,
};
use wallet_common::keys::software::SoftwareEcdsaKey;

use crate::utils::reqwest::default_reqwest_client_builder;

use super::{PidRetriever, PidRetrieverError};

// TODO: The `mdoc_wallet` field uses `Arc<>` just for testing now.
//       This should be removed as soon as actual storage is implemented.
pub struct PidIssuerClient {
    http_client: reqwest::Client,
    mdoc_wallet: Arc<MdocWallet<MdocsMap>>,
}

impl PidIssuerClient {
    pub fn new(mdoc_wallet: Arc<MdocWallet<MdocsMap>>) -> Self {
        let http_client = default_reqwest_client_builder()
            .default_headers(HeaderMap::from_iter([(
                header::ACCEPT,
                HeaderValue::from_static("application/json"),
            )]))
            .build()
            .expect("Could not build reqwest HTTP client");

        PidIssuerClient {
            http_client,
            mdoc_wallet,
        }
    }
}

impl Default for PidIssuerClient {
    fn default() -> Self {
        Self::new(Arc::new(MdocWallet::new(MdocsMap::new())))
    }
}

#[async_trait]
impl PidRetriever for PidIssuerClient {
    async fn retrieve_pid<'a>(
        &self,
        base_url: &Url,
        mdoc_trust_anchors: &[TrustAnchor<'a>],
        access_token: &str,
    ) -> Result<(), PidRetrieverError> {
        let url = base_url
            .join("start")
            .expect("Could not create \"start\" URL from PID issuer base URL");

        let service_engagement = self
            .http_client
            .post(url)
            .bearer_auth(access_token)
            .send()
            .map_err(PidRetrieverError::from)
            .and_then(|response| async {
                // Try to get the body from any 4xx or 5xx error responses,
                // in order to create an Error::PidIssuerResponse.
                // TODO: Implement proper JSON-based error reporting
                //       for the mock PID issuer.
                match response.error_for_status_ref() {
                    Ok(_) => Ok(response),
                    Err(error) => {
                        let error = match response.text().await.ok() {
                            Some(body) => PidRetrieverError::PidIssuerResponse(error, body),
                            None => PidRetrieverError::PidIssuer(error),
                        };

                        Err(error)
                    }
                }
            })
            .await?
            .json::<ServiceEngagement>()
            .await?;

        let http_client = default_reqwest_client_builder()
            .build()
            .expect("Could not build reqwest HTTP client");

        self.mdoc_wallet
            .do_issuance::<SoftwareEcdsaKey>(
                service_engagement,
                &always_agree(),
                &CborHttpClient(http_client),
                mdoc_trust_anchors,
            )
            .await?;

        Ok(())
    }
}

fn always_agree() -> impl IssuanceUserConsent {
    struct AlwaysAgree;
    #[async_trait]
    impl IssuanceUserConsent for AlwaysAgree {
        async fn ask(&self, _: &[UnsignedMdoc]) -> bool {
            true
        }
    }
    AlwaysAgree
}
