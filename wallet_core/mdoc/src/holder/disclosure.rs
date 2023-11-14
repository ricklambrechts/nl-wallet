use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use coset::{iana, CoseMac0Builder, Header, HeaderBuilder};
use futures::future::try_join_all;
use indexmap::{IndexMap, IndexSet};
use p256::{elliptic_curve::rand_core::OsRng, PublicKey, SecretKey};
use url::Url;
use webpki::TrustAnchor;

use wallet_common::{
    generator::{Generator, TimeGenerator},
    keys::SecureEcdsaKey,
};

use crate::{
    basic_sa_ext::Entry,
    identifiers::AttributeIdentifier,
    iso::*,
    utils::{
        cose::{sign_cose, ClonePayload},
        crypto::{dh_hmac_key, SessionKey, SessionKeyUser},
        keys::{KeyFactory, MdocEcdsaKey},
        reader_auth::ReaderRegistration,
        serialization::{cbor_deserialize, cbor_serialize, CborSeq, TaggedBytes},
        x509::{Certificate, CertificateType, CertificateUsage},
    },
    verifier::SessionType,
    Error, Result,
};

use super::{HolderError, HttpClient, Mdoc, MdocRetriever, Wallet};

const REFERRER_URL: &str = "https://referrer.url/";

/// This trait needs to be implemented by an entity that stores mdocs.
#[async_trait]
pub trait MdocDataSource {
    // TODO: this trait should eventually replace MdocRetriever
    //       once disclosure is fully implemented.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Return all `Mdoc` entries from storage that match a set of doc types.
    /// The result is a `Vec` of `Vec<Mdoc>` with the same `doc_type`. The order
    /// of the result is determined by the implementor.
    async fn mdoc_by_doc_types(&self, doc_types: &HashSet<&str>) -> std::result::Result<Vec<Vec<Mdoc>>, Self::Error>;
}

pub type ProposedAttributes = IndexMap<DocType, IndexMap<NameSpace, Vec<Entry>>>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct DisclosureSession<H> {
    pub return_url: Option<Url>,
    client: H,
    verifier_url: Url,
    device_key: SessionKey,
    proposed_documents: Vec<ProposedDocument>,
    pub reader_registration: ReaderRegistration,
}

/// This type is derived from an [`Mdoc`] and will be used to construct a [`Document`]
/// for disclosure. Note that this is for internal use of [`DisclosureSession`] only.
#[allow(dead_code)]
#[derive(Debug)]
struct ProposedDocument {
    private_key_id: String,
    doc_type: DocType,
    issuer_signed: IssuerSigned,
    device_signed_challenge: Vec<u8>,
}

impl ProposedDocument {
    /// For a given set of `Mdoc`s with the same `doc_type`, return two `Vec`s:
    /// * A `Vec<ProposedDocument>` that contains all of the proposed
    ///   disclosure documents that provide all of the required attributes.
    /// * A `Vec<Vec<AttributeIdentifier>>` that contain the missing
    ///   attributes for every `Mdoc` that has at least one attribute missing.
    ///
    /// This means that the sum of the length of these `Vec`s is equal to the
    /// length of the input `Vec<Mdoc>`.
    fn candidates_and_missing_attributes_from_mdocs(
        mdocs: Vec<Mdoc>,
        requested_attributes: &IndexSet<AttributeIdentifier>,
        device_signed_challenge: Vec<u8>,
    ) -> Result<(Vec<Self>, Vec<Vec<AttributeIdentifier>>)> {
        let mut all_missing_attributes = Vec::new();

        // Collect all `ProposedDocument`s for this `doc_type`,
        // for every `Mdoc` that satisfies the requested attributes.
        let proposed_documents = mdocs
            .into_iter()
            .filter(|mdoc| {
                // Calculate missing attributes for every `Mdoc` and filter it out
                // if we find any. Also, collect the missing attributes separately.
                let available_attributes = mdoc.issuer_signed.attribute_identifiers(&mdoc.doc_type);
                let missing_attributes = requested_attributes
                    .difference(&available_attributes)
                    .collect::<Vec<_>>();

                let is_satisfying = missing_attributes.is_empty();

                if !is_satisfying {
                    all_missing_attributes.push(missing_attributes.into_iter().cloned().collect());
                }

                is_satisfying
            })
            // Convert the matching `Mdoc` to a `ProposedDocument`, based on the requested attributes.
            .map(|mdoc| ProposedDocument::from_mdoc(mdoc, requested_attributes, device_signed_challenge.clone()))
            .collect::<Vec<_>>();

        Ok((proposed_documents, all_missing_attributes))
    }

