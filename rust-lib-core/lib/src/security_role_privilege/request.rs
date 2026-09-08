use std::marker::PhantomData;

use serde_json::Value as JsonValue;
use teaql_core::{Aggregate, AggregateFunction, EntityDescriptor, Expr, SelectQuery, SmartList};
use teaql_runtime::{DataServiceError, RuntimeError};

use crate::request_support::*;

impl EntityReference for crate::SecurityRolePrivilege {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(&self)
    }
}

impl EntityReference for &crate::SecurityRolePrivilege {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(self)
    }
}

// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/security_role_privilege
#[derive(Debug)]
pub struct SecurityRolePrivilegeRequest<R = crate::SecurityRolePrivilege> {
    query: SelectQuery,
    relation_selections: Vec<RelationSelection>,
    relation_filters: Vec<RelationFilter>,
    child_enhancements: Vec<QuerySelection>,
    query_options: QueryOptions,
    marker: PhantomData<R>,
}

impl<R> Clone for SecurityRolePrivilegeRequest<R> {
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

impl<R> SecurityRolePrivilegeRequest<R> {
    pub(crate) fn new() -> Self {
        Self {
            query: SelectQuery::new("SecurityRolePrivilege")
                .project("id")
                .project("version"),
            relation_selections: Vec::new(),
            relation_filters: Vec::new(),
            child_enhancements: Vec::new(),
            query_options: QueryOptions::default(),
            marker: PhantomData,
        }
    }

