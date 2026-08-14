
//! Generated TeaQL domain crate for `nexus-repository-service-core`.
//!
//! **Before writing queries**, read the `AGENTS.md` at the workspace root.
//! It contains the entity list and the exact `cargo teaql` commands to fetch API prompts.
//!
//! AI coding agents must read this crate's `AGENTS.md` before using generated
//! APIs. If this crate was downloaded from a Cargo registry, locate the
//! unpacked crate source or vendor the dependency, then read `AGENTS.md` from
//! the crate root before writing code against it.

pub mod e;
pub mod q;
pub mod request_support;
pub mod runtime;
pub mod sample_data;
pub mod platform;
pub mod repository_type;
pub mod repository_format;
pub mod write_policy;
pub mod blob_store_type;
pub mod user_status;
pub mod blob_store_configuration;
pub mod repository_configuration;
pub mod content_repository;
pub mod component;
pub mod asset_blob;
pub mod asset;
pub mod security_user;
pub mod security_role;
pub mod security_privilege;

pub use teaql_core;
pub use e::*;
pub use q::*;
pub use request_support::*;
pub use runtime::*;
pub use sample_data::*;
pub use platform::*;
pub use repository_type::*;
pub use repository_format::*;
pub use write_policy::*;
pub use blob_store_type::*;
pub use user_status::*;
pub use blob_store_configuration::*;
pub use repository_configuration::*;
pub use content_repository::*;
pub use component::*;
pub use asset_blob::*;
pub use asset::*;
pub use security_user::*;
pub use security_role::*;
pub use security_privilege::*;