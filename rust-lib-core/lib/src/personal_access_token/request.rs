use std::marker::PhantomData;

use serde_json::Value as JsonValue;
use teaql_core::{Aggregate, AggregateFunction, EntityDescriptor, Expr, SelectQuery, SmartList};
use teaql_runtime::{DataServiceError, RuntimeError};

use crate::request_support::*;

impl EntityReference for crate::PersonalAccessToken {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(&self)
    }
}

impl EntityReference for &crate::PersonalAccessToken {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(self)
    }
}

// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/personal_access_token
#[derive(Debug)]
pub struct PersonalAccessTokenRequest<R = crate::PersonalAccessToken> {
    query: SelectQuery,
    relation_selections: Vec<RelationSelection>,
    relation_filters: Vec<RelationFilter>,
    child_enhancements: Vec<QuerySelection>,
    query_options: QueryOptions,
    marker: PhantomData<R>,
}

impl<R> Clone for PersonalAccessTokenRequest<R> {
    fn clone(&self) -> Self {
        Self {
            query: self.query.clone(),
            relation_selections: self.relation_selections.clone(),
            relation_filters: self.relation_filters.clone(),
            child_enhancements: self.child_enhancements.clone(),
            query_options: self.query_options.clone(),
            marker: PhantomData,
        }
    }
}

impl<R> PersonalAccessTokenRequest<R> {
    pub(crate) fn new() -> Self {
        Self {
            query: SelectQuery::new("PersonalAccessToken")
                .project("id")
                .project("version"),
            relation_selections: Vec::new(),
            relation_filters: Vec::new(),
            child_enhancements: Vec::new(),
            query_options: QueryOptions::default(),
            marker: PhantomData,
        }
    }

    pub fn return_type<T>(self) -> PersonalAccessTokenRequest<T> {
        PersonalAccessTokenRequest {
            query: self.query,
            relation_selections: self.relation_selections,
            relation_filters: self.relation_filters,
            child_enhancements: self.child_enhancements,
            query_options: self.query_options,
            marker: PhantomData,
        }
    }

    pub fn query(&self) -> &SelectQuery {
        &self.query
    }

    pub fn relation_selections(&self) -> &[RelationSelection] {
        &self.relation_selections
    }

    pub fn relation_filters(&self) -> &[RelationFilter] {
        &self.relation_filters
    }

    pub fn child_enhancements(&self) -> &[QuerySelection] {
        &self.child_enhancements
    }

    pub fn query_options(&self) -> &QueryOptions {
        &self.query_options
    }

    pub fn into_query(self) -> SelectQuery {
        self.query
    }

    pub fn purpose(self, purpose: impl Into<String>) -> crate::PurposedQuery<Self> {
        crate::PurposedQuery::new(self, purpose)
    }

