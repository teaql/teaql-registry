// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/asset_blob
use std::collections::BTreeMap;

use teaql_macros::{teaql_entity, TeaqlEntity};

/// [TEAQL AI WARNING]
/// TeaQL was explicitly designed to PREVENT AI hallucinations and random guessing.
/// DO NOT GUESS METHOD NAMES!
/// The methods listed below are the ONLY valid ways to interact with this entity.
/// If you encounter compilation errors (e.g., method not found), DO NOT guess another method name.
/// Read the method signatures in this file before proceeding.
#[teaql_entity]
#[derive(Clone, Debug, PartialEq, TeaqlEntity)]
#[teaql(reverse_relation(
    name = "asset_list",
    target = "Asset",
    local_key = "id",
    foreign_key = "asset_blob_id",
    many
))]
#[teaql(
    entity = "AssetBlob",
    table = "asset_blob_data",
    data_service = "postgres"
)]
pub struct AssetBlob {
    #[teaql(id)]
    id: u64,

    // @source model.xml:157
    blob_ref: String,

    // @source model.xml:157
    blob_size: i64,

    // @source model.xml:157
    content_type: String,

    // @source model.xml:157
    sha1_checksum: String,

    // @source model.xml:157
    sha256_checksum: String,

    // @source model.xml:157
    md5_checksum: String,
    #[teaql(version)]
    version: i64,
    // @source model.xml:157
    #[teaql(column = "blob_store")]
    blob_store_id: u64,
    // @source model.xml:157
    #[teaql(relation(
        target = "BlobStoreConfiguration",
        local_key = "blob_store_id",
        foreign_key = "id"
    ))]
    blob_store: Option<Box<crate::BlobStoreConfiguration>>,
    #[teaql(dynamic)]
    dynamic: BTreeMap<String, teaql_core::Value>,
    #[teaql(skip)]
    pub __load_state: teaql_core::eval::LoadState,
}

impl AssetBlob {
    pub const ENTITY_NAME: &'static str = "Asset Blob";

    pub fn with_id(id: u64) -> teaql_core::Value {
        teaql_core::Value::U64(id)
    }

    pub(crate) fn runtime_new(root: teaql_runtime::EntityRuntimeState) -> Self {
        Self {
            id: 0_u64,
            blob_ref: String::new(),
            blob_size: 0_i64,
            content_type: String::new(),
            sha1_checksum: String::new(),
            sha256_checksum: String::new(),
            md5_checksum: String::new(),
            version: 0_i64,
            blob_store_id: 0_u64,
            blob_store: None,
            dynamic: BTreeMap::new(),
            __teaql_runtime_state: root,
            __load_state: teaql_core::eval::LoadState::FullyLoaded,
        }
    }

    pub fn attach_runtime_state_recursive(&mut self, root: teaql_runtime::EntityRuntimeState) {
        root.adopt_mutations_from(self.__teaql_runtime_state());
        self.__teaql_replace_runtime_state(root.clone());
        if let Some(entity) = &mut self.blob_store {
            entity.attach_runtime_state_recursive(root.clone());
        }
    }

    pub fn is_loaded(&self, field_or_relation: &str) -> bool {
        self.__load_state.is_loaded(field_or_relation)
    }

    pub fn set_load_state(&mut self, state: teaql_core::eval::LoadState) {
        self.__load_state = state;
    }

    pub fn id(&self) -> u64 {
        self.changed_id()
            .and_then(|value| value.try_u64())
            .unwrap_or(self.id)
    }

