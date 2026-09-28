//! The one error type every fallible function in this crate returns.
//!
//! Until this type existed every error crossed the bridge as a `String` built
//! with `e.to_string()`, so a Dart caller that needed to react differently to
//! "this message is a replay" and "this session is corrupt" had no choice but
//! to match on the English text of a `Display` impl it does not own. That text
//! is not a contract: libsignal rewords it between releases, and a reworded
//! message silently turns a handled case into an unhandled one.
//!
//! [`LibSignalException`] carries a machine-readable [`LibSignalErrorCode`]
//! beside the same message the `String` used to hold, and its Dart
//! `toString()` returns exactly that message, so existing
//! `e.toString().contains(...)` callers keep working while they migrate.

use libsignal_protocol::{CiphertextMessageType, SignalProtocolError};

/// What went wrong, as a value a caller can branch on.
///
/// One code per libsignal `SignalProtocolError` variant, so a caller never
/// needs the message text to tell two libsignal failures apart. The variant
/// payloads (counters, addresses, lengths) stay in the exception's `message`.
///
/// libsignal's `InvalidMessage` is split by its ciphertext type because "a
/// Whisper message failed to decrypt" and "a pre-key message failed to
/// decrypt" are handled differently by every session-recovery flow.
///
/// Errors this crate authors itself use the code of the libsignal condition
/// they stand for where one exists (a missing session is `sessionNotFound`
/// whichever layer noticed it), and `other` otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibSignalErrorCode {
    InvalidArgument,
    InvalidState,
    InvalidProtobufEncoding,
    CiphertextMessageTooShort,
    LegacyCiphertextVersion,
    UnrecognizedCiphertextVersion,
    UnrecognizedMessageVersion,
    NoKeyTypeIdentifier,
    BadKeyType,
    BadKeyLength,
    InvalidKeyAgreement,
    SignatureValidationFailed,
    UntrustedIdentity,
    InvalidPreKeyId,
    InvalidSignedPreKeyId,
    InvalidKyberPreKeyId,
    InvalidMacKeyLength,
    NoSenderKeyState,
    InvalidProtocolAddress,
    SessionNotFound,
    InvalidSessionStructure,
    InvalidSenderKeySession,
    InvalidRegistrationId,
    /// "message with old counter": a replay or a redelivery of a message whose
    /// key has already been used.
    DuplicatedMessage,
    InvalidWhisperMessage,
    InvalidPreKeyMessage,
    InvalidSenderKeyMessage,
    InvalidPlaintextMessage,
    FfiBindingError,
    ApplicationCallbackError,
    InvalidSealedSenderMessage,
    UnknownSealedSenderVersion,
    SealedSenderSelfSend,
    UnknownSealedSenderServerCertificateId,
    BadKemKeyType,
    WrongKemKeyType,
    BadKemKeyLength,
    BadKemCiphertextLength,
    /// An error this crate raised that has no libsignal counterpart (argument
    /// validation, symmetric-crypto failures, internal invariants).
    Other,
}

/// An error from any fallible call in this package.
///
/// `message` is the text the call used to throw as a bare `String`, unchanged.
/// Branch on `code`; show or log `message`.
#[flutter_rust_bridge::frb(dart_code = "
  @override
  String toString() => message;
")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibSignalException {
    pub code: LibSignalErrorCode,
    pub message: String,
}

