use axum::response::{IntoResponse, Response};
use http::StatusCode;
use nutype::nutype;

use wallet_common::{
    account::messages::errors::{AccountError, AccountErrorType},
    http_error::{HttpJsonError, HttpJsonErrorType},
};
use wallet_provider_service::{
    account_server::{ChallengeError, InstructionError, RegistrationError, WalletCertificateError},
    hsm::HsmError,
};

// Make a newtype to circumvent the orphan rule.
#[nutype(derive(Debug, Clone, From, AsRef, Display, FromStr))]
pub struct WalletProviderErrorType(AccountErrorType);

#[derive(Debug, thiserror::Error)]
pub enum WalletProviderError {
    #[error("{0}")]
    Challenge(#[from] ChallengeError),
    #[error("{0}")]
    Registration(#[from] RegistrationError),
    #[error("{0}")]
    Instruction(#[from] InstructionError),
    #[error("{0}")]
    Hsm(#[from] HsmError),
}

impl HttpJsonErrorType for WalletProviderErrorType {
    fn title(&self) -> String {
        let title = match self.as_ref() {
            AccountErrorType::Unexpected => "An unexpected error occurred",
            AccountErrorType::ChallengeValidation => "Could not validate registration challenge",
            AccountErrorType::RegistrationParsing => "Could not parse or validate registration message",
            AccountErrorType::IncorrectPin => "The PIN provided is incorrect",
            AccountErrorType::PinTimeout => "PIN checking is currently in timeout",
            AccountErrorType::AccountBlocked => "The requested account is blocked",
            AccountErrorType::InstructionValidation => "Could not validate instruction",
        };

        title.to_string()
    }

    fn status_code(&self) -> StatusCode {
        match self.as_ref() {
            AccountErrorType::Unexpected => StatusCode::INTERNAL_SERVER_ERROR,
            AccountErrorType::ChallengeValidation => StatusCode::UNAUTHORIZED,
            AccountErrorType::RegistrationParsing => StatusCode::BAD_REQUEST,
            AccountErrorType::IncorrectPin => StatusCode::FORBIDDEN,
            AccountErrorType::PinTimeout => StatusCode::FORBIDDEN,
            AccountErrorType::AccountBlocked => StatusCode::UNAUTHORIZED,
            AccountErrorType::InstructionValidation => StatusCode::FORBIDDEN,
        }
    }
}

impl From<WalletProviderError> for AccountError {
    fn from(value: WalletProviderError) -> Self {
        match value {
            WalletProviderError::Challenge(error) => match error {
                ChallengeError::WalletCertificate(WalletCertificateError::UserBlocked) => Self::AccountBlocked,
                ChallengeError::WalletCertificate(_) => Self::ChallengeValidation,
                _ => Self::ChallengeValidation,
            },
            WalletProviderError::Registration(error) => match error {
                RegistrationError::ChallengeDecoding(_) => Self::ChallengeValidation,
                RegistrationError::ChallengeValidation(_) => Self::ChallengeValidation,
                RegistrationError::MessageParsing(_) => Self::RegistrationParsing,
                RegistrationError::MessageValidation(_) => Self::RegistrationParsing,
                RegistrationError::SerialNumberMismatch { .. } => Self::RegistrationParsing,
                RegistrationError::PinPubKeyEncoding(_) => Self::Unexpected,
                RegistrationError::JwtSigning(_) => Self::Unexpected,
                RegistrationError::CertificateStorage(_) => Self::Unexpected,
                RegistrationError::WalletCertificate(_) => Self::Unexpected,
                RegistrationError::HsmError(_) => Self::Unexpected,
            },
            WalletProviderError::Instruction(error) => match error {
                InstructionError::IncorrectPin(data) => Self::IncorrectPin(data),
                InstructionError::PinTimeout(data) => Self::PinTimeout(data),
                InstructionError::AccountBlocked => Self::AccountBlocked,
                InstructionError::Validation(_) => Self::InstructionValidation,
                InstructionError::Signing(_)
                | InstructionError::Storage(_)
                | InstructionError::WalletCertificate(_)
                | InstructionError::HsmError(_) => Self::Unexpected,
            },
            WalletProviderError::Hsm(_) => Self::Unexpected,
        }
    }
}

impl From<WalletProviderError> for HttpJsonError<WalletProviderErrorType> {
    fn from(value: WalletProviderError) -> Self {
        let detail = value.to_string();
        let account_error = AccountError::from(value);

        Self::new(
            AccountErrorType::from(&account_error).into(),
            detail,
            account_error.into(),
        )
    }
}

impl IntoResponse for WalletProviderError {
    fn into_response(self) -> Response {
        HttpJsonError::<WalletProviderErrorType>::from(self).into_response()
    }
}