    /// Create a [`ProposedDocument`] from an [`Mdoc`], containing only those
    /// attributes that are requested and a [`DeviceSigned`] challenge.
    fn from_mdoc(
        mdoc: Mdoc,
        requested_attributes: &IndexSet<AttributeIdentifier>,
        device_signed_challenge: Vec<u8>,
    ) -> Self {
        let name_spaces = mdoc.issuer_signed.name_spaces.map(|name_spaces| {
            name_spaces
                .into_iter()
                .flat_map(|(name_space, attributes)| {
                    let attributes = attributes
                        .0
                        .into_iter()
                        .filter(|attribute| {
                            let attribute_identifier = AttributeIdentifier {
                                doc_type: mdoc.doc_type.clone(),
                                namespace: name_space.clone(),
                                attribute: attribute.0.element_identifier.clone(),
                            };

                            requested_attributes.contains(&attribute_identifier)
                        })
                        .collect::<Vec<_>>();

                    if attributes.is_empty() {
                        return None;
                    }

                    (name_space, attributes.into()).into()
                })
                .collect()
        });

        // Construct everything necessary for signing when the user approves the disclosure.
        let issuer_signed = IssuerSigned {
            name_spaces,
            issuer_auth: mdoc.issuer_signed.issuer_auth,
        };

        ProposedDocument {
            private_key_id: mdoc.private_key_id,
            doc_type: mdoc.doc_type,
            issuer_signed,
            device_signed_challenge,
        }
    }

