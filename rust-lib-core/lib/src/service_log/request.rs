use std::marker::PhantomData;

use serde_json::Value as JsonValue;
use teaql_core::{Aggregate, AggregateFunction, EntityDescriptor, Expr, SelectQuery, SmartList};
use teaql_runtime::{DataServiceError, RuntimeError};

use crate::request_support::*;

impl EntityReference for crate::ServiceLog {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(&self)
    }
}

impl EntityReference for &crate::ServiceLog {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(self)
    }
}

// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/service_log
#[derive(Debug)]
pub struct ServiceLogRequest<R = crate::ServiceLog> {
    query: SelectQuery,
    relation_selections: Vec<RelationSelection>,
    relation_filters: Vec<RelationFilter>,
    child_enhancements: Vec<QuerySelection>,
    query_options: QueryOptions,
    marker: PhantomData<R>,
}

impl<R> Clone for ServiceLogRequest<R> {
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

impl<R> ServiceLogRequest<R> {
    pub(crate) fn new() -> Self {
        Self {
            query: SelectQuery::new("ServiceLog")
                .project("id")
                .project("version"),
            relation_selections: Vec::new(),
            relation_filters: Vec::new(),
            child_enhancements: Vec::new(),
            query_options: QueryOptions::default(),
            marker: PhantomData,
        }
    }