    pub(crate) async fn _execute_for_list<'a, C>(
        self,
        context: &'a C,
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        let repository = context
            .personal_access_token_repository()
            .map_err(|err| DataServiceError::Runtime(RuntimeError::Graph(err.to_string())))?;
        let query_options = self.query_options.clone();
        let relation_aggregates = runtime_relation_aggregates(&query_options);
        let query = authorize_query(apply_runtime_metadata(
            self.query,
            &query_options,
            &self.child_enhancements,
        ))
        .map_err(DataServiceError::Runtime)?;
        let (mut rows, facets) = if query_options.facets.is_empty() {
            let rows = repository
                .fetch_enhanced_entities_with_relation_aggregates_owned::<R>(
                    query,
                    &relation_aggregates,
                )
                .await?;
            (rows, std::collections::BTreeMap::new())
        } else {
            let rows = repository
                .fetch_enhanced_entities_with_relation_aggregates::<R>(&query, &relation_aggregates)
                .await?;
            let facets = execute_facets(context, query.as_query(), &query_options)
                .await
                .map_err(DataServiceError::Runtime)?;
            (rows, facets)
        };
        attach_facets(&mut rows, facets);
        Ok(rows)
    }

    pub(crate) async fn _execute_for_rows<'a, C>(
        self,
        context: &'a C,
    ) -> Result<
        SmartList<teaql_core::CompactRow>,
        TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>,
    >
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = context
            .personal_access_token_repository()
            .map_err(|err| DataServiceError::Runtime(RuntimeError::Graph(err.to_string())))?;
        let query = authorize_query(apply_runtime_metadata(
            self.query,
            &self.query_options,
            &self.child_enhancements,
        ))
        .map_err(DataServiceError::Runtime)?;
        repository.fetch_smart_list(&query).await
    }

    pub(crate) async fn _execute_for_stream<'a, C>(
        self,
        context: &'a C,
    ) -> Result<
        TeaqlEntityStream<'a, R, TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>>,
        TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>,
    >
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity + 'a,
    {
        Ok(Box::pin(async_stream::try_stream! {
            use futures_util::StreamExt;
            let repository = context
                .personal_access_token_repository()
                .map_err(|err| DataServiceError::Runtime(RuntimeError::Graph(err.to_string())))?;
            let query_options = self.query_options.clone();
            let query = authorize_query(apply_runtime_metadata(
                self.query,
                &query_options,
                &self.child_enhancements,
            )).map_err(DataServiceError::Runtime)?;
            let mut chunks = repository.fetch_stream(&query).await?;
            while let Some(chunk) = chunks.next().await {
                for row in chunk?.rows {
                    yield R::from_compact_row(row).map_err(DataServiceError::Entity)?;
                }
            }
        }))
    }

    pub(crate) async fn _execute_for_first<'a, C>(
        self,
        context: &'a C,
    ) -> Result<Option<R>, TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        let rows = self.limit(1)._execute_for_list(context).await?;
        Ok(rows.into_iter().next())
    }

    pub(crate) async fn _execute_for_one<'a, C>(
        self,
        context: &'a C,
    ) -> Result<Option<R>, TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        self._execute_for_first(context).await
    }

    pub(crate) async fn _execute_for_page<'a, C>(
        self,
        context: &'a C,
        offset: u64,
        limit: u64,
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        if self.query.id_set_pagination.is_some() {
            let mut rows = self
                .clone()
                .page_offset(offset, limit)
                ._execute_for_list(context)
                .await?;
            if rows.total_count.is_none() {
                rows.total_count = Some(self._execute_for_count(context).await?);
            }
            return Ok(rows);
        }
        let total_count = self.clone()._execute_for_count(context).await?;
        let mut rows = self
            .page_offset(offset, limit)
            ._execute_for_list(context)
            .await?;
        rows.total_count = Some(total_count);
        Ok(rows)
    }

    pub(crate) async fn _execute_for_count<'a, C>(
        self,
        context: &'a C,
    ) -> Result<u64, TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = context
            .personal_access_token_repository()
            .map_err(|err| DataServiceError::Runtime(RuntimeError::Graph(err.to_string())))?;
        let query_options = self.query_options.clone();
        let mut query =
            apply_runtime_metadata(self.query, &query_options, &self.child_enhancements);
        query.projection.clear();
        query.expr_projection.clear();
        query.order_by.clear();
        query.slice = None;
        query.relations.clear();
        query = query.count(COUNT_ALIAS);
        let query = authorize_query(query).map_err(DataServiceError::Runtime)?;
        let rows = repository.fetch_all(&query).await?;
        rows.first()
            .and_then(|row| row.get(COUNT_ALIAS))
            .and_then(teaql_core::Value::try_u64)
            .ok_or_else(|| {
                DataServiceError::Runtime(RuntimeError::Graph(format!(
                    "count result for PersonalAccessToken is missing or not numeric"
                )))
            })
    }

    pub(crate) async fn _execute_for_exists<'a, C>(
        self,
        context: &'a C,
    ) -> Result<bool, TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = context
            .personal_access_token_repository()
            .map_err(|err| DataServiceError::Runtime(RuntimeError::Graph(err.to_string())))?;
        let mut query = self.query.limit(1);
        query.relations.clear();
        let query = authorize_query(query).map_err(DataServiceError::Runtime)?;
        let rows = repository.fetch_all(&query).await?;
        Ok(!rows.is_empty())
    }

    pub fn search_with_text(mut self, text: impl Into<String>) -> Self {
        self.query = self.query.search_with_text(text);
        self
    }

    pub fn filter(mut self, filter: Expr) -> Self {
        self.query = self.query.filter(filter);
        self
    }

    pub fn and_filter(mut self, filter: Expr) -> Self {
        self.query = self.query.and_filter(filter);
        self
    }

    pub fn or_filter(mut self, filter: Expr) -> Self {
        self.query = self.query.or_filter(filter);
        self
    }

    pub fn append_search_criteria(self, criteria: Expr) -> Self {
        self.and_filter(criteria)
    }

    pub fn filter_property(
        mut self,
        property1: impl AsRef<str>,
        operator: FieldOperator,
        property2: impl AsRef<str>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_column_expr(
            property1.as_ref(),
            operator,
            property2.as_ref(),
        ));
        self
    }

    pub fn with_deleted_rows(mut self) -> Self {
        self.query.filter = remove_default_live_filter(self.query.filter);
        self
    }

    pub fn deleted_rows_only(mut self) -> Self {
        self.query.filter = remove_default_live_filter(self.query.filter);
        self.query = self.query.and_filter(Expr::lte("version", 0_i64));
        self
    }

    pub fn match_types(
        mut self,
        types: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::in_list(TYPE_FIELD, types.into_iter().map(Into::into)));
        self
    }

    pub fn with_type_group(mut self) -> Self {
        self.query = self.query.project(TYPE_GROUP_FIELD);
        self
    }

    pub fn matching_any_of(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        let entity = EntityDescriptor::new(selection.query.entity.clone());
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            entity,
            selection.query.clone(),
            "id",
        ));
        self
    }

    pub fn match_any_of(self, request: impl Into<QuerySelection>) -> Self {
        self.matching_any_of(request)
    }

    pub fn enhance_child(mut self, request: impl Into<QuerySelection>) -> Self {
        self.child_enhancements.push(request.into());
        self
    }

    pub fn enhance_children_if_needed(self) -> Self {
        let request = self;
        request
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.query_options.comment = Some(comment.into());
        self
    }

    pub fn raw_sql(self, raw_sql: impl Into<String>) -> Self {
        self.unsafe_raw_sql(UnsafeRawSqlSegment::trusted(raw_sql))
    }

    pub fn unsafe_raw_sql(mut self, raw_sql: UnsafeRawSqlSegment) -> Self {
        self.query_options.raw_sql = Some(raw_sql.into_sql());
        self
    }

    pub fn raw_sql_filter(self, raw_sql: impl Into<String>) -> Self {
        self.unsafe_raw_sql_filter(UnsafeRawSqlSegment::trusted(raw_sql))
    }

    pub fn unsafe_raw_sql_filter(mut self, raw_sql: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_sql_search_criteria
            .push(raw_sql.into_sql());
        self
    }
    pub fn filter_with_json(self, json_expr: impl Into<String>) -> Self {
        self.merge_dynamic_json_expr(json_expr.into())
    }

    fn merge_dynamic_json_expr(self, json_expr: String) -> Self {
        let json = serde_json::from_str::<JsonValue>(&json_expr)
            .unwrap_or_else(|_| panic!("Input JSON format error: {json_expr}"));
        self.merge_dynamic_json(&json)
    }

    fn merge_dynamic_json(mut self, json: &JsonValue) -> Self {
        let Some(object) = json.as_object() else {
            return self;
        };

        for (field, value) in object {
            if field.starts_with('_') {
                continue;
            }
            self = self.apply_dynamic_json_filter(field, value);
        }

        self = self.apply_dynamic_json_order_by(object.get("_orderBy"));

        if let Some(offset) = dynamic_json_u64_field(object, "_start") {
            self = self.skip(offset);
        }
        if let Some(size) = dynamic_json_u64_field(object, "_size") {
            self = self.limit(size);
        }

        if let Some(page_size) = dynamic_json_u64_field(object, "_pageSize") {
            self = self.limit(page_size);
        }
        if let Some(page_number) = dynamic_json_u64_field(object, "_page") {
            if page_number > 0 {
                let size = dynamic_json_u64_field(object, "_pageSize")
                    .or_else(|| self.query.slice.as_ref().and_then(|slice| slice.limit))
                    .unwrap_or(10);
                let offset = page_number.saturating_sub(1).saturating_mul(size);
                self = self.page_offset(offset, size);
            }
        }

        self
    }

    pub(crate) fn apply_dynamic_json_filter(self, field: &str, value: &JsonValue) -> Self {
        if let Some((head, tail)) = field.split_once('.') {
            self.apply_dynamic_json_chain_filter(head, tail, value)
        } else if let Some(storage_field) = Self::dynamic_json_self_field(field) {
            self.and_filter(dynamic_json_filter_expr(storage_field, value))
        } else {
            self
        }
    }

    fn apply_dynamic_json_order_by(mut self, order_by: Option<&JsonValue>) -> Self {
        match order_by {
            Some(JsonValue::String(field)) => {
                if let Some(storage_field) = Self::dynamic_json_self_field(field) {
                    self.query = self.query.order_desc(storage_field);
                }
            }
            Some(JsonValue::Object(order_by)) => {
                self = self.apply_dynamic_json_single_order_by(order_by);
            }
            Some(JsonValue::Array(order_bys)) => {
                for order_by in order_bys {
                    if let Some(order_by) = order_by.as_object() {
                        self = self.apply_dynamic_json_single_order_by(order_by);
                    }
                }
            }
            _ => {}
        }
        self
    }

    fn apply_dynamic_json_single_order_by(
        mut self,
        order_by: &serde_json::Map<String, JsonValue>,
    ) -> Self {
        let Some(field) = order_by.get("field").and_then(JsonValue::as_str) else {
            return self;
        };
        let Some(storage_field) = Self::dynamic_json_self_field(field) else {
            return self;
        };
        if order_by
            .get("useAsc")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
        {
            self.query = self.query.order_asc(storage_field);
        } else {
            self.query = self.query.order_desc(storage_field);
        }
        self
    }

    fn dynamic_json_self_field(field: &str) -> Option<&'static str> {
        match field {
            "id" => Some("id"),
            "username" => Some("username"),
            "token_id" => Some("token_id"),
            "token_hash" => Some("token_hash"),
            "description" => Some("description"),
            "scopes" => Some("scopes"),
            "created_at" => Some("created_at"),
            "expires_at_epoch_millis" => Some("expires_at_epoch_millis"),
            "revoked" => Some("revoked"),
            "revoked_at_epoch_millis" => Some("revoked_at_epoch_millis"),
            "version" => Some("version"),
            "tenant" | "tenant_id" => Some("tenant_id"),
            "security_user" | "security_user_id" => Some("security_user_id"),
            _ => None,
        }
    }

    fn apply_dynamic_json_chain_filter(self, head: &str, tail: &str, value: &JsonValue) -> Self {
        let _ = (tail, value);
        match head {
            "tenant" => self.with_tenant_matching(
                crate::Q::tenants_minimal().apply_dynamic_json_filter(tail, value),
            ),
            "security_user" => self.with_security_user_matching(
                crate::Q::security_users_minimal().apply_dynamic_json_filter(tail, value),
            ),
            _ => self,
        }
    }

    pub fn create_property_as(
        self,
        property_name: impl Into<String>,
        raw_sql_segment: impl Into<String>,
    ) -> Self {
        self.unsafe_create_property_as(property_name, UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn unsafe_create_property_as(
        mut self,
        property_name: impl Into<String>,
        raw_sql_segment: UnsafeRawSqlSegment,
    ) -> Self {
        self.query_options
            .dynamic_properties
            .push(RawDynamicProperty::new(property_name, raw_sql_segment));
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.query = self.query.limit(limit);
        self
    }

    pub fn stream(mut self, chunk_size: usize) -> Self {
        assert!(chunk_size > 0, "stream chunk size must be positive");
        self.query = self.query.stream(chunk_size);
        self
    }

    pub fn stream_default(mut self) -> Self {
        self.query = self.query.stream_default();
        self
    }

    pub fn skip(mut self, offset: u64) -> Self {
        self.query = self.query.offset(offset);
        self
    }

    pub fn offset_only(self, offset: u64) -> Self {
        self.skip(offset)
    }

    pub fn offset(self, offset: u64, size: u64) -> Self {
        self.page_offset(offset, size)
    }

    pub fn page_offset(mut self, offset: u64, limit: u64) -> Self {
        self.query = self.query.page(offset, limit);
        self
    }

    pub fn optimize_for_continuous_page_fetch(mut self) -> Self {
        self.query = self.query.optimize_for_continuous_page_fetch();
        self
    }

    pub fn optimize_for_continuous_page_fetch_with(
        mut self,
        namespace: impl Into<String>,
        ttl_seconds: u64,
    ) -> Self {
        self.query = self
            .query
            .optimize_for_continuous_page_fetch_with(namespace, ttl_seconds);
        self
    }

    pub fn optimize_pagination_with_id_set(mut self) -> Self {
        self.query = self.query.optimize_pagination_with_id_set();
        self
    }

    pub fn optimize_pagination_with_id_set_config(
        mut self,
        namespace: impl Into<String>,
        ttl_seconds: u64,
        max_ids: u64,
    ) -> Self {
        self.query =
            self.query
                .optimize_pagination_with_id_set_config(namespace, ttl_seconds, max_ids);
        self
    }

    /// Select bounded indexed probes for a per-parent Top-N relation only
    /// when the already-loaded parent count is at or below `threshold`.
    /// Passing zero explicitly selects the provider window plan.
    pub fn top_n_probe_parent_threshold(mut self, threshold: usize) -> Self {
        self.query = self.query.top_n_probe_parent_threshold(threshold);
        self
    }

    pub fn top(self, top_n: u64) -> Self {
        self.limit(top_n)
    }

    pub fn offset_size(self, offset: u64, size: u64) -> Self {
        self.offset(offset, size)
    }

    pub fn unlimited(mut self) -> Self {
        self.query.slice = None;
        self
    }

    pub fn page_number(self, page_number: u64, page_size: u64) -> Self {
        let offset = page_number.saturating_sub(1).saturating_mul(page_size);
        self.page_offset(offset, page_size)
    }

    pub fn page_number_default(self, page_number: u64) -> Self {
        self.page_number(page_number, 10)
    }

    pub fn page(self, page_number: u64, page_size: u64) -> Self {
        self.page_number(page_number, page_size)
    }

    pub fn page_default(self, page_number: u64) -> Self {
        self.page_number_default(page_number)
    }

    pub fn select_self(mut self) -> Self {
        self.query = self.query.project("id");
        self.query = self.query.project("username");
        self.query = self.query.project("token_id");
        self.query = self.query.project("token_hash");
        self.query = self.query.project("description");
        self.query = self.query.project("scopes");
        self.query = self.query.project("created_at");
        self.query = self.query.project("expires_at_epoch_millis");
        self.query = self.query.project("revoked");
        self.query = self.query.project("revoked_at_epoch_millis");
        self.query = self.query.project("version");
        self.query = self.query.project("tenant_id");
        self.query = self.query.project("security_user_id");
        self
    }

    pub fn select_self_fields(self) -> Self {
        self.select_self()
    }

    pub fn select_self_without_parent(self) -> Self {
        self.select_self_fields()
    }

    pub fn select_all(self) -> Self {
        let mut request = self.select_self();
        request = request.select_tenant();
        request = request.select_security_user();
        request
    }

    pub fn select_children(self) -> Self {
        self.select_all()
    }

    pub fn select_any(self) -> Self {
        self.select_children()
    }

    pub fn group_by(mut self, field: impl Into<String>) -> Self {
        self.query = self.query.group_by(field);
        self
    }

    pub fn count(self) -> Self {
        self.count_as("count")
    }

    pub fn count_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count(alias)
    }

    pub fn aggregate_count(mut self, alias: impl Into<String>) -> Self {
        self.query = self.query.count(alias);
        self
    }

    pub fn aggregate_count_field(
        mut self,
        field: impl Into<String>,
        alias: impl Into<String>,
    ) -> Self {
        self.query = self.query.count_field(field, alias);
        self
    }

    pub fn aggregate_with_function(
        mut self,
        field: impl Into<String>,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.query = self.query.aggregate(Aggregate::new(function, field, alias));
        self
    }

    pub fn aggregate_sum(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
        self.query = self.query.sum(field, alias);
        self
    }

    pub fn aggregate_avg(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
        self.query = self.query.avg(field, alias);
        self
    }

    pub fn aggregate_min(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
        self.query = self.query.min(field, alias);
        self
    }

    pub fn aggregate_max(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
        self.query = self.query.max(field, alias);
        self
    }

    pub fn aggregate_stddev(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
        self.query = self.query.stddev(field, alias);
        self
    }

    pub fn aggregate_stddev_pop(
        mut self,
        field: impl Into<String>,
        alias: impl Into<String>,
    ) -> Self {
        self.query = self.query.stddev_pop(field, alias);
        self
    }

    pub fn aggregate_var_samp(
        mut self,
        field: impl Into<String>,
        alias: impl Into<String>,
    ) -> Self {
        self.query = self.query.var_samp(field, alias);
        self
    }

    pub fn aggregate_var_pop(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
        self.query = self.query.var_pop(field, alias);
        self
    }

    pub fn aggregate_bit_and(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
        self.query = self.query.bit_and(field, alias);
        self
    }

    pub fn aggregate_bit_or(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
        self.query = self.query.bit_or(field, alias);
        self
    }

    pub fn aggregate_bit_xor(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
        self.query = self.query.bit_xor(field, alias);
        self
    }

    pub fn enable_aggregation_cache(mut self) -> Self {
        self.query = self.query.enable_aggregation_cache();
        self
    }

    pub fn enable_aggregation_cache_for(mut self, cache_expired_millis: u64) -> Self {
        self.query = self
            .query
            .enable_aggregation_cache_for(cache_expired_millis);
        self
    }

    pub fn propagate_aggregation_cache(mut self, cache_expired_millis: u64) -> Self {
        self.query = self.query.propagate_aggregation_cache(cache_expired_millis);
        self
    }

    pub fn group_by_id(self) -> Self {
        self.group_by("id")
    }

    pub fn group_by_id_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("id");
        request.query = request.query.project_expr(alias, Expr::column("id"));
        request
    }

    pub fn group_by_id_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("id")
            .aggregate_with_function("id", alias, function)
    }

    pub fn count_id(self) -> Self {
        self.count_id_as("id_count")
    }

    pub fn count_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("id", alias)
    }

    pub fn sum_id(self) -> Self {
        self.sum_id_as("sum_id")
    }

    pub fn sum_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("id", alias)
    }

    pub fn avg_id(self) -> Self {
        self.avg_id_as("avg_id")
    }

    pub fn avg_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("id", alias)
    }

    pub fn min_id(self) -> Self {
        self.min_id_as("min_id")
    }

    pub fn min_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("id", alias)
    }

    pub fn max_id(self) -> Self {
        self.max_id_as("max_id")
    }

    pub fn max_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("id", alias)
    }

    pub fn with_id(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "id",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_id_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr("id", operator, values.into_iter().map(Into::into).collect())
    }

    pub fn with_id_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("id", value));
        self
    }

    pub fn with_id_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("id", value));
        self
    }

    pub fn with_id_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::in_list("id", values.into_iter().map(Into::into)));
        self
    }

    pub fn with_id_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_in_list("id", values.into_iter().map(Into::into)));
        self
    }

    pub fn order_by_id_asc(mut self) -> Self {
        self.query = self.query.order_asc("id");
        self
    }

    pub fn order_by_id_desc(mut self) -> Self {
        self.query = self.query.order_desc("id");
        self
    }

    pub fn order_by_id_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("id");
        self
    }

    pub fn order_by_id_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("id");
        self
    }

    pub fn select_username(mut self) -> Self {
        self.query = self.query.project("username");
        self
    }

    pub fn project_username(self) -> Self {
        self.select_username()
    }

    pub fn select_username_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_username_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_username_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("username", raw_sql_segment));
        self
    }

    pub fn group_by_username(self) -> Self {
        self.group_by("username")
    }

    pub fn group_by_username_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("username");
        request.query = request.query.project_expr(alias, Expr::column("username"));
        request
    }

    pub fn group_by_username_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("username")
            .aggregate_with_function("username", alias, function)
    }

    pub fn count_username(self) -> Self {
        self.count_username_as("username_count")
    }

    pub fn count_username_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("username", alias)
    }

    pub fn sum_username(self) -> Self {
        self.sum_username_as("sum_username")
    }

    pub fn sum_username_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("username", alias)
    }

    pub fn avg_username(self) -> Self {
        self.avg_username_as("avg_username")
    }

    pub fn avg_username_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("username", alias)
    }

    pub fn min_username(self) -> Self {
        self.min_username_as("min_username")
    }

    pub fn min_username_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("username", alias)
    }

    pub fn max_username(self) -> Self {
        self.max_username_as("max_username")
    }

    pub fn max_username_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("username", alias)
    }

    pub fn unselect_username(mut self) -> Self {
        self.query.projection.retain(|field| field != "username");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "username");
        self
    }

    pub fn with_username(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "username",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_username_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "username",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_username_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("username", value));
        self
    }

    pub fn with_username_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("username", value));
        self
    }

    pub fn with_username_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("username", value));
        self
    }

    pub fn with_username_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("username", value));
        self
    }

    pub fn with_username_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("username", value));
        self
    }

    pub fn with_username_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("username", value));
        self
    }

    pub fn with_username_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("username", lower, upper));
        self
    }

    pub fn with_username_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("username", range.start, range.end));
        self
    }

    pub fn with_username_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "username",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_username_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "username",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_username_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("username", value));
        self
    }

    pub fn with_username_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("username", value));
        self
    }

    pub fn with_username_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("username", value));
        self
    }

    pub fn with_username_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_begin_with("username", value));
        self
    }

    pub fn with_username_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("username", value));
        self
    }

    pub fn with_username_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("username", value));
        self
    }

    pub fn with_username_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("username", value));
        self
    }
    pub fn with_username_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("username", value));
        self
    }

    pub fn with_username_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("username", value));
        self
    }

    pub fn with_username_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("username"));
        self
    }

    pub fn with_username_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("username"));
        self
    }

    pub fn order_by_username_asc(mut self) -> Self {
        self.query = self.query.order_asc("username");
        self
    }

    pub fn order_by_username_desc(mut self) -> Self {
        self.query = self.query.order_desc("username");
        self
    }

    pub fn order_by_username_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("username");
        self
    }

    pub fn order_by_username_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("username");
        self
    }

    pub fn select_token_id(mut self) -> Self {
        self.query = self.query.project("token_id");
        self
    }

    pub fn project_token_id(self) -> Self {
        self.select_token_id()
    }

    pub fn select_token_id_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_token_id_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_token_id_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("token_id", raw_sql_segment));
        self
    }

    pub fn group_by_token_id(self) -> Self {
        self.group_by("token_id")
    }

    pub fn group_by_token_id_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("token_id");
        request.query = request.query.project_expr(alias, Expr::column("token_id"));
        request
    }

    pub fn group_by_token_id_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("token_id")
            .aggregate_with_function("token_id", alias, function)
    }

    pub fn count_token_id(self) -> Self {
        self.count_token_id_as("token_id_count")
    }

    pub fn count_token_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("token_id", alias)
    }

    pub fn sum_token_id(self) -> Self {
        self.sum_token_id_as("sum_token_id")
    }

    pub fn sum_token_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("token_id", alias)
    }

    pub fn avg_token_id(self) -> Self {
        self.avg_token_id_as("avg_token_id")
    }

    pub fn avg_token_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("token_id", alias)
    }

    pub fn min_token_id(self) -> Self {
        self.min_token_id_as("min_token_id")
    }

    pub fn min_token_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("token_id", alias)
    }

    pub fn max_token_id(self) -> Self {
        self.max_token_id_as("max_token_id")
    }

    pub fn max_token_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("token_id", alias)
    }

    pub fn unselect_token_id(mut self) -> Self {
        self.query.projection.retain(|field| field != "token_id");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "token_id");
        self
    }

    pub fn with_token_id(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "token_id",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_token_id_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "token_id",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_token_id_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("token_id", value));
        self
    }

    pub fn with_token_id_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("token_id", value));
        self
    }

    pub fn with_token_id_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("token_id", value));
        self
    }

    pub fn with_token_id_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("token_id", value));
        self
    }

    pub fn with_token_id_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("token_id", value));
        self
    }

    pub fn with_token_id_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("token_id", value));
        self
    }

    pub fn with_token_id_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("token_id", lower, upper));
        self
    }

    pub fn with_token_id_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("token_id", range.start, range.end));
        self
    }

    pub fn with_token_id_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "token_id",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_token_id_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "token_id",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_token_id_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("token_id", value));
        self
    }

    pub fn with_token_id_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("token_id", value));
        self
    }

    pub fn with_token_id_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("token_id", value));
        self
    }

    pub fn with_token_id_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_begin_with("token_id", value));
        self
    }

    pub fn with_token_id_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("token_id", value));
        self
    }

    pub fn with_token_id_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("token_id", value));
        self
    }

    pub fn with_token_id_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("token_id", value));
        self
    }
    pub fn with_token_id_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("token_id", value));
        self
    }

    pub fn with_token_id_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("token_id", value));
        self
    }

    pub fn with_token_id_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("token_id"));
        self
    }

    pub fn with_token_id_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("token_id"));
        self
    }

    pub fn order_by_token_id_asc(mut self) -> Self {
        self.query = self.query.order_asc("token_id");
        self
    }

    pub fn order_by_token_id_desc(mut self) -> Self {
        self.query = self.query.order_desc("token_id");
        self
    }

    pub fn order_by_token_id_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("token_id");
        self
    }

    pub fn order_by_token_id_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("token_id");
        self
    }

    pub fn select_token_hash(mut self) -> Self {
        self.query = self.query.project("token_hash");
        self
    }

    pub fn project_token_hash(self) -> Self {
        self.select_token_hash()
    }

    pub fn select_token_hash_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_token_hash_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_token_hash_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("token_hash", raw_sql_segment));
        self
    }

    pub fn group_by_token_hash(self) -> Self {
        self.group_by("token_hash")
    }

    pub fn group_by_token_hash_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("token_hash");
        request.query = request
            .query
            .project_expr(alias, Expr::column("token_hash"));
        request
    }

    pub fn group_by_token_hash_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("token_hash")
            .aggregate_with_function("token_hash", alias, function)
    }

    pub fn count_token_hash(self) -> Self {
        self.count_token_hash_as("token_hash_count")
    }

    pub fn count_token_hash_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("token_hash", alias)
    }

    pub fn sum_token_hash(self) -> Self {
        self.sum_token_hash_as("sum_token_hash")
    }

    pub fn sum_token_hash_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("token_hash", alias)
    }

    pub fn avg_token_hash(self) -> Self {
        self.avg_token_hash_as("avg_token_hash")
    }

    pub fn avg_token_hash_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("token_hash", alias)
    }

    pub fn min_token_hash(self) -> Self {
        self.min_token_hash_as("min_token_hash")
    }

    pub fn min_token_hash_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("token_hash", alias)
    }

    pub fn max_token_hash(self) -> Self {
        self.max_token_hash_as("max_token_hash")
    }

    pub fn max_token_hash_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("token_hash", alias)
    }

    pub fn unselect_token_hash(mut self) -> Self {
        self.query.projection.retain(|field| field != "token_hash");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "token_hash");
        self
    }

    pub fn with_token_hash(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "token_hash",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_token_hash_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "token_hash",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_token_hash_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("token_hash", value));
        self
    }

    pub fn with_token_hash_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("token_hash", value));
        self
    }

    pub fn with_token_hash_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("token_hash", value));
        self
    }

    pub fn with_token_hash_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("token_hash", value));
        self
    }

    pub fn with_token_hash_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("token_hash", value));
        self
    }

    pub fn with_token_hash_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("token_hash", value));
        self
    }

    pub fn with_token_hash_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("token_hash", lower, upper));
        self
    }

    pub fn with_token_hash_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("token_hash", range.start, range.end));
        self
    }

    pub fn with_token_hash_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "token_hash",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_token_hash_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "token_hash",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_token_hash_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("token_hash", value));
        self
    }

    pub fn with_token_hash_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_contain("token_hash", value));
        self
    }

    pub fn with_token_hash_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("token_hash", value));
        self
    }

    pub fn with_token_hash_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_begin_with("token_hash", value));
        self
    }

    pub fn with_token_hash_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("token_hash", value));
        self
    }

    pub fn with_token_hash_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_end_with("token_hash", value));
        self
    }

    pub fn with_token_hash_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("token_hash", value));
        self
    }
    pub fn with_token_hash_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("token_hash", value));
        self
    }

    pub fn with_token_hash_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("token_hash", value));
        self
    }

    pub fn with_token_hash_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("token_hash"));
        self
    }

    pub fn with_token_hash_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("token_hash"));
        self
    }

    pub fn order_by_token_hash_asc(mut self) -> Self {
        self.query = self.query.order_asc("token_hash");
        self
    }

    pub fn order_by_token_hash_desc(mut self) -> Self {
        self.query = self.query.order_desc("token_hash");
        self
    }

    pub fn order_by_token_hash_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("token_hash");
        self
    }

    pub fn order_by_token_hash_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("token_hash");
        self
    }

    pub fn select_description(mut self) -> Self {
        self.query = self.query.project("description");
        self
    }

    pub fn project_description(self) -> Self {
        self.select_description()
    }

    pub fn select_description_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_description_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_description_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("description", raw_sql_segment));
        self
    }

    pub fn group_by_description(self) -> Self {
        self.group_by("description")
    }

    pub fn group_by_description_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("description");
        request.query = request
            .query
            .project_expr(alias, Expr::column("description"));
        request
    }

    pub fn group_by_description_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("description")
            .aggregate_with_function("description", alias, function)
    }

    pub fn count_description(self) -> Self {
        self.count_description_as("description_count")
    }

    pub fn count_description_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("description", alias)
    }

    pub fn sum_description(self) -> Self {
        self.sum_description_as("sum_description")
    }

    pub fn sum_description_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("description", alias)
    }

    pub fn avg_description(self) -> Self {
        self.avg_description_as("avg_description")
    }

    pub fn avg_description_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("description", alias)
    }

    pub fn min_description(self) -> Self {
        self.min_description_as("min_description")
    }

    pub fn min_description_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("description", alias)
    }

    pub fn max_description(self) -> Self {
        self.max_description_as("max_description")
    }

    pub fn max_description_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("description", alias)
    }

    pub fn unselect_description(mut self) -> Self {
        self.query.projection.retain(|field| field != "description");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "description");
        self
    }

    pub fn with_description(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "description",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_description_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "description",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_description_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("description", value));
        self
    }

    pub fn with_description_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("description", value));
        self
    }

    pub fn with_description_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("description", value));
        self
    }

    pub fn with_description_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("description", value));
        self
    }

    pub fn with_description_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("description", value));
        self
    }

    pub fn with_description_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("description", value));
        self
    }

    pub fn with_description_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("description", lower, upper));
        self
    }

    pub fn with_description_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("description", range.start, range.end));
        self
    }

    pub fn with_description_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "description",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_description_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "description",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_description_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("description", value));
        self
    }

    pub fn with_description_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_contain("description", value));
        self
    }

    pub fn with_description_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::begin_with("description", value));
        self
    }

    pub fn with_description_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_begin_with("description", value));
        self
    }

    pub fn with_description_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("description", value));
        self
    }

    pub fn with_description_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_end_with("description", value));
        self
    }

    pub fn with_description_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::sound_like("description", value));
        self
    }
    pub fn with_description_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("description", value));
        self
    }

    pub fn with_description_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("description", value));
        self
    }

    pub fn with_description_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("description"));
        self
    }

    pub fn with_description_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("description"));
        self
    }

    pub fn order_by_description_asc(mut self) -> Self {
        self.query = self.query.order_asc("description");
        self
    }

    pub fn order_by_description_desc(mut self) -> Self {
        self.query = self.query.order_desc("description");
        self
    }

    pub fn order_by_description_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("description");
        self
    }

    pub fn order_by_description_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("description");
        self
    }

    pub fn select_scopes(mut self) -> Self {
        self.query = self.query.project("scopes");
        self
    }

    pub fn project_scopes(self) -> Self {
        self.select_scopes()
    }

    pub fn select_scopes_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_scopes_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_scopes_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("scopes", raw_sql_segment));
        self
    }

    pub fn group_by_scopes(self) -> Self {
        self.group_by("scopes")
    }

    pub fn group_by_scopes_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("scopes");
        request.query = request.query.project_expr(alias, Expr::column("scopes"));
        request
    }

    pub fn group_by_scopes_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("scopes")
            .aggregate_with_function("scopes", alias, function)
    }

    pub fn count_scopes(self) -> Self {
        self.count_scopes_as("scopes_count")
    }

    pub fn count_scopes_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("scopes", alias)
    }

    pub fn sum_scopes(self) -> Self {
        self.sum_scopes_as("sum_scopes")
    }

    pub fn sum_scopes_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("scopes", alias)
    }

    pub fn avg_scopes(self) -> Self {
        self.avg_scopes_as("avg_scopes")
    }

    pub fn avg_scopes_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("scopes", alias)
    }

    pub fn min_scopes(self) -> Self {
        self.min_scopes_as("min_scopes")
    }

    pub fn min_scopes_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("scopes", alias)
    }

    pub fn max_scopes(self) -> Self {
        self.max_scopes_as("max_scopes")
    }

    pub fn max_scopes_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("scopes", alias)
    }

    pub fn unselect_scopes(mut self) -> Self {
        self.query.projection.retain(|field| field != "scopes");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "scopes");
        self
    }

    pub fn with_scopes(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "scopes",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_scopes_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "scopes",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_scopes_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("scopes", value));
        self
    }

    pub fn with_scopes_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("scopes", value));
        self
    }

    pub fn with_scopes_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("scopes", value));
        self
    }

    pub fn with_scopes_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("scopes", value));
        self
    }

    pub fn with_scopes_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("scopes", value));
        self
    }

    pub fn with_scopes_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("scopes", value));
        self
    }

    pub fn with_scopes_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("scopes", lower, upper));
        self
    }

    pub fn with_scopes_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("scopes", range.start, range.end));
        self
    }

    pub fn with_scopes_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::in_list("scopes", values.into_iter().map(Into::into)));
        self
    }

    pub fn with_scopes_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "scopes",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_scopes_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("scopes", value));
        self
    }

    pub fn with_scopes_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("scopes", value));
        self
    }

    pub fn with_scopes_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("scopes", value));
        self
    }

    pub fn with_scopes_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("scopes", value));
        self
    }

    pub fn with_scopes_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("scopes", value));
        self
    }

    pub fn with_scopes_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("scopes", value));
        self
    }

    pub fn with_scopes_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("scopes", value));
        self
    }
    pub fn with_scopes_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("scopes", value));
        self
    }

    pub fn with_scopes_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("scopes", value));
        self
    }

    pub fn with_scopes_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("scopes"));
        self
    }

    pub fn with_scopes_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("scopes"));
        self
    }

    pub fn order_by_scopes_asc(mut self) -> Self {
        self.query = self.query.order_asc("scopes");
        self
    }

    pub fn order_by_scopes_desc(mut self) -> Self {
        self.query = self.query.order_desc("scopes");
        self
    }

    pub fn order_by_scopes_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("scopes");
        self
    }

    pub fn order_by_scopes_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("scopes");
        self
    }

    pub fn select_created_at(mut self) -> Self {
        self.query = self.query.project("created_at");
        self
    }

    pub fn project_created_at(self) -> Self {
        self.select_created_at()
    }

    pub fn select_created_at_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_created_at_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_created_at_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("created_at", raw_sql_segment));
        self
    }

    pub fn group_by_created_at(self) -> Self {
        self.group_by("created_at")
    }

    pub fn group_by_created_at_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("created_at");
        request.query = request
            .query
            .project_expr(alias, Expr::column("created_at"));
        request
    }

    pub fn group_by_created_at_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("created_at")
            .aggregate_with_function("created_at", alias, function)
    }

    pub fn count_created_at(self) -> Self {
        self.count_created_at_as("created_at_count")
    }

    pub fn count_created_at_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("created_at", alias)
    }

    pub fn sum_created_at(self) -> Self {
        self.sum_created_at_as("sum_created_at")
    }

    pub fn sum_created_at_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("created_at", alias)
    }

    pub fn avg_created_at(self) -> Self {
        self.avg_created_at_as("avg_created_at")
    }

    pub fn avg_created_at_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("created_at", alias)
    }

    pub fn min_created_at(self) -> Self {
        self.min_created_at_as("min_created_at")
    }

    pub fn min_created_at_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("created_at", alias)
    }

    pub fn max_created_at(self) -> Self {
        self.max_created_at_as("max_created_at")
    }

    pub fn max_created_at_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("created_at", alias)
    }

    pub fn unselect_created_at(mut self) -> Self {
        self.query.projection.retain(|field| field != "created_at");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "created_at");
        self
    }

    pub fn with_created_at(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "created_at",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_created_at_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "created_at",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_created_at_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("created_at", value));
        self
    }

    pub fn with_created_at_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("created_at", value));
        self
    }

    pub fn with_created_at_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("created_at", value));
        self
    }

    pub fn with_created_at_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("created_at", value));
        self
    }

    pub fn with_created_at_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("created_at", value));
        self
    }

    pub fn with_created_at_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("created_at", value));
        self
    }

    pub fn with_created_at_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("created_at", lower, upper));
        self
    }

    pub fn with_created_at_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("created_at", range.start, range.end));
        self
    }

    pub fn with_created_at_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "created_at",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_created_at_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "created_at",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_created_at_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("created_at", value));
        self
    }

    pub fn with_created_at_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("created_at", value));
        self
    }

    pub fn with_created_at_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("created_at"));
        self
    }

    pub fn with_created_at_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("created_at"));
        self
    }

    pub fn order_by_created_at_asc(mut self) -> Self {
        self.query = self.query.order_asc("created_at");
        self
    }

    pub fn order_by_created_at_desc(mut self) -> Self {
        self.query = self.query.order_desc("created_at");
        self
    }

    pub fn order_by_created_at_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("created_at");
        self
    }

    pub fn order_by_created_at_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("created_at");
        self
    }

    pub fn select_expires_at_epoch_millis(mut self) -> Self {
        self.query = self.query.project("expires_at_epoch_millis");
        self
    }

    pub fn project_expires_at_epoch_millis(self) -> Self {
        self.select_expires_at_epoch_millis()
    }

    pub fn select_expires_at_epoch_millis_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_expires_at_epoch_millis_unsafe_raw(UnsafeRawSqlSegment::trusted(
            raw_sql_segment,
        ))
    }

    pub fn select_expires_at_epoch_millis_unsafe_raw(
        mut self,
        raw_sql_segment: UnsafeRawSqlSegment,
    ) -> Self {
        self.query_options.raw_projections.push(RawProjection::new(
            "expires_at_epoch_millis",
            raw_sql_segment,
        ));
        self
    }

    pub fn select_expires_at_epoch_millis_with_function(self, function: AggregateFunction) -> Self {
        self.select_expires_at_epoch_millis_as_with_function("expires_at_epoch_millis", function)
    }

    pub fn select_expires_at_epoch_millis_as_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.aggregate_with_function("expires_at_epoch_millis", alias, function)
    }

    pub fn group_by_expires_at_epoch_millis(self) -> Self {
        self.group_by("expires_at_epoch_millis")
    }

    pub fn group_by_expires_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("expires_at_epoch_millis");
        request.query = request
            .query
            .project_expr(alias, Expr::column("expires_at_epoch_millis"));
        request
    }

    pub fn group_by_expires_at_epoch_millis_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("expires_at_epoch_millis")
            .aggregate_with_function("expires_at_epoch_millis", alias, function)
    }

    pub fn count_expires_at_epoch_millis(self) -> Self {
        self.count_expires_at_epoch_millis_as("expires_at_epoch_millis_count")
    }

    pub fn count_expires_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("expires_at_epoch_millis", alias)
    }

    pub fn sum_expires_at_epoch_millis(self) -> Self {
        self.sum_expires_at_epoch_millis_as("sum_expires_at_epoch_millis")
    }

    pub fn sum_expires_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("expires_at_epoch_millis", alias)
    }

    pub fn avg_expires_at_epoch_millis(self) -> Self {
        self.avg_expires_at_epoch_millis_as("avg_expires_at_epoch_millis")
    }

    pub fn avg_expires_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("expires_at_epoch_millis", alias)
    }

    pub fn min_expires_at_epoch_millis(self) -> Self {
        self.min_expires_at_epoch_millis_as("min_expires_at_epoch_millis")
    }

    pub fn min_expires_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("expires_at_epoch_millis", alias)
    }

    pub fn max_expires_at_epoch_millis(self) -> Self {
        self.max_expires_at_epoch_millis_as("max_expires_at_epoch_millis")
    }

    pub fn max_expires_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("expires_at_epoch_millis", alias)
    }

    pub fn standard_deviation_expires_at_epoch_millis(self) -> Self {
        self.standard_deviation_expires_at_epoch_millis_as("stdDev_expires_at_epoch_millis")
    }

    pub fn standard_deviation_expires_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_stddev("expires_at_epoch_millis", alias)
    }

    pub fn square_root_of_population_standard_deviation_expires_at_epoch_millis(self) -> Self {
        self.square_root_of_population_standard_deviation_expires_at_epoch_millis_as(
            "stdDevPop_expires_at_epoch_millis",
        )
    }

    pub fn square_root_of_population_standard_deviation_expires_at_epoch_millis_as(
        self,
        alias: impl Into<String>,
    ) -> Self {
        self.aggregate_stddev_pop("expires_at_epoch_millis", alias)
    }

    pub fn sample_variance_expires_at_epoch_millis(self) -> Self {
        self.sample_variance_expires_at_epoch_millis_as("varSamp_expires_at_epoch_millis")
    }

    pub fn sample_variance_expires_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_var_samp("expires_at_epoch_millis", alias)
    }

    pub fn sample_population_variance_expires_at_epoch_millis(self) -> Self {
        self.sample_population_variance_expires_at_epoch_millis_as("varPop_expires_at_epoch_millis")
    }

    pub fn sample_population_variance_expires_at_epoch_millis_as(
        self,
        alias: impl Into<String>,
    ) -> Self {
        self.aggregate_var_pop("expires_at_epoch_millis", alias)
    }

    pub fn unselect_expires_at_epoch_millis(mut self) -> Self {
        self.query
            .projection
            .retain(|field| field != "expires_at_epoch_millis");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "expires_at_epoch_millis");
        self
    }

    pub fn with_expires_at_epoch_millis(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "expires_at_epoch_millis",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_expires_at_epoch_millis_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "expires_at_epoch_millis",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_expires_at_epoch_millis_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::eq("expires_at_epoch_millis", value));
        self
    }

    pub fn with_expires_at_epoch_millis_is_not(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::ne("expires_at_epoch_millis", value));
        self
    }

    pub fn with_expires_at_epoch_millis_greater_than(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::gt("expires_at_epoch_millis", value));
        self
    }

    pub fn with_expires_at_epoch_millis_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::gte("expires_at_epoch_millis", value));
        self
    }

    pub fn with_expires_at_epoch_millis_less_than(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::lt("expires_at_epoch_millis", value));
        self
    }

    pub fn with_expires_at_epoch_millis_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::lte("expires_at_epoch_millis", value));
        self
    }

    pub fn with_expires_at_epoch_millis_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("expires_at_epoch_millis", lower, upper));
        self
    }

    pub fn with_expires_at_epoch_millis_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "expires_at_epoch_millis",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_expires_at_epoch_millis_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "expires_at_epoch_millis",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_expires_at_epoch_millis_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "expires_at_epoch_millis",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_expires_at_epoch_millis_before(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::lt("expires_at_epoch_millis", value));
        self
    }

    pub fn with_expires_at_epoch_millis_after(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::gt("expires_at_epoch_millis", value));
        self
    }

    pub fn with_expires_at_epoch_millis_is_unknown(mut self) -> Self {
        self.query = self
            .query
            .and_filter(Expr::is_null("expires_at_epoch_millis"));
        self
    }

    pub fn with_expires_at_epoch_millis_is_known(mut self) -> Self {
        self.query = self
            .query
            .and_filter(Expr::is_not_null("expires_at_epoch_millis"));
        self
    }

    pub fn order_by_expires_at_epoch_millis_asc(mut self) -> Self {
        self.query = self.query.order_asc("expires_at_epoch_millis");
        self
    }

    pub fn order_by_expires_at_epoch_millis_desc(mut self) -> Self {
        self.query = self.query.order_desc("expires_at_epoch_millis");
        self
    }

    pub fn order_by_expires_at_epoch_millis_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("expires_at_epoch_millis");
        self
    }

    pub fn order_by_expires_at_epoch_millis_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("expires_at_epoch_millis");
        self
    }

    pub fn select_revoked(mut self) -> Self {
        self.query = self.query.project("revoked");
        self
    }

    pub fn project_revoked(self) -> Self {
        self.select_revoked()
    }

    pub fn select_revoked_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_revoked_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_revoked_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("revoked", raw_sql_segment));
        self
    }

    pub fn group_by_revoked(self) -> Self {
        self.group_by("revoked")
    }

    pub fn group_by_revoked_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("revoked");
        request.query = request.query.project_expr(alias, Expr::column("revoked"));
        request
    }

    pub fn group_by_revoked_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("revoked")
            .aggregate_with_function("revoked", alias, function)
    }

    pub fn count_revoked(self) -> Self {
        self.count_revoked_as("revoked_count")
    }

    pub fn count_revoked_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("revoked", alias)
    }

    pub fn sum_revoked(self) -> Self {
        self.sum_revoked_as("sum_revoked")
    }

    pub fn sum_revoked_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("revoked", alias)
    }

    pub fn avg_revoked(self) -> Self {
        self.avg_revoked_as("avg_revoked")
    }

    pub fn avg_revoked_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("revoked", alias)
    }

    pub fn min_revoked(self) -> Self {
        self.min_revoked_as("min_revoked")
    }

    pub fn min_revoked_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("revoked", alias)
    }

    pub fn max_revoked(self) -> Self {
        self.max_revoked_as("max_revoked")
    }

    pub fn max_revoked_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("revoked", alias)
    }

    pub fn unselect_revoked(mut self) -> Self {
        self.query.projection.retain(|field| field != "revoked");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "revoked");
        self
    }

    pub fn which_are_revoked(mut self) -> Self {
        self.query = self.query.and_filter(Expr::eq("revoked", true));
        self
    }

    pub fn which_are_not_revoked(mut self) -> Self {
        self.query = self.query.and_filter(Expr::eq("revoked", false));
        self
    }

    pub fn with_revoked_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("revoked"));
        self
    }

    pub fn with_revoked_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("revoked"));
        self
    }
    pub fn order_by_revoked_asc(mut self) -> Self {
        self.query = self.query.order_asc("revoked");
        self
    }

    pub fn order_by_revoked_desc(mut self) -> Self {
        self.query = self.query.order_desc("revoked");
        self
    }

    pub fn order_by_revoked_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("revoked");
        self
    }

    pub fn order_by_revoked_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("revoked");
        self
    }

    pub fn select_revoked_at_epoch_millis(mut self) -> Self {
        self.query = self.query.project("revoked_at_epoch_millis");
        self
    }

    pub fn project_revoked_at_epoch_millis(self) -> Self {
        self.select_revoked_at_epoch_millis()
    }

    pub fn select_revoked_at_epoch_millis_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_revoked_at_epoch_millis_unsafe_raw(UnsafeRawSqlSegment::trusted(
            raw_sql_segment,
        ))
    }

    pub fn select_revoked_at_epoch_millis_unsafe_raw(
        mut self,
        raw_sql_segment: UnsafeRawSqlSegment,
    ) -> Self {
        self.query_options.raw_projections.push(RawProjection::new(
            "revoked_at_epoch_millis",
            raw_sql_segment,
        ));
        self
    }

    pub fn select_revoked_at_epoch_millis_with_function(self, function: AggregateFunction) -> Self {
        self.select_revoked_at_epoch_millis_as_with_function("revoked_at_epoch_millis", function)
    }

    pub fn select_revoked_at_epoch_millis_as_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.aggregate_with_function("revoked_at_epoch_millis", alias, function)
    }

    pub fn group_by_revoked_at_epoch_millis(self) -> Self {
        self.group_by("revoked_at_epoch_millis")
    }

    pub fn group_by_revoked_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("revoked_at_epoch_millis");
        request.query = request
            .query
            .project_expr(alias, Expr::column("revoked_at_epoch_millis"));
        request
    }

    pub fn group_by_revoked_at_epoch_millis_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("revoked_at_epoch_millis")
            .aggregate_with_function("revoked_at_epoch_millis", alias, function)
    }

    pub fn count_revoked_at_epoch_millis(self) -> Self {
        self.count_revoked_at_epoch_millis_as("revoked_at_epoch_millis_count")
    }

    pub fn count_revoked_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("revoked_at_epoch_millis", alias)
    }

    pub fn sum_revoked_at_epoch_millis(self) -> Self {
        self.sum_revoked_at_epoch_millis_as("sum_revoked_at_epoch_millis")
    }

    pub fn sum_revoked_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("revoked_at_epoch_millis", alias)
    }

    pub fn avg_revoked_at_epoch_millis(self) -> Self {
        self.avg_revoked_at_epoch_millis_as("avg_revoked_at_epoch_millis")
    }

    pub fn avg_revoked_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("revoked_at_epoch_millis", alias)
    }

    pub fn min_revoked_at_epoch_millis(self) -> Self {
        self.min_revoked_at_epoch_millis_as("min_revoked_at_epoch_millis")
    }

    pub fn min_revoked_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("revoked_at_epoch_millis", alias)
    }

    pub fn max_revoked_at_epoch_millis(self) -> Self {
        self.max_revoked_at_epoch_millis_as("max_revoked_at_epoch_millis")
    }

    pub fn max_revoked_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("revoked_at_epoch_millis", alias)
    }

    pub fn standard_deviation_revoked_at_epoch_millis(self) -> Self {
        self.standard_deviation_revoked_at_epoch_millis_as("stdDev_revoked_at_epoch_millis")
    }

    pub fn standard_deviation_revoked_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_stddev("revoked_at_epoch_millis", alias)
    }

    pub fn square_root_of_population_standard_deviation_revoked_at_epoch_millis(self) -> Self {
        self.square_root_of_population_standard_deviation_revoked_at_epoch_millis_as(
            "stdDevPop_revoked_at_epoch_millis",
        )
    }

    pub fn square_root_of_population_standard_deviation_revoked_at_epoch_millis_as(
        self,
        alias: impl Into<String>,
    ) -> Self {
        self.aggregate_stddev_pop("revoked_at_epoch_millis", alias)
    }

    pub fn sample_variance_revoked_at_epoch_millis(self) -> Self {
        self.sample_variance_revoked_at_epoch_millis_as("varSamp_revoked_at_epoch_millis")
    }

    pub fn sample_variance_revoked_at_epoch_millis_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_var_samp("revoked_at_epoch_millis", alias)
    }

    pub fn sample_population_variance_revoked_at_epoch_millis(self) -> Self {
        self.sample_population_variance_revoked_at_epoch_millis_as("varPop_revoked_at_epoch_millis")
    }

    pub fn sample_population_variance_revoked_at_epoch_millis_as(
        self,
        alias: impl Into<String>,
    ) -> Self {
        self.aggregate_var_pop("revoked_at_epoch_millis", alias)
    }

    pub fn unselect_revoked_at_epoch_millis(mut self) -> Self {
        self.query
            .projection
            .retain(|field| field != "revoked_at_epoch_millis");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "revoked_at_epoch_millis");
        self
    }

    pub fn with_revoked_at_epoch_millis(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "revoked_at_epoch_millis",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_revoked_at_epoch_millis_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "revoked_at_epoch_millis",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_revoked_at_epoch_millis_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::eq("revoked_at_epoch_millis", value));
        self
    }

    pub fn with_revoked_at_epoch_millis_is_not(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::ne("revoked_at_epoch_millis", value));
        self
    }

    pub fn with_revoked_at_epoch_millis_greater_than(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::gt("revoked_at_epoch_millis", value));
        self
    }

    pub fn with_revoked_at_epoch_millis_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::gte("revoked_at_epoch_millis", value));
        self
    }

    pub fn with_revoked_at_epoch_millis_less_than(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::lt("revoked_at_epoch_millis", value));
        self
    }

    pub fn with_revoked_at_epoch_millis_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::lte("revoked_at_epoch_millis", value));
        self
    }

    pub fn with_revoked_at_epoch_millis_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("revoked_at_epoch_millis", lower, upper));
        self
    }

    pub fn with_revoked_at_epoch_millis_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "revoked_at_epoch_millis",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_revoked_at_epoch_millis_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "revoked_at_epoch_millis",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_revoked_at_epoch_millis_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "revoked_at_epoch_millis",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_revoked_at_epoch_millis_before(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::lt("revoked_at_epoch_millis", value));
        self
    }

    pub fn with_revoked_at_epoch_millis_after(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::gt("revoked_at_epoch_millis", value));
        self
    }

    pub fn with_revoked_at_epoch_millis_is_unknown(mut self) -> Self {
        self.query = self
            .query
            .and_filter(Expr::is_null("revoked_at_epoch_millis"));
        self
    }

    pub fn with_revoked_at_epoch_millis_is_known(mut self) -> Self {
        self.query = self
            .query
            .and_filter(Expr::is_not_null("revoked_at_epoch_millis"));
        self
    }

    pub fn order_by_revoked_at_epoch_millis_asc(mut self) -> Self {
        self.query = self.query.order_asc("revoked_at_epoch_millis");
        self
    }

    pub fn order_by_revoked_at_epoch_millis_desc(mut self) -> Self {
        self.query = self.query.order_desc("revoked_at_epoch_millis");
        self
    }

    pub fn order_by_revoked_at_epoch_millis_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("revoked_at_epoch_millis");
        self
    }

    pub fn order_by_revoked_at_epoch_millis_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("revoked_at_epoch_millis");
        self
    }

    pub fn group_by_version(self) -> Self {
        self.group_by("version")
    }

    pub fn group_by_version_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("version");
        request.query = request.query.project_expr(alias, Expr::column("version"));
        request
    }

    pub fn group_by_version_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("version")
            .aggregate_with_function("version", alias, function)
    }

    pub fn count_version(self) -> Self {
        self.count_version_as("version_count")
    }

    pub fn count_version_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("version", alias)
    }

    pub fn sum_version(self) -> Self {
        self.sum_version_as("sum_version")
    }

    pub fn sum_version_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("version", alias)
    }

    pub fn avg_version(self) -> Self {
        self.avg_version_as("avg_version")
    }

    pub fn avg_version_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("version", alias)
    }

    pub fn min_version(self) -> Self {
        self.min_version_as("min_version")
    }

    pub fn min_version_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("version", alias)
    }

    pub fn max_version(self) -> Self {
        self.max_version_as("max_version")
    }

    pub fn max_version_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("version", alias)
    }

    pub fn order_by_version_asc(mut self) -> Self {
        self.query = self.query.order_asc("version");
        self
    }

    pub fn order_by_version_desc(mut self) -> Self {
        self.query = self.query.order_desc("version");
        self
    }

    pub fn order_by_version_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("version");
        self
    }

    pub fn order_by_version_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("version");
        self
    }
    pub fn filter_by_tenant(mut self, value: impl EntityReference) -> Self {
        self.query = self
            .query
            .and_filter(Expr::eq("tenant_id", value.entity_id_value()));
        self
    }

    pub fn with_tenant_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "tenant_id",
            <crate::Tenant as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters
            .push(RelationFilter::new("tenant", selection));
        self
    }

    pub fn without_tenant_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "tenant_id",
            <crate::Tenant as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters
            .push(RelationFilter::new("tenant", selection));
        self
    }

    pub fn have_tenant(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("tenant_id"));
        self
    }

    pub fn have_no_tenant(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("tenant_id"));
        self
    }

    pub fn group_by_tenant(self) -> Self {
        self.group_by("tenant_id")
    }

    pub fn group_by_tenant_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("tenant_id");
        request.query = request.query.project_expr(alias, Expr::column("tenant_id"));
        request
    }

    pub fn group_by_tenant_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("tenant_id")
            .aggregate_with_function("tenant_id", alias, function)
    }

    pub fn group_by_tenant_with(mut self, request: impl Into<QuerySelection>) -> Self {
        self.query = self.query.group_by("tenant_id");
        self.query_options.object_group_bys.push(ObjectGroupBy::new(
            "tenant",
            "tenant_id",
            request,
        ));
        self
    }

    pub fn group_by_tenant_with_details(self) -> Self {
        self.group_by_tenant_with_details_from(crate::Q::tenants().unlimited())
    }

    pub fn group_by_tenant_with_details_from(self, request: impl Into<QuerySelection>) -> Self {
        self.group_by_tenant_with(request)
    }

    pub fn roll_up_to_tenant(self) -> Self {
        self.roll_up_to_tenant_with(crate::Q::tenants().unlimited())
    }

    pub fn roll_up_to_tenant_with(self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.with_tenant_matching(selection.clone())
            .group_by_tenant_with(selection)
    }

    pub fn count_tenant(self) -> Self {
        self.count_tenant_as("tenant_count")
    }

    pub fn count_tenant_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("tenant_id", alias)
    }

    pub fn unselect_tenant(mut self) -> Self {
        self.query.projection.retain(|field| field != "tenant_id");
        self.query
            .relations
            .retain(|relation| relation.name != "tenant");
        self
    }

    pub fn filter_by_security_user(mut self, value: impl EntityReference) -> Self {
        self.query = self
            .query
            .and_filter(Expr::eq("security_user_id", value.entity_id_value()));
        self
    }

    pub fn with_security_user_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "security_user_id",
            <crate::SecurityUser as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters
            .push(RelationFilter::new("security_user", selection));
        self
    }

    pub fn without_security_user_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "security_user_id",
            <crate::SecurityUser as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters
            .push(RelationFilter::new("security_user", selection));
        self
    }

    pub fn have_security_user(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("security_user_id"));
        self
    }

    pub fn have_no_security_user(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("security_user_id"));
        self
    }

    pub fn group_by_security_user(self) -> Self {
        self.group_by("security_user_id")
    }

    pub fn group_by_security_user_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("security_user_id");
        request.query = request
            .query
            .project_expr(alias, Expr::column("security_user_id"));
        request
    }

    pub fn group_by_security_user_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("security_user_id").aggregate_with_function(
            "security_user_id",
            alias,
            function,
        )
    }

    pub fn group_by_security_user_with(mut self, request: impl Into<QuerySelection>) -> Self {
        self.query = self.query.group_by("security_user_id");
        self.query_options.object_group_bys.push(ObjectGroupBy::new(
            "security_user",
            "security_user_id",
            request,
        ));
        self
    }

    pub fn group_by_security_user_with_details(self) -> Self {
        self.group_by_security_user_with_details_from(crate::Q::security_users().unlimited())
    }

    pub fn group_by_security_user_with_details_from(
        self,
        request: impl Into<QuerySelection>,
    ) -> Self {
        self.group_by_security_user_with(request)
    }

    pub fn roll_up_to_security_user(self) -> Self {
        self.roll_up_to_security_user_with(crate::Q::security_users().unlimited())
    }

    pub fn roll_up_to_security_user_with(self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.with_security_user_matching(selection.clone())
            .group_by_security_user_with(selection)
    }

    pub fn count_security_user(self) -> Self {
        self.count_security_user_as("security_user_count")
    }

    pub fn count_security_user_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("security_user_id", alias)
    }

    pub fn unselect_security_user(mut self) -> Self {
        self.query
            .projection
            .retain(|field| field != "security_user_id");
        self.query
            .relations
            .retain(|relation| relation.name != "security_user");
        self
    }
    pub fn select_tenant(mut self) -> Self {
        self.query = self.query.relation("tenant");
        self
    }

    pub fn select_tenant_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("tenant", selection.into_query());
        self
    }

    pub fn facet_by_tenant_as(
        self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
    ) -> Self {
        self.facet_by_tenant_as_with_options(facet_name, request, true)
    }

    pub fn facet_by_tenant_as_with_options(
        mut self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
        include_all_facets: bool,
    ) -> Self {
        self.query_options.facets.push(FacetRequest::new(
            facet_name,
            "tenant",
            request,
            include_all_facets,
        ));
        self
    }

    pub fn select_security_user(mut self) -> Self {
        self.query = self.query.relation("security_user");
        self
    }

    pub fn select_security_user_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self
            .query
            .relation_query("security_user", selection.into_query());
        self
    }

    pub fn facet_by_security_user_as(
        self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
    ) -> Self {
        self.facet_by_security_user_as_with_options(facet_name, request, true)
    }

    pub fn facet_by_security_user_as_with_options(
        mut self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
        include_all_facets: bool,
    ) -> Self {
        self.query_options.facets.push(FacetRequest::new(
            facet_name,
            "security_user",
            request,
            include_all_facets,
        ));
        self
    }
}