    pub fn return_type<T>(self) -> SecurityRolePrivilegeRequest<T> {
        SecurityRolePrivilegeRequest {
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
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        let repository = context
            .security_role_privilege_repository()
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
        TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>,
    >
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = context
            .security_role_privilege_repository()
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
        TeaqlEntityStream<'a, R, TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>>,
        TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>,
    >
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity + 'a,
    {
        Ok(Box::pin(async_stream::try_stream! {
            use futures_util::StreamExt;
            let repository = context
                .security_role_privilege_repository()
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
    ) -> Result<Option<R>, TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>>
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
    ) -> Result<Option<R>, TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>>
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
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>>
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
    ) -> Result<u64, TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = context
            .security_role_privilege_repository()
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
                    "count result for SecurityRolePrivilege is missing or not numeric"
                )))
            })
    }

    pub(crate) async fn _execute_for_exists<'a, C>(
        self,
        context: &'a C,
    ) -> Result<bool, TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = context
            .security_role_privilege_repository()
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
            "version" => Some("version"),
            "tenant" | "tenant_id" => Some("tenant_id"),
            "security_role" | "security_role_id" => Some("security_role_id"),
            "security_privilege" | "security_privilege_id" => Some("security_privilege_id"),
            _ => None,
        }
    }

    fn apply_dynamic_json_chain_filter(self, head: &str, tail: &str, value: &JsonValue) -> Self {
        let _ = (tail, value);
        match head {
            "tenant" => self.with_tenant_matching(
                crate::Q::tenants_minimal().apply_dynamic_json_filter(tail, value),
            ),
            "security_role" => self.with_security_role_matching(
                crate::Q::security_roles_minimal().apply_dynamic_json_filter(tail, value),
            ),
            "security_privilege" => self.with_security_privilege_matching(
                crate::Q::security_privileges_minimal().apply_dynamic_json_filter(tail, value),
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
        self.query = self.query.project("version");
        self.query = self.query.project("tenant_id");
        self.query = self.query.project("security_role_id");
        self.query = self.query.project("security_privilege_id");
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
        request = request.select_security_role();
        request = request.select_security_privilege();
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

    pub fn filter_by_security_role(mut self, value: impl EntityReference) -> Self {
        self.query = self
            .query
            .and_filter(Expr::eq("security_role_id", value.entity_id_value()));
        self
    }

    pub fn with_security_role_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "security_role_id",
            <crate::SecurityRole as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters
            .push(RelationFilter::new("security_role", selection));
        self
    }

    pub fn without_security_role_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "security_role_id",
            <crate::SecurityRole as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters
            .push(RelationFilter::new("security_role", selection));
        self
    }

    pub fn have_security_role(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("security_role_id"));
        self
    }

    pub fn have_no_security_role(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("security_role_id"));
        self
    }

    pub fn group_by_security_role(self) -> Self {
        self.group_by("security_role_id")
    }

    pub fn group_by_security_role_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("security_role_id");
        request.query = request
            .query
            .project_expr(alias, Expr::column("security_role_id"));
        request
    }

    pub fn group_by_security_role_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("security_role_id").aggregate_with_function(
            "security_role_id",
            alias,
            function,
        )
    }

    pub fn group_by_security_role_with(mut self, request: impl Into<QuerySelection>) -> Self {
        self.query = self.query.group_by("security_role_id");
        self.query_options.object_group_bys.push(ObjectGroupBy::new(
            "security_role",
            "security_role_id",
            request,
        ));
        self
    }

    pub fn group_by_security_role_with_details(self) -> Self {
        self.group_by_security_role_with_details_from(crate::Q::security_roles().unlimited())
    }

    pub fn group_by_security_role_with_details_from(
        self,
        request: impl Into<QuerySelection>,
    ) -> Self {
        self.group_by_security_role_with(request)
    }

    pub fn roll_up_to_security_role(self) -> Self {
        self.roll_up_to_security_role_with(crate::Q::security_roles().unlimited())
    }

    pub fn roll_up_to_security_role_with(self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.with_security_role_matching(selection.clone())
            .group_by_security_role_with(selection)
    }

    pub fn count_security_role(self) -> Self {
        self.count_security_role_as("security_role_count")
    }

    pub fn count_security_role_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("security_role_id", alias)
    }

    pub fn unselect_security_role(mut self) -> Self {
        self.query
            .projection
            .retain(|field| field != "security_role_id");
        self.query
            .relations
            .retain(|relation| relation.name != "security_role");
        self
    }

    pub fn filter_by_security_privilege(mut self, value: impl EntityReference) -> Self {
        self.query = self
            .query
            .and_filter(Expr::eq("security_privilege_id", value.entity_id_value()));
        self
    }

    pub fn with_security_privilege_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "security_privilege_id",
            <crate::SecurityPrivilege as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters
            .push(RelationFilter::new("security_privilege", selection));
        self
    }

    pub fn without_security_privilege_matching(
        mut self,
        request: impl Into<QuerySelection>,
    ) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "security_privilege_id",
            <crate::SecurityPrivilege as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters
            .push(RelationFilter::new("security_privilege", selection));
        self
    }

    pub fn have_security_privilege(mut self) -> Self {
        self.query = self
            .query
            .and_filter(Expr::is_not_null("security_privilege_id"));
        self
    }

    pub fn have_no_security_privilege(mut self) -> Self {
        self.query = self
            .query
            .and_filter(Expr::is_null("security_privilege_id"));
        self
    }

    pub fn group_by_security_privilege(self) -> Self {
        self.group_by("security_privilege_id")
    }

    pub fn group_by_security_privilege_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("security_privilege_id");
        request.query = request
            .query
            .project_expr(alias, Expr::column("security_privilege_id"));
        request
    }

    pub fn group_by_security_privilege_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("security_privilege_id")
            .aggregate_with_function("security_privilege_id", alias, function)
    }

    pub fn group_by_security_privilege_with(mut self, request: impl Into<QuerySelection>) -> Self {
        self.query = self.query.group_by("security_privilege_id");
        self.query_options.object_group_bys.push(ObjectGroupBy::new(
            "security_privilege",
            "security_privilege_id",
            request,
        ));
        self
    }

    pub fn group_by_security_privilege_with_details(self) -> Self {
        self.group_by_security_privilege_with_details_from(
            crate::Q::security_privileges().unlimited(),
        )
    }

    pub fn group_by_security_privilege_with_details_from(
        self,
        request: impl Into<QuerySelection>,
    ) -> Self {
        self.group_by_security_privilege_with(request)
    }

    pub fn roll_up_to_security_privilege(self) -> Self {
        self.roll_up_to_security_privilege_with(crate::Q::security_privileges().unlimited())
    }

    pub fn roll_up_to_security_privilege_with(self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.with_security_privilege_matching(selection.clone())
            .group_by_security_privilege_with(selection)
    }

    pub fn count_security_privilege(self) -> Self {
        self.count_security_privilege_as("security_privilege_count")
    }

    pub fn count_security_privilege_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("security_privilege_id", alias)
    }

    pub fn unselect_security_privilege(mut self) -> Self {
        self.query
            .projection
            .retain(|field| field != "security_privilege_id");
        self.query
            .relations
            .retain(|relation| relation.name != "security_privilege");
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

    pub fn select_security_role(mut self) -> Self {
        self.query = self.query.relation("security_role");
        self
    }

    pub fn select_security_role_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self
            .query
            .relation_query("security_role", selection.into_query());
        self
    }

    pub fn facet_by_security_role_as(
        self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
    ) -> Self {
        self.facet_by_security_role_as_with_options(facet_name, request, true)
    }

    pub fn facet_by_security_role_as_with_options(
        mut self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
        include_all_facets: bool,
    ) -> Self {
        self.query_options.facets.push(FacetRequest::new(
            facet_name,
            "security_role",
            request,
            include_all_facets,
        ));
        self
    }

    pub fn select_security_privilege(mut self) -> Self {
        self.query = self.query.relation("security_privilege");
        self
    }

    pub fn select_security_privilege_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self
            .query
            .relation_query("security_privilege", selection.into_query());
        self
    }

    pub fn facet_by_security_privilege_as(
        self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
    ) -> Self {
        self.facet_by_security_privilege_as_with_options(facet_name, request, true)
    }

    pub fn facet_by_security_privilege_as_with_options(
        mut self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
        include_all_facets: bool,
    ) -> Self {
        self.query_options.facets.push(FacetRequest::new(
            facet_name,
            "security_privilege",
            request,
            include_all_facets,
        ));
        self
    }
}

