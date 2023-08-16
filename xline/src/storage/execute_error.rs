use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::request_validation::ValidationError;

/// Error met when executing commands
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ExecuteError {
    /// Invalid Request Error
    #[error("invalid request")]
    InvalidRequest(ValidationError),

    /// Key not found
    #[error("key not found")]
    KeyNotFound,
    /// Revision is higher than current
    #[error("required revision {0} is higher than current revision {1}")]
    RevisionTooLarge(i64, i64),
    /// Revision compacted
    #[error("required revision {0} has been compacted, compacted revision is {1}")]
    RevisionCompacted(i64, i64),

    /// Lease not found
    #[error("lease {0} not found")]
    LeaseNotFound(i64),
    /// Lease is expired
    #[error("lease {0} is expired")]
    LeaseExpired(i64),
    /// Lease ttl is too large
    #[error("lease ttl is too large: {0}")]
    LeaseTtlTooLarge(i64),
    /// Lease already exists
    #[error("lease {0} already exists")]
    LeaseAlreadyExists(i64),

    // AuthErrors
    /// Auth is not enabled
    #[error("auth is not enabled")]
    AuthNotEnabled,
    /// Auth failed
    #[error("invalid username or password")]
    AuthFailed,
    /// User not found
    #[error("user {0} not found")]
    UserNotFound(String),
    /// User already exists
    #[error("user {0} already exists")]
    UserAlreadyExists(String),
    /// User already has role
    #[error("user {0} already has role {1}")]
    UserAlreadyHasRole(String, String),
    /// Password was given for no password user
    #[error("password was given for no password user")]
    NoPasswordUser,
    /// Role not found
    #[error("role {0} not found")]
    RoleNotFound(String),
    /// Role already exists
    #[error("role {0} already exists")]
    RoleAlreadyExists(String),
    /// Role not granted
    #[error("role {0} is not granted to the user")]
    RoleNotGranted(String),
    /// Root role not exist
    #[error("root user does not have root role")]
    RootRoleNotExist,
    /// Permission not granted
    #[error("permission not granted to the role")]
    PermissionNotGranted,
    /// Permission not given
    #[error("permission not given")]
    PermissionNotGiven,
    /// Invalid auth management
    #[error("invalid auth management")]
    InvalidAuthManagement,
    /// Invalid auth token
    #[error("invalid auth token")]
    InvalidAuthToken,
    /// Token manager is not initialized
    #[error("token manager is not initialized")]
    TokenManagerNotInit,
    /// Token is not provided
    #[error("token is not provided")]
    TokenNotProvided,
    /// Token is expired
    #[error("token's revision {0} is older than current revision {1}")]
    TokenOldRevision(i64, i64),

    /// Db error
    #[error("db error: {0}")]
    DbError(String),

    /// Permission denied Error
    #[error("permission denied")]
    PermissionDenied,
}

// The etcd client relies on GRPC error messages for error type interpretation.
// In order to create an etcd-compatible API with Xline, it is necessary to return exact GRPC statuses to the etcd client.
// Refer to `https://github.com/etcd-io/etcd/blob/main/api/v3rpc/rpctypes/error.go` for etcd's error parsing mechanism,
// and refer to `https://github.com/etcd-io/etcd/blob/main/client/v3/doc.go` for how errors are handled by etcd client.
impl From<ExecuteError> for tonic::Status {
    #[inline]
    fn from(err: ExecuteError) -> Self {
        let (code, message) = match err {
            ExecuteError::InvalidRequest(e) => return e.into(),
            ExecuteError::KeyNotFound => (
                tonic::Code::InvalidArgument,
                "etcdserver: key not found".to_owned(),
            ),
            ExecuteError::RevisionTooLarge(_, _) => (
                tonic::Code::OutOfRange,
                "etcdserver: mvcc: required revision is a future revision".to_owned(),
            ),
            ExecuteError::RevisionCompacted(_, _) => (
                tonic::Code::OutOfRange,
                "etcdserver: mvcc: required revision has been compacted".to_owned(),
            ),
            ExecuteError::LeaseNotFound(_) => (
                tonic::Code::NotFound,
                "etcdserver: requested lease not found".to_owned(),
            ),
            ExecuteError::LeaseTtlTooLarge(_) => (
                tonic::Code::OutOfRange,
                "etcdserver: too large lease TTL".to_owned(),
            ),
            ExecuteError::LeaseAlreadyExists(_) => (
                tonic::Code::FailedPrecondition,
                "etcdserver: lease already exists".to_owned(),
            ),
            ExecuteError::AuthNotEnabled => (
                tonic::Code::FailedPrecondition,
                "etcdserver: authentication is not enabled".to_owned(),
            ),
            ExecuteError::AuthFailed => (
                tonic::Code::InvalidArgument,
                "etcdserver: authentication failed, invalid user ID or password".to_owned(),
            ),
            ExecuteError::UserNotFound(_) => (
                tonic::Code::FailedPrecondition,
                "etcdserver: user name not found".to_owned(),
            ),
            ExecuteError::UserAlreadyExists(_) => (
                tonic::Code::FailedPrecondition,
                "etcdserver: user name already exists".to_owned(),
            ),
            ExecuteError::RoleNotFound(_) => (
                tonic::Code::FailedPrecondition,
                "etcdserver: role name not found".to_owned(),
            ),
            ExecuteError::RoleAlreadyExists(_) => (
                tonic::Code::FailedPrecondition,
                "etcdserver: role name already exists".to_owned(),
            ),
            ExecuteError::RoleNotGranted(_) => (
                tonic::Code::FailedPrecondition,
                "etcdserver: role is not granted to the user".to_owned(),
            ),
            ExecuteError::RootRoleNotExist => (
                tonic::Code::FailedPrecondition,
                "etcdserver: root user does not have root role".to_owned(),
            ),
            ExecuteError::PermissionNotGranted => (
                tonic::Code::FailedPrecondition,
                "etcdserver: permission is not granted to the role".to_owned(),
            ),
            ExecuteError::PermissionNotGiven => (
                tonic::Code::InvalidArgument,
                "etcdserver: permission not given".to_owned(),
            ),
            ExecuteError::InvalidAuthToken | ExecuteError::TokenOldRevision(_, _) => (
                tonic::Code::Unauthenticated,
                "etcdserver: invalid auth token".to_owned(),
            ),
            ExecuteError::PermissionDenied => (
                tonic::Code::PermissionDenied,
                "etcdserver: permission denied".to_owned(),
            ),
            ExecuteError::InvalidAuthManagement => (
                tonic::Code::InvalidArgument,
                "etcdserver: invalid auth management".to_owned(),
            ),
            ExecuteError::LeaseExpired(_) => (tonic::Code::DeadlineExceeded, err.to_string()),
            ExecuteError::UserAlreadyHasRole(_, _)
            | ExecuteError::NoPasswordUser
            | ExecuteError::TokenManagerNotInit => {
                (tonic::Code::FailedPrecondition, err.to_string())
            }
            ExecuteError::TokenNotProvided => (tonic::Code::InvalidArgument, err.to_string()),
            ExecuteError::DbError(_) => (tonic::Code::Internal, err.to_string()),
        };

        tonic::Status::new(code, message)
    }
}