    pub fn return_type<T>(self) -> ServiceLogRequest<T> {
        ServiceLogRequest {
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
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::ServiceLogRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        let repository = context
            .service_log_repository()
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
    ) -> Result<SmartList<teaql_core::CompactRow>, TeaqlDataServiceError<C::ServiceLogRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = context
            .service_log_repository()
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
        TeaqlEntityStream<'a, R, TeaqlDataServiceError<C::ServiceLogRepository<'a>>>,
        TeaqlDataServiceError<C::ServiceLogRepository<'a>>,
    >
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity + 'a,
    {
        Ok(Box::pin(async_stream::try_stream! {
            use futures_util::StreamExt;
            let repository = context
                .service_log_repository()
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
    ) -> Result<Option<R>, TeaqlDataServiceError<C::ServiceLogRepository<'a>>>
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
    ) -> Result<Option<R>, TeaqlDataServiceError<C::ServiceLogRepository<'a>>>
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
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::ServiceLogRepository<'a>>>
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
    ) -> Result<u64, TeaqlDataServiceError<C::ServiceLogRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = context
            .service_log_repository()
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
                    "count result for ServiceLog is missing or not numeric"
                )))
            })
    }

    pub(crate) async fn _execute_for_exists<'a, C>(
        self,
        context: &'a C,
    ) -> Result<bool, TeaqlDataServiceError<C::ServiceLogRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = context
            .service_log_repository()
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
            "event_time" => Some("event_time"),
            "log_type" => Some("log_type"),
            "operator_id" => Some("operator_id"),
            "operator_name" => Some("operator_name"),
            "client_ip" => Some("client_ip"),
            "action" => Some("action"),
            "repository_name" => Some("repository_name"),
            "artifact_path" => Some("artifact_path"),
            "format_name" => Some("format_name"),
            "content_size" => Some("content_size"),
            "status" => Some("status"),
            "error_message" => Some("error_message"),
            "version" => Some("version"),
            "tenant" | "tenant_id" => Some("tenant_id"),
            _ => None,
        }
    }

    fn apply_dynamic_json_chain_filter(self, head: &str, tail: &str, value: &JsonValue) -> Self {
        let _ = (tail, value);
        match head {
            "tenant" => self.with_tenant_matching(
                crate::Q::tenants_minimal().apply_dynamic_json_filter(tail, value),
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
        self.query = self.query.project("event_time");
        self.query = self.query.project("log_type");
        self.query = self.query.project("operator_id");
        self.query = self.query.project("operator_name");
        self.query = self.query.project("client_ip");
        self.query = self.query.project("action");
        self.query = self.query.project("repository_name");
        self.query = self.query.project("artifact_path");
        self.query = self.query.project("format_name");
        self.query = self.query.project("content_size");
        self.query = self.query.project("status");
        self.query = self.query.project("error_message");
        self.query = self.query.project("version");
        self.query = self.query.project("tenant_id");
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

    pub fn select_event_time(mut self) -> Self {
        self.query = self.query.project("event_time");
        self
    }

    pub fn project_event_time(self) -> Self {
        self.select_event_time()
    }

    pub fn select_event_time_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_event_time_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_event_time_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("event_time", raw_sql_segment));
        self
    }

    pub fn group_by_event_time(self) -> Self {
        self.group_by("event_time")
    }

    pub fn group_by_event_time_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("event_time");
        request.query = request
            .query
            .project_expr(alias, Expr::column("event_time"));
        request
    }

    pub fn group_by_event_time_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("event_time")
            .aggregate_with_function("event_time", alias, function)
    }

    pub fn count_event_time(self) -> Self {
        self.count_event_time_as("event_time_count")
    }

    pub fn count_event_time_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("event_time", alias)
    }

    pub fn sum_event_time(self) -> Self {
        self.sum_event_time_as("sum_event_time")
    }

    pub fn sum_event_time_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("event_time", alias)
    }

    pub fn avg_event_time(self) -> Self {
        self.avg_event_time_as("avg_event_time")
    }

    pub fn avg_event_time_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("event_time", alias)
    }

    pub fn min_event_time(self) -> Self {
        self.min_event_time_as("min_event_time")
    }

    pub fn min_event_time_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("event_time", alias)
    }

    pub fn max_event_time(self) -> Self {
        self.max_event_time_as("max_event_time")
    }

    pub fn max_event_time_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("event_time", alias)
    }

    pub fn unselect_event_time(mut self) -> Self {
        self.query.projection.retain(|field| field != "event_time");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "event_time");
        self
    }

    pub fn with_event_time(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "event_time",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_event_time_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "event_time",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_event_time_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("event_time", value));
        self
    }

    pub fn with_event_time_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("event_time", value));
        self
    }

    pub fn with_event_time_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("event_time", value));
        self
    }

    pub fn with_event_time_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("event_time", value));
        self
    }

    pub fn with_event_time_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("event_time", value));
        self
    }

    pub fn with_event_time_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("event_time", value));
        self
    }

    pub fn with_event_time_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("event_time", lower, upper));
        self
    }

    pub fn with_event_time_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("event_time", range.start, range.end));
        self
    }

    pub fn with_event_time_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "event_time",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_event_time_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "event_time",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_event_time_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("event_time", value));
        self
    }

    pub fn with_event_time_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("event_time", value));
        self
    }

    pub fn with_event_time_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("event_time"));
        self
    }

    pub fn with_event_time_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("event_time"));
        self
    }

    pub fn order_by_event_time_asc(mut self) -> Self {
        self.query = self.query.order_asc("event_time");
        self
    }

    pub fn order_by_event_time_desc(mut self) -> Self {
        self.query = self.query.order_desc("event_time");
        self
    }

    pub fn order_by_event_time_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("event_time");
        self
    }

    pub fn order_by_event_time_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("event_time");
        self
    }

    pub fn select_log_type(mut self) -> Self {
        self.query = self.query.project("log_type");
        self
    }

    pub fn project_log_type(self) -> Self {
        self.select_log_type()
    }

    pub fn select_log_type_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_log_type_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_log_type_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("log_type", raw_sql_segment));
        self
    }

    pub fn group_by_log_type(self) -> Self {
        self.group_by("log_type")
    }

    pub fn group_by_log_type_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("log_type");
        request.query = request.query.project_expr(alias, Expr::column("log_type"));
        request
    }

    pub fn group_by_log_type_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("log_type")
            .aggregate_with_function("log_type", alias, function)
    }

    pub fn count_log_type(self) -> Self {
        self.count_log_type_as("log_type_count")
    }

    pub fn count_log_type_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("log_type", alias)
    }

    pub fn sum_log_type(self) -> Self {
        self.sum_log_type_as("sum_log_type")
    }

    pub fn sum_log_type_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("log_type", alias)
    }

    pub fn avg_log_type(self) -> Self {
        self.avg_log_type_as("avg_log_type")
    }

    pub fn avg_log_type_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("log_type", alias)
    }

    pub fn min_log_type(self) -> Self {
        self.min_log_type_as("min_log_type")
    }

    pub fn min_log_type_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("log_type", alias)
    }

    pub fn max_log_type(self) -> Self {
        self.max_log_type_as("max_log_type")
    }

    pub fn max_log_type_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("log_type", alias)
    }

    pub fn unselect_log_type(mut self) -> Self {
        self.query.projection.retain(|field| field != "log_type");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "log_type");
        self
    }

    pub fn with_log_type(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "log_type",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_log_type_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "log_type",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_log_type_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("log_type", value));
        self
    }

    pub fn with_log_type_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("log_type", value));
        self
    }

    pub fn with_log_type_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("log_type", value));
        self
    }

    pub fn with_log_type_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("log_type", value));
        self
    }

    pub fn with_log_type_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("log_type", value));
        self
    }

    pub fn with_log_type_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("log_type", value));
        self
    }

    pub fn with_log_type_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("log_type", lower, upper));
        self
    }

    pub fn with_log_type_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("log_type", range.start, range.end));
        self
    }

    pub fn with_log_type_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "log_type",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_log_type_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "log_type",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_log_type_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("log_type", value));
        self
    }

    pub fn with_log_type_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("log_type", value));
        self
    }

    pub fn with_log_type_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("log_type", value));
        self
    }

    pub fn with_log_type_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_begin_with("log_type", value));
        self
    }

    pub fn with_log_type_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("log_type", value));
        self
    }

    pub fn with_log_type_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("log_type", value));
        self
    }

    pub fn with_log_type_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("log_type", value));
        self
    }
    pub fn with_log_type_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("log_type", value));
        self
    }

    pub fn with_log_type_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("log_type", value));
        self
    }

    pub fn with_log_type_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("log_type"));
        self
    }

    pub fn with_log_type_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("log_type"));
        self
    }

    pub fn order_by_log_type_asc(mut self) -> Self {
        self.query = self.query.order_asc("log_type");
        self
    }

    pub fn order_by_log_type_desc(mut self) -> Self {
        self.query = self.query.order_desc("log_type");
        self
    }

    pub fn order_by_log_type_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("log_type");
        self
    }

    pub fn order_by_log_type_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("log_type");
        self
    }

    pub fn select_operator_id(mut self) -> Self {
        self.query = self.query.project("operator_id");
        self
    }

    pub fn project_operator_id(self) -> Self {
        self.select_operator_id()
    }

    pub fn select_operator_id_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_operator_id_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_operator_id_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("operator_id", raw_sql_segment));
        self
    }

    pub fn select_operator_id_with_function(self, function: AggregateFunction) -> Self {
        self.select_operator_id_as_with_function("operator_id", function)
    }

    pub fn select_operator_id_as_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.aggregate_with_function("operator_id", alias, function)
    }

    pub fn group_by_operator_id(self) -> Self {
        self.group_by("operator_id")
    }

    pub fn group_by_operator_id_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("operator_id");
        request.query = request
            .query
            .project_expr(alias, Expr::column("operator_id"));
        request
    }

    pub fn group_by_operator_id_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("operator_id")
            .aggregate_with_function("operator_id", alias, function)
    }

    pub fn count_operator_id(self) -> Self {
        self.count_operator_id_as("operator_id_count")
    }

    pub fn count_operator_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("operator_id", alias)
    }

    pub fn sum_operator_id(self) -> Self {
        self.sum_operator_id_as("sum_operator_id")
    }

    pub fn sum_operator_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("operator_id", alias)
    }

    pub fn avg_operator_id(self) -> Self {
        self.avg_operator_id_as("avg_operator_id")
    }

    pub fn avg_operator_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("operator_id", alias)
    }

    pub fn min_operator_id(self) -> Self {
        self.min_operator_id_as("min_operator_id")
    }

    pub fn min_operator_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("operator_id", alias)
    }

    pub fn max_operator_id(self) -> Self {
        self.max_operator_id_as("max_operator_id")
    }

    pub fn max_operator_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("operator_id", alias)
    }

    pub fn standard_deviation_operator_id(self) -> Self {
        self.standard_deviation_operator_id_as("stdDev_operator_id")
    }

    pub fn standard_deviation_operator_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_stddev("operator_id", alias)
    }

    pub fn square_root_of_population_standard_deviation_operator_id(self) -> Self {
        self.square_root_of_population_standard_deviation_operator_id_as("stdDevPop_operator_id")
    }

    pub fn square_root_of_population_standard_deviation_operator_id_as(
        self,
        alias: impl Into<String>,
    ) -> Self {
        self.aggregate_stddev_pop("operator_id", alias)
    }

    pub fn sample_variance_operator_id(self) -> Self {
        self.sample_variance_operator_id_as("varSamp_operator_id")
    }

    pub fn sample_variance_operator_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_var_samp("operator_id", alias)
    }

    pub fn sample_population_variance_operator_id(self) -> Self {
        self.sample_population_variance_operator_id_as("varPop_operator_id")
    }

    pub fn sample_population_variance_operator_id_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_var_pop("operator_id", alias)
    }

    pub fn unselect_operator_id(mut self) -> Self {
        self.query.projection.retain(|field| field != "operator_id");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "operator_id");
        self
    }

    pub fn with_operator_id(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "operator_id",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_operator_id_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "operator_id",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_operator_id_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("operator_id", value));
        self
    }

    pub fn with_operator_id_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("operator_id", value));
        self
    }

    pub fn with_operator_id_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("operator_id", value));
        self
    }

    pub fn with_operator_id_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("operator_id", value));
        self
    }

    pub fn with_operator_id_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("operator_id", value));
        self
    }

    pub fn with_operator_id_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("operator_id", value));
        self
    }

    pub fn with_operator_id_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("operator_id", lower, upper));
        self
    }

    pub fn with_operator_id_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("operator_id", range.start, range.end));
        self
    }

    pub fn with_operator_id_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "operator_id",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_operator_id_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "operator_id",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_operator_id_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("operator_id", value));
        self
    }

    pub fn with_operator_id_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("operator_id", value));
        self
    }

    pub fn with_operator_id_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("operator_id"));
        self
    }

    pub fn with_operator_id_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("operator_id"));
        self
    }

    pub fn order_by_operator_id_asc(mut self) -> Self {
        self.query = self.query.order_asc("operator_id");
        self
    }

    pub fn order_by_operator_id_desc(mut self) -> Self {
        self.query = self.query.order_desc("operator_id");
        self
    }

    pub fn order_by_operator_id_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("operator_id");
        self
    }

    pub fn order_by_operator_id_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("operator_id");
        self
    }

    pub fn select_operator_name(mut self) -> Self {
        self.query = self.query.project("operator_name");
        self
    }

    pub fn project_operator_name(self) -> Self {
        self.select_operator_name()
    }

    pub fn select_operator_name_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_operator_name_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_operator_name_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("operator_name", raw_sql_segment));
        self
    }

    pub fn group_by_operator_name(self) -> Self {
        self.group_by("operator_name")
    }

    pub fn group_by_operator_name_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("operator_name");
        request.query = request
            .query
            .project_expr(alias, Expr::column("operator_name"));
        request
    }

    pub fn group_by_operator_name_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("operator_name")
            .aggregate_with_function("operator_name", alias, function)
    }

    pub fn count_operator_name(self) -> Self {
        self.count_operator_name_as("operator_name_count")
    }

    pub fn count_operator_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("operator_name", alias)
    }

    pub fn sum_operator_name(self) -> Self {
        self.sum_operator_name_as("sum_operator_name")
    }

    pub fn sum_operator_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("operator_name", alias)
    }

    pub fn avg_operator_name(self) -> Self {
        self.avg_operator_name_as("avg_operator_name")
    }

    pub fn avg_operator_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("operator_name", alias)
    }

    pub fn min_operator_name(self) -> Self {
        self.min_operator_name_as("min_operator_name")
    }

    pub fn min_operator_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("operator_name", alias)
    }

    pub fn max_operator_name(self) -> Self {
        self.max_operator_name_as("max_operator_name")
    }

    pub fn max_operator_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("operator_name", alias)
    }

    pub fn unselect_operator_name(mut self) -> Self {
        self.query
            .projection
            .retain(|field| field != "operator_name");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "operator_name");
        self
    }

    pub fn with_operator_name(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "operator_name",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_operator_name_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "operator_name",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_operator_name_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("operator_name", value));
        self
    }

    pub fn with_operator_name_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("operator_name", value));
        self
    }

    pub fn with_operator_name_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("operator_name", value));
        self
    }

    pub fn with_operator_name_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("operator_name", value));
        self
    }

    pub fn with_operator_name_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("operator_name", value));
        self
    }

    pub fn with_operator_name_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("operator_name", value));
        self
    }

    pub fn with_operator_name_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("operator_name", lower, upper));
        self
    }

    pub fn with_operator_name_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("operator_name", range.start, range.end));
        self
    }

    pub fn with_operator_name_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "operator_name",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_operator_name_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "operator_name",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_operator_name_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("operator_name", value));
        self
    }

    pub fn with_operator_name_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_contain("operator_name", value));
        self
    }

    pub fn with_operator_name_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::begin_with("operator_name", value));
        self
    }

    pub fn with_operator_name_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_begin_with("operator_name", value));
        self
    }

    pub fn with_operator_name_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::end_with("operator_name", value));
        self
    }

    pub fn with_operator_name_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_end_with("operator_name", value));
        self
    }

    pub fn with_operator_name_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::sound_like("operator_name", value));
        self
    }
    pub fn with_operator_name_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("operator_name", value));
        self
    }

    pub fn with_operator_name_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("operator_name", value));
        self
    }

    pub fn with_operator_name_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("operator_name"));
        self
    }

    pub fn with_operator_name_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("operator_name"));
        self
    }

    pub fn order_by_operator_name_asc(mut self) -> Self {
        self.query = self.query.order_asc("operator_name");
        self
    }

    pub fn order_by_operator_name_desc(mut self) -> Self {
        self.query = self.query.order_desc("operator_name");
        self
    }

    pub fn order_by_operator_name_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("operator_name");
        self
    }

    pub fn order_by_operator_name_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("operator_name");
        self
    }

    pub fn select_client_ip(mut self) -> Self {
        self.query = self.query.project("client_ip");
        self
    }

    pub fn project_client_ip(self) -> Self {
        self.select_client_ip()
    }

    pub fn select_client_ip_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_client_ip_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_client_ip_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("client_ip", raw_sql_segment));
        self
    }

    pub fn group_by_client_ip(self) -> Self {
        self.group_by("client_ip")
    }

    pub fn group_by_client_ip_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("client_ip");
        request.query = request.query.project_expr(alias, Expr::column("client_ip"));
        request
    }

    pub fn group_by_client_ip_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("client_ip")
            .aggregate_with_function("client_ip", alias, function)
    }

    pub fn count_client_ip(self) -> Self {
        self.count_client_ip_as("client_ip_count")
    }

    pub fn count_client_ip_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("client_ip", alias)
    }

    pub fn sum_client_ip(self) -> Self {
        self.sum_client_ip_as("sum_client_ip")
    }

    pub fn sum_client_ip_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("client_ip", alias)
    }

    pub fn avg_client_ip(self) -> Self {
        self.avg_client_ip_as("avg_client_ip")
    }

    pub fn avg_client_ip_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("client_ip", alias)
    }

    pub fn min_client_ip(self) -> Self {
        self.min_client_ip_as("min_client_ip")
    }

    pub fn min_client_ip_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("client_ip", alias)
    }

    pub fn max_client_ip(self) -> Self {
        self.max_client_ip_as("max_client_ip")
    }

    pub fn max_client_ip_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("client_ip", alias)
    }

    pub fn unselect_client_ip(mut self) -> Self {
        self.query.projection.retain(|field| field != "client_ip");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "client_ip");
        self
    }

    pub fn with_client_ip(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "client_ip",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_client_ip_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "client_ip",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_client_ip_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("client_ip", value));
        self
    }

    pub fn with_client_ip_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("client_ip", value));
        self
    }

    pub fn with_client_ip_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("client_ip", value));
        self
    }

    pub fn with_client_ip_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("client_ip", value));
        self
    }

    pub fn with_client_ip_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("client_ip", value));
        self
    }

    pub fn with_client_ip_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("client_ip", value));
        self
    }

    pub fn with_client_ip_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("client_ip", lower, upper));
        self
    }

    pub fn with_client_ip_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("client_ip", range.start, range.end));
        self
    }

    pub fn with_client_ip_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "client_ip",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_client_ip_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "client_ip",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_client_ip_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("client_ip", value));
        self
    }

    pub fn with_client_ip_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("client_ip", value));
        self
    }

    pub fn with_client_ip_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("client_ip", value));
        self
    }

    pub fn with_client_ip_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_begin_with("client_ip", value));
        self
    }

    pub fn with_client_ip_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("client_ip", value));
        self
    }

    pub fn with_client_ip_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_end_with("client_ip", value));
        self
    }

    pub fn with_client_ip_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("client_ip", value));
        self
    }
    pub fn with_client_ip_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("client_ip", value));
        self
    }

    pub fn with_client_ip_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("client_ip", value));
        self
    }

    pub fn with_client_ip_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("client_ip"));
        self
    }

    pub fn with_client_ip_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("client_ip"));
        self
    }

    pub fn order_by_client_ip_asc(mut self) -> Self {
        self.query = self.query.order_asc("client_ip");
        self
    }

    pub fn order_by_client_ip_desc(mut self) -> Self {
        self.query = self.query.order_desc("client_ip");
        self
    }

    pub fn order_by_client_ip_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("client_ip");
        self
    }

    pub fn order_by_client_ip_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("client_ip");
        self
    }

    pub fn select_action(mut self) -> Self {
        self.query = self.query.project("action");
        self
    }

    pub fn project_action(self) -> Self {
        self.select_action()
    }

    pub fn select_action_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_action_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_action_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("action", raw_sql_segment));
        self
    }

    pub fn group_by_action(self) -> Self {
        self.group_by("action")
    }

    pub fn group_by_action_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("action");
        request.query = request.query.project_expr(alias, Expr::column("action"));
        request
    }

    pub fn group_by_action_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("action")
            .aggregate_with_function("action", alias, function)
    }

    pub fn count_action(self) -> Self {
        self.count_action_as("action_count")
    }

    pub fn count_action_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("action", alias)
    }

    pub fn sum_action(self) -> Self {
        self.sum_action_as("sum_action")
    }

    pub fn sum_action_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("action", alias)
    }

    pub fn avg_action(self) -> Self {
        self.avg_action_as("avg_action")
    }

    pub fn avg_action_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("action", alias)
    }

    pub fn min_action(self) -> Self {
        self.min_action_as("min_action")
    }

    pub fn min_action_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("action", alias)
    }

    pub fn max_action(self) -> Self {
        self.max_action_as("max_action")
    }

    pub fn max_action_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("action", alias)
    }

    pub fn unselect_action(mut self) -> Self {
        self.query.projection.retain(|field| field != "action");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "action");
        self
    }

    pub fn with_action(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "action",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_action_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "action",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_action_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("action", value));
        self
    }

    pub fn with_action_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("action", value));
        self
    }

    pub fn with_action_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("action", value));
        self
    }

    pub fn with_action_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("action", value));
        self
    }

    pub fn with_action_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("action", value));
        self
    }

    pub fn with_action_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("action", value));
        self
    }

    pub fn with_action_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("action", lower, upper));
        self
    }

    pub fn with_action_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("action", range.start, range.end));
        self
    }

    pub fn with_action_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::in_list("action", values.into_iter().map(Into::into)));
        self
    }

    pub fn with_action_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "action",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_action_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("action", value));
        self
    }

    pub fn with_action_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("action", value));
        self
    }

    pub fn with_action_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("action", value));
        self
    }

    pub fn with_action_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("action", value));
        self
    }

    pub fn with_action_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("action", value));
        self
    }

    pub fn with_action_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("action", value));
        self
    }

    pub fn with_action_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("action", value));
        self
    }
    pub fn with_action_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("action", value));
        self
    }

    pub fn with_action_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("action", value));
        self
    }

    pub fn with_action_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("action"));
        self
    }

    pub fn with_action_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("action"));
        self
    }

    pub fn order_by_action_asc(mut self) -> Self {
        self.query = self.query.order_asc("action");
        self
    }

    pub fn order_by_action_desc(mut self) -> Self {
        self.query = self.query.order_desc("action");
        self
    }

    pub fn order_by_action_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("action");
        self
    }

    pub fn order_by_action_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("action");
        self
    }

    pub fn select_repository_name(mut self) -> Self {
        self.query = self.query.project("repository_name");
        self
    }

    pub fn project_repository_name(self) -> Self {
        self.select_repository_name()
    }

    pub fn select_repository_name_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_repository_name_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_repository_name_unsafe_raw(
        mut self,
        raw_sql_segment: UnsafeRawSqlSegment,
    ) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("repository_name", raw_sql_segment));
        self
    }

    pub fn group_by_repository_name(self) -> Self {
        self.group_by("repository_name")
    }

    pub fn group_by_repository_name_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("repository_name");
        request.query = request
            .query
            .project_expr(alias, Expr::column("repository_name"));
        request
    }

    pub fn group_by_repository_name_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("repository_name")
            .aggregate_with_function("repository_name", alias, function)
    }

    pub fn count_repository_name(self) -> Self {
        self.count_repository_name_as("repository_name_count")
    }

    pub fn count_repository_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("repository_name", alias)
    }

    pub fn sum_repository_name(self) -> Self {
        self.sum_repository_name_as("sum_repository_name")
    }

    pub fn sum_repository_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("repository_name", alias)
    }

    pub fn avg_repository_name(self) -> Self {
        self.avg_repository_name_as("avg_repository_name")
    }

    pub fn avg_repository_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("repository_name", alias)
    }

    pub fn min_repository_name(self) -> Self {
        self.min_repository_name_as("min_repository_name")
    }

    pub fn min_repository_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("repository_name", alias)
    }

    pub fn max_repository_name(self) -> Self {
        self.max_repository_name_as("max_repository_name")
    }

    pub fn max_repository_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("repository_name", alias)
    }

    pub fn unselect_repository_name(mut self) -> Self {
        self.query
            .projection
            .retain(|field| field != "repository_name");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "repository_name");
        self
    }

    pub fn with_repository_name(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "repository_name",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_repository_name_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "repository_name",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_repository_name_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("repository_name", value));
        self
    }

    pub fn with_repository_name_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_name", value));
        self
    }

    pub fn with_repository_name_greater_than(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gt("repository_name", value));
        self
    }

    pub fn with_repository_name_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("repository_name", value));
        self
    }

    pub fn with_repository_name_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("repository_name", value));
        self
    }

    pub fn with_repository_name_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("repository_name", value));
        self
    }

    pub fn with_repository_name_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("repository_name", lower, upper));
        self
    }

    pub fn with_repository_name_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query =
            self.query
                .and_filter(Expr::between("repository_name", range.start, range.end));
        self
    }

    pub fn with_repository_name_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "repository_name",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_repository_name_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "repository_name",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_repository_name_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::contain("repository_name", value));
        self
    }

    pub fn with_repository_name_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_contain("repository_name", value));
        self
    }

    pub fn with_repository_name_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::begin_with("repository_name", value));
        self
    }

    pub fn with_repository_name_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_begin_with("repository_name", value));
        self
    }

    pub fn with_repository_name_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::end_with("repository_name", value));
        self
    }

    pub fn with_repository_name_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_end_with("repository_name", value));
        self
    }

    pub fn with_repository_name_sounding_like(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::sound_like("repository_name", value));
        self
    }
    pub fn with_repository_name_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("repository_name", value));
        self
    }

    pub fn with_repository_name_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("repository_name", value));
        self
    }

    pub fn with_repository_name_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("repository_name"));
        self
    }

    pub fn with_repository_name_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("repository_name"));
        self
    }

    pub fn order_by_repository_name_asc(mut self) -> Self {
        self.query = self.query.order_asc("repository_name");
        self
    }

    pub fn order_by_repository_name_desc(mut self) -> Self {
        self.query = self.query.order_desc("repository_name");
        self
    }

    pub fn order_by_repository_name_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("repository_name");
        self
    }

    pub fn order_by_repository_name_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("repository_name");
        self
    }

    pub fn select_artifact_path(mut self) -> Self {
        self.query = self.query.project("artifact_path");
        self
    }

    pub fn project_artifact_path(self) -> Self {
        self.select_artifact_path()
    }

    pub fn select_artifact_path_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_artifact_path_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_artifact_path_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("artifact_path", raw_sql_segment));
        self
    }

    pub fn group_by_artifact_path(self) -> Self {
        self.group_by("artifact_path")
    }

    pub fn group_by_artifact_path_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("artifact_path");
        request.query = request
            .query
            .project_expr(alias, Expr::column("artifact_path"));
        request
    }

    pub fn group_by_artifact_path_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("artifact_path")
            .aggregate_with_function("artifact_path", alias, function)
    }

    pub fn count_artifact_path(self) -> Self {
        self.count_artifact_path_as("artifact_path_count")
    }

    pub fn count_artifact_path_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("artifact_path", alias)
    }

    pub fn sum_artifact_path(self) -> Self {
        self.sum_artifact_path_as("sum_artifact_path")
    }

    pub fn sum_artifact_path_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("artifact_path", alias)
    }

    pub fn avg_artifact_path(self) -> Self {
        self.avg_artifact_path_as("avg_artifact_path")
    }

    pub fn avg_artifact_path_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("artifact_path", alias)
    }

    pub fn min_artifact_path(self) -> Self {
        self.min_artifact_path_as("min_artifact_path")
    }

    pub fn min_artifact_path_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("artifact_path", alias)
    }

    pub fn max_artifact_path(self) -> Self {
        self.max_artifact_path_as("max_artifact_path")
    }

    pub fn max_artifact_path_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("artifact_path", alias)
    }

    pub fn unselect_artifact_path(mut self) -> Self {
        self.query
            .projection
            .retain(|field| field != "artifact_path");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "artifact_path");
        self
    }

    pub fn with_artifact_path(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "artifact_path",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_artifact_path_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "artifact_path",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_artifact_path_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("artifact_path", value));
        self
    }

    pub fn with_artifact_path_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("artifact_path", value));
        self
    }

    pub fn with_artifact_path_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("artifact_path", value));
        self
    }

    pub fn with_artifact_path_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("artifact_path", value));
        self
    }

    pub fn with_artifact_path_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("artifact_path", value));
        self
    }

    pub fn with_artifact_path_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("artifact_path", value));
        self
    }

    pub fn with_artifact_path_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("artifact_path", lower, upper));
        self
    }

    pub fn with_artifact_path_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("artifact_path", range.start, range.end));
        self
    }

    pub fn with_artifact_path_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "artifact_path",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_artifact_path_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "artifact_path",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_artifact_path_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("artifact_path", value));
        self
    }

    pub fn with_artifact_path_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_contain("artifact_path", value));
        self
    }

    pub fn with_artifact_path_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::begin_with("artifact_path", value));
        self
    }

    pub fn with_artifact_path_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_begin_with("artifact_path", value));
        self
    }

    pub fn with_artifact_path_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::end_with("artifact_path", value));
        self
    }

    pub fn with_artifact_path_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_end_with("artifact_path", value));
        self
    }

    pub fn with_artifact_path_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::sound_like("artifact_path", value));
        self
    }
    pub fn with_artifact_path_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("artifact_path", value));
        self
    }

    pub fn with_artifact_path_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("artifact_path", value));
        self
    }

    pub fn with_artifact_path_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("artifact_path"));
        self
    }

    pub fn with_artifact_path_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("artifact_path"));
        self
    }

    pub fn order_by_artifact_path_asc(mut self) -> Self {
        self.query = self.query.order_asc("artifact_path");
        self
    }

    pub fn order_by_artifact_path_desc(mut self) -> Self {
        self.query = self.query.order_desc("artifact_path");
        self
    }

    pub fn order_by_artifact_path_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("artifact_path");
        self
    }

    pub fn order_by_artifact_path_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("artifact_path");
        self
    }

    pub fn select_format_name(mut self) -> Self {
        self.query = self.query.project("format_name");
        self
    }

    pub fn project_format_name(self) -> Self {
        self.select_format_name()
    }

    pub fn select_format_name_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_format_name_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_format_name_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("format_name", raw_sql_segment));
        self
    }

    pub fn group_by_format_name(self) -> Self {
        self.group_by("format_name")
    }

    pub fn group_by_format_name_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("format_name");
        request.query = request
            .query
            .project_expr(alias, Expr::column("format_name"));
        request
    }

    pub fn group_by_format_name_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("format_name")
            .aggregate_with_function("format_name", alias, function)
    }

    pub fn count_format_name(self) -> Self {
        self.count_format_name_as("format_name_count")
    }

    pub fn count_format_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("format_name", alias)
    }

    pub fn sum_format_name(self) -> Self {
        self.sum_format_name_as("sum_format_name")
    }

    pub fn sum_format_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("format_name", alias)
    }

    pub fn avg_format_name(self) -> Self {
        self.avg_format_name_as("avg_format_name")
    }

    pub fn avg_format_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("format_name", alias)
    }

    pub fn min_format_name(self) -> Self {
        self.min_format_name_as("min_format_name")
    }

    pub fn min_format_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("format_name", alias)
    }

    pub fn max_format_name(self) -> Self {
        self.max_format_name_as("max_format_name")
    }

    pub fn max_format_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("format_name", alias)
    }

    pub fn unselect_format_name(mut self) -> Self {
        self.query.projection.retain(|field| field != "format_name");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "format_name");
        self
    }

    pub fn with_format_name(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "format_name",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_format_name_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "format_name",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_format_name_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("format_name", value));
        self
    }

    pub fn with_format_name_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("format_name", value));
        self
    }

    pub fn with_format_name_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("format_name", value));
        self
    }

    pub fn with_format_name_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("format_name", value));
        self
    }

    pub fn with_format_name_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("format_name", value));
        self
    }

    pub fn with_format_name_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("format_name", value));
        self
    }

    pub fn with_format_name_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("format_name", lower, upper));
        self
    }

    pub fn with_format_name_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("format_name", range.start, range.end));
        self
    }

    pub fn with_format_name_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "format_name",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_format_name_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "format_name",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_format_name_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("format_name", value));
        self
    }

    pub fn with_format_name_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_contain("format_name", value));
        self
    }

    pub fn with_format_name_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::begin_with("format_name", value));
        self
    }

    pub fn with_format_name_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_begin_with("format_name", value));
        self
    }

    pub fn with_format_name_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("format_name", value));
        self
    }

    pub fn with_format_name_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_end_with("format_name", value));
        self
    }

    pub fn with_format_name_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::sound_like("format_name", value));
        self
    }
    pub fn with_format_name_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("format_name", value));
        self
    }

    pub fn with_format_name_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("format_name", value));
        self
    }

    pub fn with_format_name_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("format_name"));
        self
    }

    pub fn with_format_name_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("format_name"));
        self
    }

    pub fn order_by_format_name_asc(mut self) -> Self {
        self.query = self.query.order_asc("format_name");
        self
    }

    pub fn order_by_format_name_desc(mut self) -> Self {
        self.query = self.query.order_desc("format_name");
        self
    }

    pub fn order_by_format_name_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("format_name");
        self
    }

    pub fn order_by_format_name_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("format_name");
        self
    }

    pub fn select_content_size(mut self) -> Self {
        self.query = self.query.project("content_size");
        self
    }

    pub fn project_content_size(self) -> Self {
        self.select_content_size()
    }

    pub fn select_content_size_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_content_size_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_content_size_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("content_size", raw_sql_segment));
        self
    }

    pub fn select_content_size_with_function(self, function: AggregateFunction) -> Self {
        self.select_content_size_as_with_function("content_size", function)
    }

    pub fn select_content_size_as_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.aggregate_with_function("content_size", alias, function)
    }

    pub fn group_by_content_size(self) -> Self {
        self.group_by("content_size")
    }

    pub fn group_by_content_size_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("content_size");
        request.query = request
            .query
            .project_expr(alias, Expr::column("content_size"));
        request
    }

    pub fn group_by_content_size_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("content_size")
            .aggregate_with_function("content_size", alias, function)
    }

    pub fn count_content_size(self) -> Self {
        self.count_content_size_as("content_size_count")
    }

    pub fn count_content_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("content_size", alias)
    }

    pub fn sum_content_size(self) -> Self {
        self.sum_content_size_as("sum_content_size")
    }

    pub fn sum_content_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("content_size", alias)
    }

    pub fn avg_content_size(self) -> Self {
        self.avg_content_size_as("avg_content_size")
    }

    pub fn avg_content_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("content_size", alias)
    }

    pub fn min_content_size(self) -> Self {
        self.min_content_size_as("min_content_size")
    }

    pub fn min_content_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("content_size", alias)
    }

    pub fn max_content_size(self) -> Self {
        self.max_content_size_as("max_content_size")
    }

    pub fn max_content_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("content_size", alias)
    }

    pub fn standard_deviation_content_size(self) -> Self {
        self.standard_deviation_content_size_as("stdDev_content_size")
    }

    pub fn standard_deviation_content_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_stddev("content_size", alias)
    }

    pub fn square_root_of_population_standard_deviation_content_size(self) -> Self {
        self.square_root_of_population_standard_deviation_content_size_as("stdDevPop_content_size")
    }

    pub fn square_root_of_population_standard_deviation_content_size_as(
        self,
        alias: impl Into<String>,
    ) -> Self {
        self.aggregate_stddev_pop("content_size", alias)
    }

    pub fn sample_variance_content_size(self) -> Self {
        self.sample_variance_content_size_as("varSamp_content_size")
    }

    pub fn sample_variance_content_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_var_samp("content_size", alias)
    }

    pub fn sample_population_variance_content_size(self) -> Self {
        self.sample_population_variance_content_size_as("varPop_content_size")
    }

    pub fn sample_population_variance_content_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_var_pop("content_size", alias)
    }

    pub fn unselect_content_size(mut self) -> Self {
        self.query
            .projection
            .retain(|field| field != "content_size");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "content_size");
        self
    }

    pub fn with_content_size(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "content_size",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_content_size_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "content_size",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_content_size_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("content_size", value));
        self
    }

    pub fn with_content_size_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("content_size", value));
        self
    }

    pub fn with_content_size_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("content_size", value));
        self
    }

    pub fn with_content_size_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("content_size", value));
        self
    }

    pub fn with_content_size_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("content_size", value));
        self
    }

    pub fn with_content_size_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("content_size", value));
        self
    }

    pub fn with_content_size_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("content_size", lower, upper));
        self
    }

    pub fn with_content_size_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("content_size", range.start, range.end));
        self
    }

    pub fn with_content_size_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "content_size",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_content_size_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "content_size",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_content_size_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("content_size", value));
        self
    }

    pub fn with_content_size_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("content_size", value));
        self
    }

    pub fn with_content_size_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("content_size"));
        self
    }

    pub fn with_content_size_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("content_size"));
        self
    }

    pub fn order_by_content_size_asc(mut self) -> Self {
        self.query = self.query.order_asc("content_size");
        self
    }

    pub fn order_by_content_size_desc(mut self) -> Self {
        self.query = self.query.order_desc("content_size");
        self
    }

    pub fn order_by_content_size_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("content_size");
        self
    }

    pub fn order_by_content_size_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("content_size");
        self
    }

    pub fn select_status(mut self) -> Self {
        self.query = self.query.project("status");
        self
    }

    pub fn project_status(self) -> Self {
        self.select_status()
    }

    pub fn select_status_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_status_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_status_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("status", raw_sql_segment));
        self
    }

    pub fn group_by_status(self) -> Self {
        self.group_by("status")
    }

    pub fn group_by_status_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("status");
        request.query = request.query.project_expr(alias, Expr::column("status"));
        request
    }

    pub fn group_by_status_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("status")
            .aggregate_with_function("status", alias, function)
    }

    pub fn count_status(self) -> Self {
        self.count_status_as("status_count")
    }

    pub fn count_status_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("status", alias)
    }

    pub fn sum_status(self) -> Self {
        self.sum_status_as("sum_status")
    }

    pub fn sum_status_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("status", alias)
    }

    pub fn avg_status(self) -> Self {
        self.avg_status_as("avg_status")
    }

    pub fn avg_status_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("status", alias)
    }

    pub fn min_status(self) -> Self {
        self.min_status_as("min_status")
    }

    pub fn min_status_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("status", alias)
    }

    pub fn max_status(self) -> Self {
        self.max_status_as("max_status")
    }

    pub fn max_status_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("status", alias)
    }

    pub fn unselect_status(mut self) -> Self {
        self.query.projection.retain(|field| field != "status");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "status");
        self
    }

    pub fn with_status(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "status",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_status_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "status",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_status_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("status", value));
        self
    }

    pub fn with_status_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("status", value));
        self
    }

    pub fn with_status_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("status", value));
        self
    }

    pub fn with_status_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("status", value));
        self
    }

    pub fn with_status_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("status", value));
        self
    }

    pub fn with_status_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("status", value));
        self
    }

    pub fn with_status_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("status", lower, upper));
        self
    }

    pub fn with_status_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("status", range.start, range.end));
        self
    }

    pub fn with_status_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::in_list("status", values.into_iter().map(Into::into)));
        self
    }

    pub fn with_status_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "status",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_status_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("status", value));
        self
    }

    pub fn with_status_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("status", value));
        self
    }

    pub fn with_status_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("status", value));
        self
    }

    pub fn with_status_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("status", value));
        self
    }

    pub fn with_status_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("status", value));
        self
    }

    pub fn with_status_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("status", value));
        self
    }

    pub fn with_status_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("status", value));
        self
    }
    pub fn with_status_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("status", value));
        self
    }

    pub fn with_status_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("status", value));
        self
    }

    pub fn with_status_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("status"));
        self
    }

    pub fn with_status_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("status"));
        self
    }

    pub fn order_by_status_asc(mut self) -> Self {
        self.query = self.query.order_asc("status");
        self
    }

    pub fn order_by_status_desc(mut self) -> Self {
        self.query = self.query.order_desc("status");
        self
    }

    pub fn order_by_status_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("status");
        self
    }

    pub fn order_by_status_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("status");
        self
    }

    pub fn select_error_message(mut self) -> Self {
        self.query = self.query.project("error_message");
        self
    }

    pub fn project_error_message(self) -> Self {
        self.select_error_message()
    }

    pub fn select_error_message_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_error_message_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_error_message_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("error_message", raw_sql_segment));
        self
    }

    pub fn group_by_error_message(self) -> Self {
        self.group_by("error_message")
    }

    pub fn group_by_error_message_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("error_message");
        request.query = request
            .query
            .project_expr(alias, Expr::column("error_message"));
        request
    }

    pub fn group_by_error_message_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("error_message")
            .aggregate_with_function("error_message", alias, function)
    }

    pub fn count_error_message(self) -> Self {
        self.count_error_message_as("error_message_count")
    }

    pub fn count_error_message_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("error_message", alias)
    }

    pub fn sum_error_message(self) -> Self {
        self.sum_error_message_as("sum_error_message")
    }

    pub fn sum_error_message_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("error_message", alias)
    }

    pub fn avg_error_message(self) -> Self {
        self.avg_error_message_as("avg_error_message")
    }

    pub fn avg_error_message_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("error_message", alias)
    }

    pub fn min_error_message(self) -> Self {
        self.min_error_message_as("min_error_message")
    }

    pub fn min_error_message_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("error_message", alias)
    }

    pub fn max_error_message(self) -> Self {
        self.max_error_message_as("max_error_message")
    }

    pub fn max_error_message_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("error_message", alias)
    }

    pub fn unselect_error_message(mut self) -> Self {
        self.query
            .projection
            .retain(|field| field != "error_message");
        self.query_options
            .raw_projections
            .retain(|projection| projection.property_name != "error_message");
        self
    }

    pub fn with_error_message(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "error_message",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_error_message_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "error_message",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_error_message_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("error_message", value));
        self
    }

    pub fn with_error_message_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("error_message", value));
        self
    }

    pub fn with_error_message_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("error_message", value));
        self
    }

    pub fn with_error_message_greater_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::gte("error_message", value));
        self
    }

    pub fn with_error_message_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("error_message", value));
        self
    }

    pub fn with_error_message_less_than_or_equal_to(
        mut self,
        value: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::lte("error_message", value));
        self
    }

    pub fn with_error_message_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self
            .query
            .and_filter(Expr::between("error_message", lower, upper));
        self
    }

    pub fn with_error_message_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self
            .query
            .and_filter(Expr::between("error_message", range.start, range.end));
        self
    }

    pub fn with_error_message_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "error_message",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_error_message_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "error_message",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_error_message_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("error_message", value));
        self
    }

    pub fn with_error_message_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_contain("error_message", value));
        self
    }

    pub fn with_error_message_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::begin_with("error_message", value));
        self
    }

    pub fn with_error_message_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_begin_with("error_message", value));
        self
    }

    pub fn with_error_message_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::end_with("error_message", value));
        self
    }

    pub fn with_error_message_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::not_end_with("error_message", value));
        self
    }

    pub fn with_error_message_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self
            .query
            .and_filter(Expr::sound_like("error_message", value));
        self
    }
    pub fn with_error_message_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("error_message", value));
        self
    }

    pub fn with_error_message_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("error_message", value));
        self
    }

    pub fn with_error_message_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("error_message"));
        self
    }

    pub fn with_error_message_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("error_message"));
        self
    }

    pub fn order_by_error_message_asc(mut self) -> Self {
        self.query = self.query.order_asc("error_message");
        self
    }

    pub fn order_by_error_message_desc(mut self) -> Self {
        self.query = self.query.order_desc("error_message");
        self
    }

    pub fn order_by_error_message_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("error_message");
        self
    }

    pub fn order_by_error_message_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("error_message");
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
}