impl<R> Default for SecurityRolePrivilegeRequest<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> From<SecurityRolePrivilegeRequest<R>> for SelectQuery {
    fn from(request: SecurityRolePrivilegeRequest<R>) -> Self {
        QuerySelection::from(request).into_query()
    }
}

impl<R> From<SecurityRolePrivilegeRequest<R>> for QuerySelection {
    fn from(request: SecurityRolePrivilegeRequest<R>) -> Self {
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
    for teaql_core::Audited<crate::SecurityRolePrivilege>
where
    C: crate::request_support::TeaqlRepositoryProvider + ?Sized + 'a,
{
    type Error = crate::TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>;
    type Entity = crate::SecurityRolePrivilege;
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

impl<R: teaql_core::Entity> crate::PurposedQuery<SecurityRolePrivilegeRequest<R>> {
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.inner.query_options.comment = Some(comment.into());
        self
    }

    pub fn new_entity<C>(&self, context: &C) -> crate::SecurityRolePrivilege
    where
        C: crate::TeaqlRuntime + ?Sized,
    {
        self.require_comment();
        let mut entity = crate::SecurityRolePrivilege::runtime_new(
            context.user_context().entity_runtime_state(),
        );
        if let Ok(id) = context
            .user_context()
            .next_id(crate::SecurityRolePrivilege::ENTITY_NAME)
        {
            entity.update_id(id);
        }
        teaql_core::Entity::mark_as_new(&mut entity);
        entity
    }

    fn into_inner_with_trace(mut self) -> SecurityRolePrivilegeRequest<R> {
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
        crate::request_support::TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>,
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
        crate::request_support::TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>,
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
        crate::request_support::TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>,
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
        crate::request_support::TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>,
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
            crate::request_support::TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>,
        >,
        crate::request_support::TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>,
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
        crate::request_support::TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>,
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
        crate::request_support::TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>,
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
        crate::request_support::TeaqlDataServiceError<C::SecurityRolePrivilegeRepository<'a>>,
    >
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()
            ._execute_for_count(context)
            .await
    }
}