    /// Return the attributes contained within this [`ProposedDocument`].
    fn name_spaces(&self) -> IndexMap<NameSpace, Vec<Entry>> {
        self.issuer_signed
            .name_spaces
            .as_ref()
            .map(|name_spaces| {
                name_spaces
                    .iter()
                    .map(|(name_space, attributes)| (name_space.clone(), attributes.into()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl<H> DisclosureSession<H>
where
    H: HttpClient,
{
    pub async fn start<'a>(
        client: H,
        reader_engagement_bytes: &[u8],
        return_url: Option<Url>,
        session_type: SessionType,
        mdoc_data_source: &impl MdocDataSource,
        trust_anchors: &[TrustAnchor<'a>],
    ) -> Result<Self> {
        // Deserialize the `ReaderEngagement` from the received bytes.
        let reader_engagement: ReaderEngagement = cbor_deserialize(reader_engagement_bytes)?;

        // Extract the verifier URL, return an error if it is is missing.
        let verifier_url = reader_engagement.verifier_url()?;

        // Create a new `DeviceEngagement` message and private key. Use a
        // static referrer URL, as this is not a feature we actually use.
        let (device_engagement, ephemeral_privkey) =
            DeviceEngagement::new_device_engagement(Url::parse(REFERRER_URL).unwrap())?;

        // Derive the session transcript and keys in both directions from the
        // `ReaderEngagement`, the `DeviceEngagement` and the ephemeral private key.
        let (transcript, reader_key, device_key) = reader_engagement.transcript_and_keys_for_device_engagement(
            session_type,
            &device_engagement,
            ephemeral_privkey,
        )?;

        // Send the `DeviceEngagement` to the verifier and decrypt the returned `DeviceRequest`.
        let session_data: SessionData = client.post(verifier_url, &device_engagement).await?;
        let device_request: DeviceRequest = session_data.decrypt_and_deserialize(&reader_key)?;

        // A device request without `DocumentRequest` entries is useless, so return an error.
        if device_request.doc_requests.is_empty() {
            return Err(HolderError::NoDocumentRequests.into());
        }

        // Verify reader authentication and decode `ReaderRegistration` from it at the same time.
        // Reader authentication is required to be present at this time.
        let reader_registration = device_request
            .verify(&transcript, &TimeGenerator, trust_anchors)?
            .ok_or(HolderError::ReaderAuthMissing)?;

        // Fetch documents from the database, calculate which ones satisfy the request and
        // formulate proposals for those documents. If there is a mismatch, return an error.
        let candidates_by_doc_type = match device_request
            .match_stored_documents(mdoc_data_source, &transcript)
            .await?
        {
            DeviceRequestMatch::Candidates(candidates) => candidates,
            DeviceRequestMatch::MissingAttributes(missing_attributes) => {
                // Attributes are missing, turn the `missing_attributes`
                // into an error along with the `ReaderRegistration`.
                let error = HolderError::AttributesNotAvailable {
                    reader_registration: reader_registration.into(),
                    missing_attributes,
                };

                return Err(error.into());
            }
        };

        // If we have multiple candidates for any of the doc types, return an error.
        // TODO: Support having the user a choose between multiple candidates.
        if candidates_by_doc_type.values().any(|candidates| candidates.len() > 1) {
            let duplicate_doc_types = candidates_by_doc_type
                .into_iter()
                .filter(|(_, candidates)| candidates.len() > 1)
                .map(|(doc_type, _)| doc_type)
                .collect();

            return Err(HolderError::MultipleCandidates(duplicate_doc_types).into());
        }

        // Now that we know that we have exactly one candidate for every `doc_type`,
        // we can flatten these candidates to a 1-dimensional `Vec`.
        let proposed_documents = candidates_by_doc_type.into_values().flatten().collect();

        // Retain all the necessary information to either abort or finish the disclosure session later.
        let session = DisclosureSession {
            client,
            return_url,
            verifier_url: verifier_url.clone(),
            device_key,
            proposed_documents,
            reader_registration,
        };

        Ok(session)
    }

    pub fn proposed_attributes(&self) -> ProposedAttributes {
        // Get all of the attributes to be disclosed from the
        // prepared `IssuerSigned` on the `ProposedDocument`s.
        self.proposed_documents
            .iter()
            .map(|document| (document.doc_type.clone(), document.name_spaces()))
            .collect()
    }

    // TODO: Implement terminate and disclose methods.
}

impl<H: HttpClient> Wallet<H> {
    pub async fn disclose<'a, K: MdocEcdsaKey + Sync>(
        &self,
        device_request: &DeviceRequest,
        session_transcript: &SessionTranscript,
        key_factory: &'a impl KeyFactory<'a, Key = K>,
        mdoc_retriever: &impl MdocRetriever,
    ) -> Result<DeviceResponse> {
        let docs: Vec<Document> = try_join_all(device_request.doc_requests.iter().map(|doc_request| {
            self.disclose_document::<K>(doc_request, session_transcript, key_factory, mdoc_retriever)
        }))
        .await?;

        let response = DeviceResponse {
            version: DeviceResponseVersion::V1_0,
            documents: Some(docs),
            document_errors: None, // TODO: consider using this for reporting errors per document/mdoc
            status: 0,
        };
        Ok(response)
    }

    async fn disclose_document<'a, K: MdocEcdsaKey + Sync>(
        &self,
        doc_request: &DocRequest,
        session_transcript: &SessionTranscript,
        key_factory: &'a impl KeyFactory<'a, Key = K>,
        mdoc_retriever: &impl MdocRetriever,
    ) -> Result<Document> {
        let items_request = &doc_request.items_request.0;

        // This takes any mdoc of the specified doctype. TODO: allow user choice.
        let creds =
            mdoc_retriever
                .get(&items_request.doc_type)
                .ok_or(Error::from(HolderError::UnsatisfiableRequest(
                    items_request.doc_type.clone(),
                )))?;
        let cred = &creds
            .first()
            .ok_or(Error::from(HolderError::UnsatisfiableRequest(
                items_request.doc_type.clone(),
            )))?
            .cred_copies[0];
        let document = cred
            .disclose_document(items_request, session_transcript, key_factory)
            .await?;
        Ok(document)
    }
}

impl Mdoc {
    pub async fn disclose_document<'a, K: MdocEcdsaKey + Sync>(
        &self,
        items_request: &ItemsRequest,
        session_transcript: &SessionTranscript,
        key_factory: &'a impl KeyFactory<'a, Key = K>,
    ) -> Result<Document> {
        let disclosed_namespaces: IssuerNameSpaces = self
            .issuer_signed
            .name_spaces
            .as_ref()
            .unwrap()
            .iter()
            .filter(|&(namespace, _)| items_request.name_spaces.contains_key(namespace))
            .map(|(namespace, attributes)| {
                (
                    namespace.clone(),
                    attributes.filter(items_request.name_spaces.get(namespace).unwrap()),
                )
            })
            .collect();

        let doc = Document {
            doc_type: items_request.doc_type.clone(),
            issuer_signed: IssuerSigned {
                name_spaces: Some(disclosed_namespaces),
                issuer_auth: self.issuer_signed.issuer_auth.clone(),
            },
            device_signed: DeviceSigned::new_signature(
                &key_factory.generate_existing(&self.private_key_id, self.public_key()?),
                &cbor_serialize(&TaggedBytes(CborSeq(DeviceAuthenticationKeyed {
                    device_authentication: Default::default(),
                    session_transcript: session_transcript.clone(),
                    doc_type: self.doc_type.clone(),
                    device_name_spaces_bytes: TaggedBytes(IndexMap::new()),
                })))?,
            )
            .await,
            errors: None,
        };
        Ok(doc)
    }
}

impl DeviceSigned {
    pub async fn new_signature(private_key: &(impl SecureEcdsaKey + Sync), challenge: &[u8]) -> DeviceSigned {
        let cose = sign_cose(challenge, Header::default(), private_key, false).await;

        DeviceSigned {
            name_spaces: IndexMap::new().into(),
            device_auth: DeviceAuth::DeviceSignature(cose.into()),
        }
    }

    #[allow(dead_code)] // TODO test this
    pub fn new_mac(
        private_key: &SecretKey,
        reader_pub_key: &PublicKey,
        session_transcript: &SessionTranscript,
        device_auth: &DeviceAuthenticationBytes,
    ) -> Result<DeviceSigned> {
        let key = dh_hmac_key(
            private_key,
            reader_pub_key,
            &cbor_serialize(&TaggedBytes(session_transcript))?,
            "EMacKey",
            32,
        )?;

        let cose = CoseMac0Builder::new()
            .payload(cbor_serialize(device_auth)?)
            .protected(HeaderBuilder::new().algorithm(iana::Algorithm::ES256).build())
            .create_tag(&[], |data| ring::hmac::sign(&key, data).as_ref().into())
            .build()
            .clone_without_payload();

        let device_signed = DeviceSigned {
            name_spaces: IndexMap::new().into(),
            device_auth: DeviceAuth::DeviceMac(cose.into()),
        };
        Ok(device_signed)
    }
}

enum DeviceRequestMatch {
    Candidates(HashMap<DocType, Vec<ProposedDocument>>),
    MissingAttributes(Vec<AttributeIdentifier>), // TODO: Report on missing attributes per `Mdoc` candidate.
}

impl DeviceRequest {
    /// Verify reader authentication, if present.
    /// Note that since each DocRequest carries its own reader authentication, the spec allows the
    /// the DocRequests to be signed by distinct readers. TODO maybe support this.
    /// For now, this function requires either none of the DocRequests to be signed, or all of them
    /// by the same reader.
    pub fn verify(
        &self,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Option<ReaderRegistration>> {
        // If there are no doc requests or none of them have reader authentication, return `None`.
        if self.doc_requests.iter().all(|d| d.reader_auth.is_none()) {
            return Ok(None);
        }

        // Otherwise, all of the doc requests need reader authentication.
        if self.doc_requests.iter().any(|d| d.reader_auth.is_none()) {
            return Err(HolderError::ReaderAuthMissing.into());
        }

        // Verify all `DocRequest` entries and make sure the resulting certificates are all exactly equal.
        // Note that the unwraps are safe, since we checked for the presence of reader authentication above.
        let certificate = self
            .doc_requests
            .iter()
            .try_fold(None, {
                |result_cert, doc_request| -> Result<_> {
                    let doc_request_cert = doc_request.verify(session_transcript, time, trust_anchors)?.unwrap();

                    // If there is a certificate from a previous iteration, compare our certificate to that.
                    if let Some(result_cert) = result_cert {
                        if doc_request_cert != result_cert {
                            return Err(HolderError::ReaderAuthsInconsistent.into());
                        }
                    }

                    Ok(doc_request_cert.into())
                }
            })?
            .unwrap();

        // Extract `ReaderRegistration` from the one certificate.
        let reader_registration = match CertificateType::from_certificate(&certificate).map_err(HolderError::from)? {
            Some(CertificateType::ReaderAuth(reader_registration)) => *reader_registration,
            _ => return Err(HolderError::NoReaderRegistration(certificate).into()),
        };

        // Verify that the requested attributes are included in the reader authentication.
        self.verify_requested_attributes(&reader_registration)
            .map_err(HolderError::from)?;

        Ok(reader_registration.into())
    }

    async fn match_stored_documents(
        &self,
        mdoc_data_source: &impl MdocDataSource,
        session_transcript: &SessionTranscript,
    ) -> Result<DeviceRequestMatch> {
        // Make a `HashSet` of doc types from the `DeviceRequest` to account
        // for potential duplicate doc types in the request, then fetch them
        // from our data source.
        let doc_types = self
            .doc_requests
            .iter()
            .map(|doc_request| doc_request.items_request.0.doc_type.as_str())
            .collect();

        let mdocs = mdoc_data_source
            .mdoc_by_doc_types(&doc_types)
            .await
            .map_err(|error| HolderError::MdocDataSource(error.into()))?;

        // For each `doc_type`, calculate the set of `AttributeIdentifier`s that
        // are needed to satisfy the request. Note that a `doc_type` may occur more
        // than once in a `DeviceRequest`, so we combine all attributes and then split
        // them out by `doc_type`.
        let mut requested_attributes_by_doc_type = self.attribute_identifiers().into_iter().fold(
            HashMap::<_, IndexSet<_>>::with_capacity(doc_types.len()),
            |mut requested_attributes, attribute_identifier| {
                // This unwrap is safe, as `doc_types` is derived from the same `DeviceRequest`.
                let doc_type = *doc_types.get(attribute_identifier.doc_type.as_str()).unwrap();
                requested_attributes
                    .entry(doc_type)
                    .or_default()
                    .insert(attribute_identifier);

                requested_attributes
            },
        );

        // Each `Vec<Mdoc>` that is returned from storage should contain `Mdoc`s
        // that have the same `doc_type`. Below, we iterate over all of these
        // `Vec`s and perform the following steps:
        //
        // * Filter out any empty `Vec<Mdoc>`.
        // * Get the `doc_type` from the first `Mdoc` entry.
        // * Remove the value for this `doc_type` from `requested_attributes_by_doc_type`.
        // * Do some sanity checks, as the request should actually contain this `doc_type`
        //   and any subsequent `Mdoc`s should have the same `doc_type`. This is part of
        //   the contract of `MdocDataSource` that is not enforceable.
        // * Calculate the challenge needed to create the `DeviceSigned` for this
        //   `doc_type` later on during actual disclosure.
        // * Convert all `Mdoc`s that satisfy the requirement to `ProposedDocument`,
        //   while collecting any missing attributes separately.
        // * Collect the candidates in a `HashMap` per `doc_type`.
        //
        // Note that we consume the requested attributes from
        // `requested_attributes_by_doc_type` for the following reasons:
        //
        // * A `doc_type` should not occur more than once in the top-level
        //  `Vec` returned by `MdocDataSource`.
        // * After gathering all the candidates, any requested attributes that
        //   still remain in `requested_attributes_by_doc_type` are not satisfied,
        //   which means that all of them count as missing attributes.
        let mut all_missing_attributes = Vec::<Vec<AttributeIdentifier>>::new();

        let candidates_by_doc_type = mdocs
            .into_iter()
            .filter(|doc_type_mdocs| !doc_type_mdocs.is_empty())
            .map(|doc_type_mdocs| {
                // First, remove the `IndexSet` of attributes that are required for this
                // `doc_type` from the global `HashSet`. If this cannot be found, then
                // `MdocDataSource` did not obey the contract as noted in the comment above.
                let first_doc_type = doc_type_mdocs.first().unwrap().doc_type.as_str();
                let (doc_type, requested_attributes) = requested_attributes_by_doc_type
                    .remove_entry(first_doc_type)
                    .expect("Received mdoc candidate with unexpected doc_type from storage");

                // Do another sanity check, all of the remaining `Mdoc`s
                // in the `Vec` should have the same `doc_type`.
                for mdoc in &doc_type_mdocs {
                    if mdoc.doc_type != doc_type {
                        panic!("Received mdoc candidate with inconsistent doc_type from storage");
                    }
                }

                // Calculate the `DeviceAuthentication` for this `doc_type` and turn it into bytes,
                // so that it can be used as a challenge when constructing `DeviceSigned` later on.
                let device_authentication =
                    DeviceAuthentication::from_session_transcript(session_transcript.clone(), doc_type.to_string());
                let device_signed_challenge = cbor_serialize(&TaggedBytes(device_authentication))?;

                // Get all the candidates and missing attributes from the provided `Mdoc`s.
                let (candidates, missing_attributes) = ProposedDocument::candidates_and_missing_attributes_from_mdocs(
                    doc_type_mdocs,
                    &requested_attributes,
                    device_signed_challenge,
                )?;

                // If we have multiple `Mdoc`s with missing attributes, just record the first one.
                // TODO: Report on missing attributes for multiple `Mdoc` candidates.
                if let Some(missing_attributes) = missing_attributes.into_iter().next() {
                    all_missing_attributes.push(missing_attributes);
                }

                Ok((doc_type.to_string(), candidates))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        // If we cannot find a suitable candidate for any of the doc types
        // or one of the doc types is missing entirely, collect all of the
        // attributes that are missing and return this as the
        // `DeviceRequestMatch::MissingAttributes` invariant.
        if candidates_by_doc_type.values().any(|candidates| candidates.is_empty())
            || !requested_attributes_by_doc_type.is_empty()
        {
            // Combine the missing attributes from the processed `Mdoc`s with
            // the requested attributes for any `doc_type` we did not see at all.
            let missing_attributes = all_missing_attributes
                .into_iter()
                .flatten()
                .chain(requested_attributes_by_doc_type.into_values().flatten())
                .collect();

            return Ok(DeviceRequestMatch::MissingAttributes(missing_attributes));
        }

        // Each `doc_type` has at least one candidates, return these now.
        Ok(DeviceRequestMatch::Candidates(candidates_by_doc_type))
    }
}

impl ReaderEngagement {
    /// Get the URL for the HTTPS endpoint of the verifier.
    fn verifier_url(&self) -> Result<&Url> {
        let verifier_url = self
            .0
            .connection_methods
            .as_ref()
            .and_then(|methods| methods.first())
            .map(|method| &method.0.connection_options.0.uri)
            .ok_or(HolderError::VerifierUrlMissing)?;

        Ok(verifier_url)
    }

    /// Get the public key of the verifier.
    fn verifier_public_key(&self) -> Result<PublicKey> {
        let verifier_public_key = self
            .0
            .security
            .as_ref()
            .ok_or(HolderError::VerifierEphemeralKeyMissing)?
            .try_into()?;

        Ok(verifier_public_key)
    }

    /// Calculate the [`SessionTranscript`], the [`SessionKey`] for the reader
    /// (for decrypting the [`DeviceRequest`]) and the [`SessionKey`] for the
    /// device (for encrypting the [`DeviceResponse`]).
    fn transcript_and_keys_for_device_engagement(
        &self,
        session_type: SessionType,
        device_engagement: &DeviceEngagement,
        device_private_key: SecretKey,
    ) -> Result<(SessionTranscript, SessionKey, SessionKey)> {
        let verifier_public_key = self.verifier_public_key()?;

        // Create the session transcript so far based on both engagement payloads.
        let session_transcript = SessionTranscript::new(session_type, self, device_engagement)
            .map_err(|_| HolderError::VerifierEphemeralKeyMissing)?;

        // Derive the session key for both directions from the private and public keys and the session transcript.
        let reader_key = SessionKey::new(
            &device_private_key,
            &verifier_public_key,
            &session_transcript,
            SessionKeyUser::Reader,
        )?;
        let device_key = SessionKey::new(
            &device_private_key,
            &verifier_public_key,
            &session_transcript,
            SessionKeyUser::Device,
        )?;

        Ok((session_transcript, reader_key, device_key))
    }
}

impl DocRequest {
    pub fn verify(
        &self,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Option<Certificate>> {
        // If reader authentication is present, verify it and return the certificate.
        self.reader_auth
            .as_ref()
            .map(|reader_auth| {
                // Reconstruct the reader authentication bytes for this `DocRequest`,
                // based on the item requests and session transcript.
                let reader_auth_payload = ReaderAuthenticationKeyed {
                    reader_auth_string: Default::default(),
                    session_transcript: session_transcript.clone(),
                    items_request_bytes: self.items_request.clone(),
                };
                let reader_auth_payload = TaggedBytes(CborSeq(reader_auth_payload));

                // Perform verification and return the `Certificate`.
                let cose = reader_auth.clone_with_payload(cbor_serialize(&reader_auth_payload)?);
                cose.verify_against_trust_anchors(CertificateUsage::ReaderAuth, time, trust_anchors)?;
                let cert = cose.signing_cert()?;

                Ok(cert)
            })
            .transpose()
    }
}

impl Attributes {
    /// Return a copy that contains only the items requested in `items_request`.
    fn filter(&self, requested: &DataElements) -> Attributes {
        self.0
            .clone()
            .into_iter()
            .filter(|attr| requested.contains_key(&attr.0.element_identifier))
            .collect::<Vec<_>>()
            .into()
    }
}

impl DeviceEngagement {
    pub fn new_device_engagement(referrer_url: Url) -> Result<(DeviceEngagement, SecretKey)> {
        let privkey = SecretKey::random(&mut OsRng);

        let engagement = Engagement {
            version: EngagementVersion::V1_0,
            security: Some((&privkey.public_key()).try_into()?),
            connection_methods: None,
            origin_infos: vec![
                OriginInfo {
                    cat: OriginInfoDirection::Received,
                    typ: OriginInfoType::Website(referrer_url),
                },
                OriginInfo {
                    cat: OriginInfoDirection::Delivered,
                    typ: OriginInfoType::MessageData,
                },
            ],
        };

        Ok((engagement.into(), privkey))
    }
}

impl DeviceAuthentication {
    /// Re-construct a [`DeviceAuthentication`] from a [`SessionTranscript`] and [`DocType`].
    pub fn from_session_transcript(session_transcript: SessionTranscript, doc_type: DocType) -> Self {
        DeviceAuthenticationKeyed {
            device_authentication: Default::default(),
            session_transcript,
            doc_type,
            device_name_spaces_bytes: TaggedBytes(IndexMap::new()),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use std::{convert::Infallible, fmt};

    use assert_matches::assert_matches;
    use serde::{de::DeserializeOwned, Serialize};

    use crate::{
        examples::{Example, Examples},
        mock,
        server_keys::PrivateKey,
        utils::{
            cose::{self, MdocCose},
            reader_auth::{AuthorizedAttribute, AuthorizedMdoc, AuthorizedNamespace},
            x509::OwnedTrustAnchor,
        },
    };

    use super::*;

    // Constants for testing.
    const RP_CA_CN: &str = "ca.rp.example.com";
    const RP_CERT_CN: &str = "cert.rp.example.com";
    const SESSION_URL: &str = "http://example.com/disclosure";
    const RETURN_URL: &str = "http://example.com/return";

    // Describe what is in `DeviceResponse::example()`.
    const EXAMPLE_DOC_TYPE: &str = "org.iso.18013.5.1.mDL";
    const EXAMPLE_NAMESPACE: &str = "org.iso.18013.5.1";
    const EXAMPLE_ATTRIBUTES: [&str; 5] = [
        "family_name",
        "issue_date",
        "expiry_date",
        "document_number",
        "driving_privileges",
    ];

    /// Build an [`ItemsRequest`] from a list of attributes.
    fn items_request(
        doc_type: String,
        name_space: String,
        attributes: impl Iterator<Item = impl Into<String>>,
    ) -> ItemsRequest {
        ItemsRequest {
            doc_type,
            name_spaces: IndexMap::from_iter([(
                name_space,
                attributes.map(|attribute| (attribute.into(), false)).collect(),
            )]),
            request_info: None,
        }
    }

    /// Build attributes for [`ReaderRegistration`] from a list of attributes.
    fn reader_registration_attributes(
        doc_type: String,
        name_space: String,
        attributes: impl Iterator<Item = impl Into<String>>,
    ) -> IndexMap<String, AuthorizedMdoc> {
        [(
            doc_type,
            AuthorizedMdoc(
                [(
                    name_space,
                    AuthorizedNamespace(
                        attributes
                            .map(|attribute| (attribute.into(), AuthorizedAttribute {}))
                            .collect(),
                    ),
                )]
                .into(),
            ),
        )]
        .into()
    }

    /// A type that implements `MdocDataSource` and simply returns
    /// the [`Mdoc`] contained in `DeviceResponse::example()`, if its
    /// `doc_type` is requested.
    #[derive(Debug, Default)]
    struct MockMdocDataSource {}

    #[async_trait]
    impl MdocDataSource for MockMdocDataSource {
        type Error = Infallible;

        async fn mdoc_by_doc_types(
            &self,
            doc_types: &HashSet<&str>,
        ) -> std::result::Result<Vec<Vec<Mdoc>>, Self::Error> {
            if doc_types.contains(EXAMPLE_DOC_TYPE) {
                let trust_anchors = Examples::iaca_trust_anchors();
                let mdoc = mock::mdoc_from_example_device_response(trust_anchors);

                return Ok(vec![vec![mdoc]]);
            }

            Ok(Default::default())
        }
    }

    /// This type contains the minimum logic to respond with the correct
    /// verifier messages in a disclosure session. Currently it only responds
    /// with a [`SessionData`] containing a [`DeviceRequest`].
    struct MockVerifierSession {
        session_type: SessionType,
        trust_anchors: Vec<OwnedTrustAnchor>,
        private_key: PrivateKey,
        reader_engagement: ReaderEngagement,
        reader_ephemeral_key: SecretKey,
    }

    impl fmt::Debug for MockVerifierSession {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MockVerifierSession")
                .field("session_type", &self.session_type)
                .field("trust_anchors", &self.trust_anchors)
                .field("reader_engagement", &self.reader_engagement)
                .finish_non_exhaustive()
        }
    }

    impl MockVerifierSession {
        fn new(session_type: SessionType, session_url: Url, reader_registration: ReaderRegistration) -> Self {
            // Generate trust anchors, signing key and certificate containing `ReaderRegistration`.
            let (ca, ca_privkey) = Certificate::new_ca(RP_CA_CN).unwrap();
            let trust_anchors = vec![OwnedTrustAnchor::try_from(ca.as_bytes()).unwrap()];
            let (rp_certificate, rp_signing_key) = Certificate::new(
                &ca,
                &ca_privkey,
                RP_CERT_CN,
                CertificateType::ReaderAuth(reader_registration.into()),
            )
            .unwrap();
            let private_key = PrivateKey::new(rp_signing_key, rp_certificate);

            // Generate the `ReaderEngagement` that would be be sent in the UL.
            let (reader_engagement, reader_ephemeral_key) =
                ReaderEngagement::new_reader_engagement(session_url).unwrap();

            MockVerifierSession {
                session_type,
                trust_anchors,
                private_key,
                reader_engagement,
                reader_ephemeral_key,
            }
        }

        fn client(&self) -> MockVerifierSessionClient {
            MockVerifierSessionClient { session: self }
        }

        fn reader_engagement_bytes(&self) -> Vec<u8> {
            cbor_serialize(&self.reader_engagement).unwrap()
        }

        fn trust_anchors(&self) -> Vec<TrustAnchor> {
            self.trust_anchors
                .iter()
                .map(|anchor| anchor.into())
                .collect::<Vec<_>>()
        }

        // Generate the `SessionData` response containing the `DeviceRequest`,
        // based on the `DeviceEngagement` received from the device.
        async fn device_request_session_data(&self, device_engagement: DeviceEngagement) -> SessionData {
            // Create the session transcript and encryption key.
            let session_transcript =
                SessionTranscript::new(self.session_type, &self.reader_engagement, &device_engagement).unwrap();

            let device_public_key = device_engagement.0.security.as_ref().unwrap().try_into().unwrap();

            let reader_key = SessionKey::new(
                &self.reader_ephemeral_key,
                &device_public_key,
                &session_transcript,
                SessionKeyUser::Reader,
            )
            .unwrap();

            // Generate the example `ItemRequest`.
            let items_request = items_request(
                EXAMPLE_DOC_TYPE.to_string(),
                EXAMPLE_NAMESPACE.to_string(),
                EXAMPLE_ATTRIBUTES.iter().copied(),
            );

            // Generate the reader authentication signature, without payload.
            let reader_auth = ReaderAuthenticationKeyed {
                reader_auth_string: Default::default(),
                session_transcript,
                items_request_bytes: items_request.clone().into(),
            };

            let cose = MdocCose::<_, ReaderAuthenticationBytes>::sign(
                &TaggedBytes(CborSeq(reader_auth)),
                cose::new_certificate_header(&self.private_key.cert_bts),
                &self.private_key,
                false,
            )
            .await
            .unwrap();

            // Create and encrypt the `DeviceRequest`.
            let doc_request = DocRequest {
                items_request: items_request.into(),
                reader_auth: Some(cose.0.into()),
            };

            let device_request = DeviceRequest {
                version: DeviceRequestVersion::V1_0,
                doc_requests: vec![doc_request],
            };

            SessionData::serialize_and_encrypt(&device_request, &reader_key).unwrap()
        }
    }

    /// This type implements [`HttpClient`] and simply forwards the
    /// requests to an instance of [`MockVerifierSession`].
    #[derive(Debug)]
    struct MockVerifierSessionClient<'a> {
        session: &'a MockVerifierSession,
    }

    #[async_trait]
    impl HttpClient for MockVerifierSessionClient<'_> {
        async fn post<R, V>(&self, url: &Url, val: &V) -> Result<R>
        where
            V: Serialize + Sync,
            R: DeserializeOwned,
        {
            // The URL has to match the one on the configured `ReaderEngagement`.
            assert_eq!(url, self.session.reader_engagement.verifier_url().unwrap());

            // Serialize and deserialize both the request and response
            // in order to adhere to the trait bounds.
            let device_engagement = cbor_deserialize(cbor_serialize(val).unwrap().as_slice()).unwrap();
            let session_data = self.session.device_request_session_data(device_engagement).await;
            let result = cbor_deserialize(cbor_serialize(&session_data).unwrap().as_slice()).unwrap();

            Ok(result)
        }
    }

    #[tokio::test]
    async fn test_disclosure_session_start() {
        // Create a reader registration with all of the example attributes.
        let reader_registration = ReaderRegistration {
            attributes: reader_registration_attributes(
                EXAMPLE_DOC_TYPE.to_string(),
                EXAMPLE_NAMESPACE.to_string(),
                EXAMPLE_ATTRIBUTES.iter().copied(),
            ),
            ..Default::default()
        };

        // Create a mock session, client and mdoc data source.
        let verifier_session = MockVerifierSession::new(
            SessionType::SameDevice,
            SESSION_URL.parse().unwrap(),
            reader_registration.clone(),
        );
        let mdoc_data_source = MockMdocDataSource::default();

        let return_url = Url::parse(RETURN_URL).unwrap();

        // Starting a disclosure session should now succeed.
        let session = DisclosureSession::start(
            verifier_session.client(),
            &verifier_session.reader_engagement_bytes(),
            return_url.clone().into(),
            verifier_session.session_type,
            &mdoc_data_source,
            &verifier_session.trust_anchors(),
        )
        .await
        .expect("Could not start disclosure session");

        // Test if the return `Url` and `ReaderRegistration` match the input.
        assert_eq!(session.return_url.as_ref().unwrap(), &return_url);
        assert_eq!(session.reader_registration, reader_registration);

        // Test that the proposal for disclosure contains the example attributes, in order.
        let entry_keys = session
            .proposed_attributes()
            .remove(EXAMPLE_DOC_TYPE)
            .and_then(|mut name_space| name_space.remove(EXAMPLE_NAMESPACE))
            .map(|entries| entries.into_iter().map(|entry| entry.name).collect::<Vec<_>>())
            .unwrap_or_default();

        assert_eq!(entry_keys, EXAMPLE_ATTRIBUTES);
    }

    #[tokio::test]
    async fn test_disclosure_session_start_error_decode_reader_engagement() {
        let verifier_session = MockVerifierSession::new(
            SessionType::SameDevice,
            SESSION_URL.parse().unwrap(),
            Default::default(),
        );
        let mdoc_data_source = MockMdocDataSource::default();

        // Starting a `DisclosureSession` with invalid `ReaderEngagement`
        // bytes should result in a `Error::Cbor` error.
        let error = DisclosureSession::start(
            verifier_session.client(),
            &[],
            None,
            verifier_session.session_type,
            &mdoc_data_source,
            &verifier_session.trust_anchors(),
        )
        .await
        .expect_err("Starting disclosure session should have resulted in an error");

        assert_matches!(error, Error::Cbor(_));
    }

    #[test]
    fn test_device_authentication_bytes_from_session_transcript() {
        let session_transcript = DeviceAuthenticationBytes::example().0 .0.session_transcript;
        println!("{:?}", session_transcript);
        let device_authentication =
            DeviceAuthentication::from_session_transcript(session_transcript, EXAMPLE_DOC_TYPE.to_string());

        assert_eq!(
            cbor_serialize(&TaggedBytes(device_authentication)).unwrap(),
            DeviceAuthenticationBytes::example_bts()
        );
    }
}
