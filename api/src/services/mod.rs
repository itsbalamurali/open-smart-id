pub mod account;
pub mod admin_user;
pub mod audit_log;
pub mod auth;
pub mod certificate;
pub mod device;
pub mod notification;
pub mod relying_party;
pub mod role;
pub mod session;

/// Generate `find_by_id` and `delete` methods for a service.
/// Usage: `impl_crud_basics!(entity_module::Entity, "Entity name");`
macro_rules! impl_crud_basics {
    ($entity:path, $name:expr) => {
        pub async fn find_by_id(
            db: &sea_orm::DatabaseConnection,
            id: &str,
        ) -> Result<<$entity as sea_orm::EntityTrait>::Model, $crate::services::relying_party::ServiceError> {
            use sea_orm::EntityTrait;
            <$entity>::find_by_id(id)
                .one(db)
                .await
                .map_err($crate::services::relying_party::ServiceError::Db)?
                .ok_or_else(|| $crate::services::relying_party::ServiceError::NotFound(
                    concat!($name, " not found").to_string(),
                ))
        }

        pub async fn delete(
            db: &sea_orm::DatabaseConnection,
            id: &str,
        ) -> Result<(), $crate::services::relying_party::ServiceError> {
            use sea_orm::ModelTrait;
            let model = Self::find_by_id(db, id).await?;
            model.delete(db).await.map_err($crate::services::relying_party::ServiceError::Db)?;
            Ok(())
        }
    };
}

pub(crate) use impl_crud_basics;

pub use certificate::CertificateService;
#[allow(unused_imports)]
pub use certificate::{
    AuthCertificateLevel, CertificateInfo, CertificateResponse, CertificateState,
    SignCertificateLevel,
};
pub use notification::NotificationService;
#[allow(unused_imports)]
pub use relying_party::{ApiErrorResponse, ErrorDetail, ProblemDetails};
pub use session::SessionNotifier;
#[allow(unused_imports)]
pub use session::{
    AcspV2Signature, CertificateChoiceSignature, DeviceLinkResponse, FlowType, InteractionType,
    NotificationAuthenticationResponse, NotificationCertificateChoiceResponse,
    NotificationSigningLinkedResponse, NotificationSigningResponse, RawDigestSignature,
    SessionEndResult, SessionResult, SessionResultDetails, SessionSignature,
    SessionSignatureProtocol, SessionState, SessionStatusResponse,
};
