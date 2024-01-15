use url::Url;

use wallet::{
    errors::DisclosureError, mdoc::ReaderRegistration, DisclosureDocument, DisclosureProposal,
    MissingDisclosureAttributes,
};

use super::{
    card::{into_card_attributes, CardAttribute, LocalizedString},
    instruction::WalletInstructionError,
};

#[derive(Clone)]
pub enum Image {
    Svg { xml: String },
    Png { base64: String },
    Jpg { base64: String },
    Asset { path: String },
}

#[derive(Clone)]
pub struct Organization {
    pub legal_name: Vec<LocalizedString>,
    pub display_name: Vec<LocalizedString>,
    pub description: Vec<LocalizedString>,
    pub image: Option<Image>,
    pub web_url: Option<String>,
    pub kvk: Option<String>,
    pub city: Option<Vec<LocalizedString>>,
    pub category: Vec<LocalizedString>,
    pub department: Option<Vec<LocalizedString>>,
    pub country_code: Option<String>,
}

pub struct RequestPolicy {
    pub data_storage_duration_in_minutes: Option<u64>,
    pub data_shared_with_third_parties: bool,
    pub data_deletion_possible: bool,
    pub policy_url: String,
}

pub struct MissingAttribute {
    pub labels: Vec<LocalizedString>,
}

pub struct RequestedCard {
    pub doc_type: String,
    pub attributes: Vec<CardAttribute>,
}

pub enum StartDisclosureResult {
    Request {
        relying_party: Organization,
        policy: RequestPolicy,
        requested_cards: Vec<RequestedCard>,
        is_first_interaction_with_relying_party: bool,
        request_purpose: Vec<LocalizedString>,
    },
    RequestAttributesMissing {
        relying_party: Organization,
        missing_attributes: Vec<MissingAttribute>,
        is_first_interaction_with_relying_party: bool,
        request_purpose: Vec<LocalizedString>,
    },
}

pub enum AcceptDisclosureResult {
    Ok { return_url: Option<String> },
    InstructionError { error: WalletInstructionError },
}

pub struct RPLocalizedStrings(pub wallet::mdoc::LocalizedStrings);

impl From<RPLocalizedStrings> for Vec<LocalizedString> {
    fn from(value: RPLocalizedStrings) -> Self {
        let RPLocalizedStrings(wallet::mdoc::LocalizedStrings(localized_strings)) = value;
        localized_strings
            .iter()
            .map(|(language, value)| LocalizedString {
                language: language.to_owned(),
                value: value.to_owned(),
            })
            .collect()
    }
}

impl From<wallet::mdoc::Image> for Image {
    fn from(value: wallet::mdoc::Image) -> Self {
        match value.mime_type {
            wallet::mdoc::ImageType::Svg => Image::Svg { xml: value.image_data },
            wallet::mdoc::ImageType::Png => Image::Png {
                base64: value.image_data,
            },
            wallet::mdoc::ImageType::Jpeg => Image::Jpg {
                base64: value.image_data,
            },
        }
    }
}

impl From<wallet::mdoc::Organization> for Organization {
    fn from(value: wallet::mdoc::Organization) -> Self {
        Organization {
            legal_name: RPLocalizedStrings(value.legal_name).into(),
            display_name: RPLocalizedStrings(value.display_name).into(),
            description: RPLocalizedStrings(value.description).into(),
            image: value.logo.map(|logo| logo.into()),
            kvk: value.kvk,
            city: value.city.map(|city| RPLocalizedStrings(city).into()),
            category: RPLocalizedStrings(value.category).into(),
            department: value.department.map(|department| RPLocalizedStrings(department).into()),
            country_code: value.country_code,
            web_url: value.web_url.map(|url| url.to_string()),
        }
    }
}

impl From<&ReaderRegistration> for RequestPolicy {
    fn from(value: &ReaderRegistration) -> Self {
        RequestPolicy {
            data_storage_duration_in_minutes: value.retention_policy.max_duration_in_minutes,
            data_shared_with_third_parties: value.sharing_policy.intent_to_share,
            data_deletion_possible: value.deletion_policy.deleteable,
            policy_url: value
                .organization
                .privacy_policy_url
                .as_ref()
                .map(|url| url.to_string())
                .unwrap_or_default(),
        }
    }
}

impl RequestedCard {
    fn from_disclosure_documents(documents: Vec<DisclosureDocument>) -> Vec<Self> {
        documents.into_iter().map(RequestedCard::from).collect()
    }
}

impl From<DisclosureDocument> for RequestedCard {
    fn from(value: DisclosureDocument) -> Self {
        RequestedCard {
            doc_type: value.doc_type.to_string(),
            attributes: into_card_attributes(value.attributes),
        }
    }
}

impl MissingAttribute {
    fn from_missing_disclosure_attributes(attributes: Vec<MissingDisclosureAttributes>) -> Vec<Self> {
        attributes
            .into_iter()
            .flat_map(|doc_attributes| doc_attributes.attributes.into_iter())
            .map(|(_, labels)| {
                let labels = labels
                    .into_iter()
                    .map(|(language, value)| LocalizedString {
                        language: language.to_string(),
                        value: value.to_string(),
                    })
                    .collect::<Vec<_>>();

                MissingAttribute { labels }
            })
            .collect::<Vec<_>>()
    }
}

impl TryFrom<Result<DisclosureProposal, DisclosureError>> for StartDisclosureResult {
    type Error = DisclosureError;

    fn try_from(value: Result<DisclosureProposal, DisclosureError>) -> Result<Self, Self::Error> {
        match value {
            Ok(proposal) => {
                let policy: RequestPolicy = (&proposal.reader_registration).into();
                let request_purpose: Vec<LocalizedString> =
                    RPLocalizedStrings(proposal.reader_registration.purpose_statement).into();
                let result = StartDisclosureResult::Request {
                    relying_party: proposal.reader_registration.organization.into(),
                    policy,
                    requested_cards: RequestedCard::from_disclosure_documents(proposal.documents),
                    is_first_interaction_with_relying_party: false, //TODO: Resolve this value
                    request_purpose,
                };

                Ok(result)
            }
            Err(error) => match error {
                DisclosureError::AttributesNotAvailable {
                    reader_registration,
                    missing_attributes,
                } => {
                    let request_purpose: Vec<LocalizedString> =
                        RPLocalizedStrings(reader_registration.purpose_statement).into();
                    let missing_attributes = MissingAttribute::from_missing_disclosure_attributes(missing_attributes);
                    let result = StartDisclosureResult::RequestAttributesMissing {
                        relying_party: reader_registration.organization.into(),
                        missing_attributes,
                        is_first_interaction_with_relying_party: false, //TODO: Resolve this value
                        request_purpose,
                    };

                    Ok(result)
                }
                _ => Err(error),
            },
        }
    }
}

impl TryFrom<Result<Option<Url>, DisclosureError>> for AcceptDisclosureResult {
    type Error = DisclosureError;

    fn try_from(value: Result<Option<Url>, DisclosureError>) -> Result<Self, Self::Error> {
        match value {
            Ok(return_url) => Ok(AcceptDisclosureResult::Ok {
                return_url: return_url.map(|return_url| return_url.into()),
            }),
            Err(DisclosureError::Instruction(instruction_error)) => Ok(AcceptDisclosureResult::InstructionError {
                error: instruction_error.try_into().map_err(DisclosureError::Instruction)?,
            }),
            Err(error) => Err(error),
        }
    }
}
