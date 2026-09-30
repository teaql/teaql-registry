//! Generated TeaQL domain crate for `teaql-registry-service-core`.
//!
//! **Before writing queries**, read the `AGENTS.md` at the workspace root.
//! It contains the entity list and the exact `cargo teaql` commands to fetch API prompts.
//!
//! AI coding agents must read this crate's `AGENTS.md` before using generated
//! APIs. If this crate was downloaded from a Cargo registry, locate the
//! unpacked crate source or vendor the dependency, then read `AGENTS.md` from
//! the crate root before writing code against it.

pub mod asset;
pub mod asset_blob;
pub mod blob_store_configuration;
pub mod blob_store_type;
pub mod component;
pub mod content_repository;
pub mod e;
pub mod personal_access_token;
pub mod platform;
pub mod q;
pub mod repository_configuration;
pub mod repository_format;
pub mod repository_type;
pub mod request_support;
pub mod runtime;
pub mod sample_data;
pub mod security_privilege;
pub mod security_role;
pub mod security_role_privilege;
pub mod security_user;
pub mod security_user_role;
pub mod service_log;
pub mod tenant;
pub mod user_status;
pub mod write_policy;

pub use asset::*;
pub use asset_blob::*;
pub use blob_store_configuration::*;
pub use blob_store_type::*;
pub use component::*;
pub use content_repository::*;
pub use e::*;
pub use personal_access_token::*;
pub use platform::*;
pub use q::*;
pub use repository_configuration::*;
pub use repository_format::*;
pub use repository_type::*;
pub use request_support::*;
pub use runtime::*;
pub use sample_data::*;
pub use security_privilege::*;
pub use security_role::*;
pub use security_role_privilege::*;
pub use security_user::*;
pub use security_user_role::*;
pub use service_log::*;
pub use teaql_core;
pub use tenant::*;
pub use user_status::*;
pub use write_policy::*;
