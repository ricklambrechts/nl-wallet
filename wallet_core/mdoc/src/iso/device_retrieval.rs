//! Data structures with which a verifier requests attributes from a holder.

use std::{borrow::Cow, fmt::Debug};

use ciborium::value::Value;
use coset::CoseSign1;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use crate::{
    iso::{engagement::*, mdocs::*},
    utils::{
        cose::MdocCose,
        serialization::{CborSeq, ReaderAuthenticationString, RequiredValue, TaggedBytes},
    },
};

/// Sent by the RP to the holder to request the disclosure of attributes out of one or more mdocs.
/// For each mdoc out of which attributes are requested, a [`DocRequest`] is included.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRequest {
    pub version: DeviceRequestVersion,
    pub doc_requests: Vec<DocRequest>,
    pub return_url: Option<Url>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub enum DeviceRequestVersion {
    #[default]
    #[serde(rename = "1.0")]
    V1_0,
}

/// Requests attributes out of an mdoc of a specified doctype to be disclosed, as part of a [`DeviceRequest`].
/// Includes reader (RP) authentication.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocRequest {
    pub items_request: ItemsRequestBytes,
    pub reader_auth: Option<ReaderAuth>,
}

pub type ReaderAuth = MdocCose<CoseSign1, Value>;
pub type ReaderAuthenticationBytes<'a> = TaggedBytes<ReaderAuthentication<'a>>;
pub type ReaderAuthentication<'a> = CborSeq<ReaderAuthenticationKeyed<'a>>;

#[cfg_attr(any(test, feature = "examples"), derive(Deserialize))]
#[derive(Serialize, Debug, Clone)]
pub struct ReaderAuthenticationKeyed<'a> {
    pub reader_auth_string: RequiredValue<ReaderAuthenticationString>,
    pub session_transcript: Cow<'a, SessionTranscript>,
    pub items_request_bytes: Cow<'a, ItemsRequestBytes>,
}

impl<'a> ReaderAuthenticationKeyed<'a> {
    pub fn new(session_transcript: &'a SessionTranscript, items_request_bytes: &'a ItemsRequestBytes) -> Self {
        ReaderAuthenticationKeyed {
            reader_auth_string: Default::default(),
            session_transcript: Cow::Borrowed(session_transcript),
            items_request_bytes: Cow::Borrowed(items_request_bytes),
        }
    }
}

/// See [`ItemsRequest`].
pub type ItemsRequestBytes = TaggedBytes<ItemsRequest>;

/// Requests attributes out of an mdoc of a specified doctype to be disclosed, as part of a [`DocRequest`] in a
/// [`DeviceRequest`].
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemsRequest {
    pub doc_type: DocType,
    pub name_spaces: NameSpaces,

    /// Free-form additional information.
    pub request_info: Option<IndexMap<String, Value>>,
}

/// The attribute names that the RP wishes disclosed, grouped per namespace, as part of a [`ItemsRequest`].
pub type NameSpaces = IndexMap<NameSpace, DataElements>;

/// The attribute names that the RP wishes disclosed within a particular namespace, as part of a [`ItemsRequest`],
/// along with a boolean with which the RP can claim its intention to (not) retain the attribute value after receiving
/// and verifying it.
pub type DataElements = IndexMap<DataElementIdentifier, IndentToRetain>;

///  Claimed intention of the RP to (not) retain the attribute value after receiving and verifying it, as part of
/// [`DataElements`] within a [`ItemsRequest`].
pub type IndentToRetain = bool;

#[cfg(any(test, feature = "test"))]
mod test {
    use super::*;

    impl DeviceRequest {
        pub fn from_doc_requests(doc_requests: Vec<DocRequest>) -> Self {
            DeviceRequest {
                doc_requests,
                ..Default::default()
            }
        }

        pub fn from_items_requests(items_requests: Vec<ItemsRequest>) -> Self {
            Self::from_doc_requests(
                items_requests
                    .into_iter()
                    .map(|items_request| DocRequest {
                        items_request: items_request.into(),
                        reader_auth: None,
                    })
                    .collect(),
            )
        }
    }
}