    pub fn update_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.id = value.try_u64().unwrap_or(self.id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "id", value);
        self
    }

    pub fn changed_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state().get(&self.entity_key(), "id")
    }

    pub fn eval_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "id".to_string(),
                attempted_path: "id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.id())
        }
    }

    pub fn blob_ref(&self) -> String {
        self.changed_blob_ref()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.blob_ref.clone())
    }

    pub fn update_blob_ref(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.blob_ref = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.blob_ref.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "blob_ref", value);
        self
    }

    pub fn changed_blob_ref(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "blob_ref")
    }

    pub fn eval_blob_ref(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("blob_ref") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "blob_ref".to_string(),
                attempted_path: "blob_ref".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.blob_ref())
        }
    }

    pub fn blob_size(&self) -> i64 {
        self.changed_blob_size()
            .and_then(|value| value.try_i64())
            .unwrap_or(self.blob_size)
    }

    pub fn update_blob_size(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.blob_size = value.try_i64().unwrap_or(self.blob_size.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "blob_size", value);
        self
    }

    pub fn changed_blob_size(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "blob_size")
    }

    pub fn eval_blob_size(&self) -> teaql_core::eval::EvalResult<i64> {
        if !self.is_loaded("blob_size") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "blob_size".to_string(),
                attempted_path: "blob_size".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.blob_size())
        }
    }

    pub fn content_type(&self) -> String {
        self.changed_content_type()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.content_type.clone())
    }

    pub fn update_content_type(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.content_type = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.content_type.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "content_type", value);
        self
    }

    pub fn changed_content_type(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "content_type")
    }

    pub fn eval_content_type(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("content_type") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "content_type".to_string(),
                attempted_path: "content_type".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.content_type())
        }
    }

    pub fn sha1_checksum(&self) -> String {
        self.changed_sha1_checksum()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.sha1_checksum.clone())
    }

    pub fn update_sha1_checksum(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.sha1_checksum = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.sha1_checksum.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "sha1_checksum", value);
        self
    }

    pub fn changed_sha1_checksum(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "sha1_checksum")
    }

    pub fn eval_sha1_checksum(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("sha1_checksum") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "sha1_checksum".to_string(),
                attempted_path: "sha1_checksum".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.sha1_checksum())
        }
    }

    pub fn sha256_checksum(&self) -> String {
        self.changed_sha256_checksum()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.sha256_checksum.clone())
    }

    pub fn update_sha256_checksum(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.sha256_checksum = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.sha256_checksum.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "sha256_checksum", value);
        self
    }

    pub fn changed_sha256_checksum(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "sha256_checksum")
    }

    pub fn eval_sha256_checksum(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("sha256_checksum") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "sha256_checksum".to_string(),
                attempted_path: "sha256_checksum".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.sha256_checksum())
        }
    }

    pub fn md5_checksum(&self) -> String {
        self.changed_md5_checksum()
            .and_then(|value| value.try_text().map(|value| value.to_owned()))
            .unwrap_or_else(|| self.md5_checksum.clone())
    }

    pub fn update_md5_checksum(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.md5_checksum = value
            .try_text()
            .map(|value| value.trim().to_owned())
            .unwrap_or_else(|| self.md5_checksum.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "md5_checksum", value);
        self
    }

    pub fn changed_md5_checksum(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "md5_checksum")
    }

    pub fn eval_md5_checksum(&self) -> teaql_core::eval::EvalResult<String> {
        if !self.is_loaded("md5_checksum") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "md5_checksum".to_string(),
                attempted_path: "md5_checksum".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.md5_checksum())
        }
    }

    pub fn version(&self) -> i64 {
        self.changed_version()
            .and_then(|value| value.try_i64())
            .unwrap_or(self.version)
    }

    pub fn update_version(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.version = value.try_i64().unwrap_or(self.version.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "version", value);
        self
    }

    pub fn changed_version(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "version")
    }

    pub fn eval_version(&self) -> teaql_core::eval::EvalResult<i64> {
        if !self.is_loaded("version") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "version".to_string(),
                attempted_path: "version".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.version())
        }
    }
    pub fn blob_store_id(&self) -> u64 {
        self.changed_blob_store_id()
            .and_then(|value| value.try_u64())
            .unwrap_or(self.blob_store_id)
    }

    pub fn update_blob_store_id(&mut self, value: impl Into<teaql_core::Value>) -> &mut Self {
        let value = value.into();
        self.blob_store_id = value.try_u64().unwrap_or(self.blob_store_id.clone());
        self.__teaql_runtime_state()
            .set(self.entity_key(), "blob_store_id", value);
        self
    }

    pub fn changed_blob_store_id(&self) -> Option<teaql_core::Value> {
        self.__teaql_runtime_state()
            .get(&self.entity_key(), "blob_store_id")
    }

    pub fn eval_blob_store_id(&self) -> teaql_core::eval::EvalResult<u64> {
        if !self.is_loaded("blob_store_id") {
            teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "blob_store_id".to_string(),
                attempted_path: "blob_store_id".to_string(),
            }
        } else {
            teaql_core::eval::EvalResult::Value(self.blob_store_id())
        }
    }
    pub fn blob_store(&self) -> Option<&crate::BlobStoreConfiguration> {
        self.blob_store.as_deref().or_else(|| {
            self.__teaql_runtime_state()
                .resolve_entity(self.blob_store_id())
        })
    }

    pub fn eval_blob_store(&self) -> teaql_core::eval::EvalResult<&crate::BlobStoreConfiguration> {
        match self.blob_store() {
            Some(v) => teaql_core::eval::EvalResult::Value(v),
            None if self.is_loaded("blob_store") => teaql_core::eval::EvalResult::Null,
            None => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "blob_store".to_string(),
                attempted_path: "blob_store".to_string(),
            },
        }
    }
    /// Returns the relation view installed by the query that loaded this entity.
    /// This method never performs an implicit database query.
    pub fn asset_list(
        &self,
    ) -> teaql_runtime::RelationHandle<'_, teaql_core::SmartList<crate::Asset>> {
        self.__teaql_runtime_state().relation_list(
            <Self as teaql_core::TeaqlEntity>::ENTITY_NAME,
            self.id(),
            "asset_list",
        )
    }

    pub fn eval_asset_list(
        &self,
    ) -> teaql_core::eval::EvalResult<&teaql_core::SmartList<crate::Asset>> {
        let relation = self.asset_list();
        match relation.state() {
            teaql_runtime::LoadedRelation::Loaded | teaql_runtime::LoadedRelation::Empty => {
                teaql_core::eval::EvalResult::Value(
                    relation
                        .value()
                        .expect("loaded list relation must have a value"),
                )
            }
            teaql_runtime::LoadedRelation::NotLoaded => teaql_core::eval::EvalResult::NotLoaded {
                failed_node: "asset_list".to_string(),
                attempted_path: "asset_list".to_string(),
            },
        }
    }
}