impl<R> Default for PersonalAccessTokenRequest<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> From<PersonalAccessTokenRequest<R>> for SelectQuery {
    fn from(request: PersonalAccessTokenRequest<R>) -> Self {
        QuerySelection::from(request).into_query()
    }
}

impl<R> From<PersonalAccessTokenRequest<R>> for QuerySelection {
    fn from(request: PersonalAccessTokenRequest<R>) -> Self {
        Self {
            query: request.query,
            relation_selections: request.relation_selections,
            relation_filters: request.relation_filters,
            child_enhancements: request.child_enhancements,
            query_options: request.query_options,
        }
    }
}

impl<'a, C> crate::request_support::AuditedSave<'a, C>
    for teaql_core::Audited<crate::PersonalAccessToken>
where
    C: crate::request_support::TeaqlRepositoryProvider + ?Sized + 'a,
{
    type Error = crate::TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>;
    type Entity = crate::PersonalAccessToken;
    fn save(
        self,
        context: &'a C,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Entity, Self::Error>> + '_>>
    {
        Box::pin(async move {
            teaql_runtime::save_audited_ledger_entity(self, context.user_context())
                .await
                .map_err(DataServiceError::Runtime)
        })
    }
}

impl<R: teaql_core::Entity> crate::PurposedQuery<PersonalAccessTokenRequest<R>> {
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.inner.query_options.comment = Some(comment.into());
        self
    }

    pub fn new_entity<C>(&self, context: &C) -> crate::PersonalAccessToken
    where
        C: crate::TeaqlRuntime + ?Sized,
    {
        self.require_comment();
        let mut entity =
            crate::PersonalAccessToken::runtime_new(context.user_context().entity_runtime_state());
        if let Ok(id) = context
            .user_context()
            .next_id(crate::PersonalAccessToken::ENTITY_NAME)
        {
            entity.update_id(id);
        }
        teaql_core::Entity::mark_as_new(&mut entity);
        entity
    }

    fn into_inner_with_trace(mut self) -> PersonalAccessTokenRequest<R> {
        self.require_comment();
        self.inner
            .query
            .trace_chain
            .push(teaql_core::TraceNode::typed(
                teaql_core::TraceKind::Purpose,
                self.inner.query.entity.clone(),
                None,
                self.purpose,
            ));
        self.inner
    }

    fn require_comment(&self) {
        assert!(
            self.inner
                .query_options
                .comment
                .as_deref()
                .is_some_and(|comment| !comment.trim().is_empty()),
            "query comment must not be empty"
        );
    }

    pub async fn execute_for_page<'a, C>(
        self,
        context: &'a C,
        offset: u64,
        limit: u64,
    ) -> Result<
        teaql_core::SmartList<R>,
        crate::request_support::TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>,
    >
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()
            ._execute_for_page(context, offset, limit)
            .await
    }

    pub async fn execute_for_exists<'a, C>(
        self,
        context: &'a C,
    ) -> Result<
        bool,
        crate::request_support::TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>,
    >
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()
            ._execute_for_exists(context)
            .await
    }

    pub async fn execute_for_list<'a, C>(
        self,
        context: &'a C,
    ) -> Result<
        teaql_core::SmartList<R>,
        crate::request_support::TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>,
    >
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()
            ._execute_for_list(context)
            .await
    }

    pub async fn execute_for_rows<'a, C>(
        self,
        context: &'a C,
    ) -> Result<
        teaql_core::SmartList<teaql_core::CompactRow>,
        crate::request_support::TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>,
    >
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()
            ._execute_for_rows(context)
            .await
    }

    /// Execute query as a lazy entity stream without materializing the result set.
    /// Set chunk size via .stream(chunk_size) or .stream_default() on the query.
    pub async fn execute_for_stream<'a, C>(
        self,
        context: &'a C,
    ) -> Result<
        crate::request_support::TeaqlEntityStream<
            'a,
            R,
            crate::request_support::TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>,
        >,
        crate::request_support::TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>,
    >
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity + 'a,
    {
        self.into_inner_with_trace()
            ._execute_for_stream(context)
            .await
    }

    pub async fn execute_for_first<'a, C>(
        self,
        context: &'a C,
    ) -> Result<
        Option<R>,
        crate::request_support::TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>,
    >
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()
            ._execute_for_first(context)
            .await
    }

    pub async fn execute_for_one<'a, C>(
        self,
        context: &'a C,
    ) -> Result<
        Option<R>,
        crate::request_support::TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>,
    >
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_one(context).await
    }

    pub async fn execute_for_count<'a, C>(
        self,
        context: &'a C,
    ) -> Result<
        u64,
        crate::request_support::TeaqlDataServiceError<C::PersonalAccessTokenRepository<'a>>,
    >
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()
            ._execute_for_count(context)
            .await
    }
}