impl LibSignalException {
    pub(crate) fn new(code: LibSignalErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for LibSignalException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for LibSignalException {}

impl From<SignalProtocolError> for LibSignalException {
    fn from(error: SignalProtocolError) -> Self {
        use LibSignalErrorCode as C;
        use SignalProtocolError as E;
        // Exhaustive on purpose: a libsignal bump that adds a variant must
        // fail to compile here rather than fall into `Other` unnoticed.
        let code = match &error {
            E::InvalidArgument(_) => C::InvalidArgument,
            E::InvalidState(_, _) => C::InvalidState,
            E::InvalidProtobufEncoding => C::InvalidProtobufEncoding,
            E::CiphertextMessageTooShort(_) => C::CiphertextMessageTooShort,
            E::LegacyCiphertextVersion(_) => C::LegacyCiphertextVersion,
            E::UnrecognizedCiphertextVersion(_) => C::UnrecognizedCiphertextVersion,
            E::UnrecognizedMessageVersion(_) => C::UnrecognizedMessageVersion,
            E::NoKeyTypeIdentifier => C::NoKeyTypeIdentifier,
            E::BadKeyType(_) => C::BadKeyType,
            E::BadKeyLength(_, _) => C::BadKeyLength,
            E::InvalidKeyAgreement => C::InvalidKeyAgreement,
            E::SignatureValidationFailed => C::SignatureValidationFailed,
            E::UntrustedIdentity(_) => C::UntrustedIdentity,
            E::InvalidPreKeyId => C::InvalidPreKeyId,
            E::InvalidSignedPreKeyId => C::InvalidSignedPreKeyId,
            E::InvalidKyberPreKeyId => C::InvalidKyberPreKeyId,
            E::InvalidMacKeyLength(_) => C::InvalidMacKeyLength,
            E::NoSenderKeyState { .. } => C::NoSenderKeyState,
            E::InvalidProtocolAddress { .. } => C::InvalidProtocolAddress,
            E::SessionNotFound(_) => C::SessionNotFound,
            E::InvalidSessionStructure(_) => C::InvalidSessionStructure,
            E::InvalidSenderKeySession { .. } => C::InvalidSenderKeySession,
            E::InvalidRegistrationId(_, _) => C::InvalidRegistrationId,
            E::DuplicatedMessage(_, _) => C::DuplicatedMessage,
            E::InvalidMessage(kind, _) => match kind {
                CiphertextMessageType::Whisper => C::InvalidWhisperMessage,
                CiphertextMessageType::PreKey => C::InvalidPreKeyMessage,
                CiphertextMessageType::SenderKey => C::InvalidSenderKeyMessage,
                CiphertextMessageType::Plaintext => C::InvalidPlaintextMessage,
            },
            E::FfiBindingError(_) => C::FfiBindingError,
            E::ApplicationCallbackError(_, _) => C::ApplicationCallbackError,
            E::InvalidSealedSenderMessage(_) => C::InvalidSealedSenderMessage,
            E::UnknownSealedSenderVersion(_) => C::UnknownSealedSenderVersion,
            E::SealedSenderSelfSend => C::SealedSenderSelfSend,
            E::UnknownSealedSenderServerCertificateId(_) => {
                C::UnknownSealedSenderServerCertificateId
            }
            E::BadKEMKeyType(_) => C::BadKemKeyType,
            E::WrongKEMKeyType(_, _) => C::WrongKemKeyType,
            E::BadKEMKeyLength(_, _) => C::BadKemKeyLength,
            E::BadKEMCiphertextLength(_, _) => C::BadKemCiphertextLength,
        };
        Self::new(code, error.to_string())
    }
}

/// Key parsing fails with a `CurveError`; libsignal already folds that into
/// `SignalProtocolError`, so reuse its mapping rather than restate it.
impl From<libsignal_core::curve::CurveError> for LibSignalException {
    fn from(error: libsignal_core::curve::CurveError) -> Self {
        SignalProtocolError::from(error).into()
    }
}

/// A device id outside libsignal's accepted range is a bad argument.
impl From<libsignal_core::InvalidDeviceId> for LibSignalException {
    fn from(error: libsignal_core::InvalidDeviceId) -> Self {
        Self::new(LibSignalErrorCode::InvalidArgument, error.to_string())
    }
}

/// Crate-authored text with no libsignal counterpart. Lets `?` and
/// `.ok_or("...")?` keep working on every internal path that builds its own
/// message; anything that DOES stand for a libsignal condition must construct
/// the exception with that condition's code instead.
impl From<String> for LibSignalException {
    fn from(message: String) -> Self {
        Self::new(LibSignalErrorCode::Other, message)
    }
}

impl From<&str> for LibSignalException {
    fn from(message: &str) -> Self {
        Self::new(LibSignalErrorCode::Other, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libsignal_protocol::{DeviceId, ProtocolAddress, SessionNotFound};

    fn address() -> ProtocolAddress {
        ProtocolAddress::new("alice".to_owned(), DeviceId::new(1).unwrap())
    }

    #[test]
    fn keeps_the_message_it_used_to_throw() {
        let error = SignalProtocolError::DuplicatedMessage(3, 7);
        let text = error.to_string();
        let converted = LibSignalException::from(error);
        assert_eq!(converted.code, LibSignalErrorCode::DuplicatedMessage);
        assert_eq!(converted.message, text);
        assert_eq!(converted.to_string(), text);
    }

    #[test]
    fn splits_invalid_message_by_ciphertext_type() {
        let whisper = SignalProtocolError::InvalidMessage(
            CiphertextMessageType::Whisper,
            "decryption failed".to_owned(),
        );
        let prekey = SignalProtocolError::InvalidMessage(
            CiphertextMessageType::PreKey,
            "decryption failed".to_owned(),
        );
        assert_eq!(
            LibSignalException::from(whisper).code,
            LibSignalErrorCode::InvalidWhisperMessage
        );
        assert_eq!(
            LibSignalException::from(prekey).code,
            LibSignalErrorCode::InvalidPreKeyMessage
        );
    }

    #[test]
    fn maps_the_conditions_callers_recover_from() {
        let cases = [
            (
                SignalProtocolError::UntrustedIdentity(address()),
                LibSignalErrorCode::UntrustedIdentity,
            ),
            (
                SignalProtocolError::SessionNotFound(SessionNotFound::new(address(), "decrypt")),
                LibSignalErrorCode::SessionNotFound,
            ),
            (
                SignalProtocolError::InvalidProtobufEncoding,
                LibSignalErrorCode::InvalidProtobufEncoding,
            ),
            (
                SignalProtocolError::UnrecognizedMessageVersion(9),
                LibSignalErrorCode::UnrecognizedMessageVersion,
            ),
            (
                SignalProtocolError::LegacyCiphertextVersion(2),
                LibSignalErrorCode::LegacyCiphertextVersion,
            ),
            (
                SignalProtocolError::UnrecognizedCiphertextVersion(9),
                LibSignalErrorCode::UnrecognizedCiphertextVersion,
            ),
        ];
        for (error, code) in cases {
            assert_eq!(LibSignalException::from(error).code, code);
        }
    }

    #[test]
    fn crate_authored_text_is_other() {
        let error: LibSignalException = "Invalid remote device ID".into();
        assert_eq!(error.code, LibSignalErrorCode::Other);
        assert_eq!(error.message, "Invalid remote device ID");
    }
}