impl<R> Default for ServiceLogRequest<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> From<ServiceLogRequest<R>> for SelectQuery {
    fn from(request: ServiceLogRequest<R>) -> Self {
        QuerySelection::from(request).into_query()
    }
}

impl<R> From<ServiceLogRequest<R>> for QuerySelection {
    fn from(request: ServiceLogRequest<R>) -> Self {
        Self {
            query: request.query,
            relation_selections: request.relation_selections,
            relation_filters: request.relation_filters,
            child_enhancements: request.child_enhancements,
            query_options: request.query_options,
        }
    }
}

impl<'a, C> crate::request_support::AuditedSave<'a, C> for teaql_core::Audited<crate::ServiceLog>
where
    C: crate::request_support::TeaqlRepositoryProvider + ?Sized + 'a,
{
    type Error = crate::TeaqlDataServiceError<C::ServiceLogRepository<'a>>;
    type Entity = crate::ServiceLog;
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

impl<R: teaql_core::Entity> crate::PurposedQuery<ServiceLogRequest<R>> {
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.inner.query_options.comment = Some(comment.into());
        self
    }

    pub fn new_entity<C>(&self, context: &C) -> crate::ServiceLog
    where
        C: crate::TeaqlRuntime + ?Sized,
    {
        self.require_comment();
        let mut entity =
            crate::ServiceLog::runtime_new(context.user_context().entity_runtime_state());
        if let Ok(id) = context
            .user_context()
            .next_id(crate::ServiceLog::ENTITY_NAME)
        {
            entity.update_id(id);
        }
        teaql_core::Entity::mark_as_new(&mut entity);
        entity
    }

    fn into_inner_with_trace(mut self) -> ServiceLogRequest<R> {
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
        crate::request_support::TeaqlDataServiceError<C::ServiceLogRepository<'a>>,
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
    ) -> Result<bool, crate::request_support::TeaqlDataServiceError<C::ServiceLogRepository<'a>>>
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
        crate::request_support::TeaqlDataServiceError<C::ServiceLogRepository<'a>>,
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
        crate::request_support::TeaqlDataServiceError<C::ServiceLogRepository<'a>>,
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
            crate::request_support::TeaqlDataServiceError<C::ServiceLogRepository<'a>>,
        >,
        crate::request_support::TeaqlDataServiceError<C::ServiceLogRepository<'a>>,
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
    ) -> Result<Option<R>, crate::request_support::TeaqlDataServiceError<C::ServiceLogRepository<'a>>>
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
    ) -> Result<Option<R>, crate::request_support::TeaqlDataServiceError<C::ServiceLogRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_one(context).await
    }

    pub async fn execute_for_count<'a, C>(
        self,
        context: &'a C,
    ) -> Result<u64, crate::request_support::TeaqlDataServiceError<C::ServiceLogRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()
            ._execute_for_count(context)
            .await
    }
}
