use std::marker::PhantomData;

use serde_json::Value as JsonValue;
use teaql_core::{Aggregate, AggregateFunction, EntityDescriptor, Expr, Record, SelectQuery, SmartList};
use teaql_runtime::{DataServiceError, RuntimeError};

use crate::request_support::*;

impl EntityReference for crate::Platform {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(&self)
    }
}

impl EntityReference for &crate::Platform {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(self)
    }
}

// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/platform
#[derive(Debug)]
pub struct PlatformRequest<R = crate::Platform> {
    query: SelectQuery,
    relation_selections: Vec<RelationSelection>,
    relation_filters: Vec<RelationFilter>,
    child_enhancements: Vec<QuerySelection>,
    query_options: QueryOptions,
    marker: PhantomData<R>,
}

impl<R> Clone for PlatformRequest<R> {
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

impl<R> PlatformRequest<R> {
    pub(crate) fn new() -> Self {
        Self {
            query: SelectQuery::new("Platform")
                .project("id"),
            relation_selections: Vec::new(),
            relation_filters: Vec::new(),
            child_enhancements: Vec::new(),
            query_options: QueryOptions::default(),
            marker: PhantomData,
        }
    }

    pub fn return_type<T>(self) -> PlatformRequest<T> {
        PlatformRequest {
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
        ctx: &'a C,
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        let repository = ctx
            .platform_repository()
            .map_err(|err| DataServiceError::Runtime(RuntimeError::Graph(err.to_string())))?;
        let query_options = self.query_options.clone();
        let relation_aggregates = runtime_relation_aggregates(&query_options);
        let query = authorize_query(apply_runtime_metadata(
            self.query,
            &query_options,
            &self.child_enhancements,
        )).map_err(DataServiceError::Runtime)?;
        let mut rows = repository.fetch_enhanced_entities_with_relation_aggregates::<R>(
            &query,
            &relation_aggregates,
        ).await?;
        let facets = execute_facets(ctx, query.as_query(), &query_options)
            .await
            .map_err(DataServiceError::Runtime)?;
        attach_facets(&mut rows, facets);
        Ok(rows)
    }

    pub(crate) async fn _execute_for_stream<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<TeaqlEntityStream<'a, R, TeaqlDataServiceError<C::PlatformRepository<'a>>>, TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity + 'a,
    {
        Ok(Box::pin(async_stream::try_stream! {
            use futures_util::StreamExt;
            let repository = ctx
                .platform_repository()
                .map_err(|err| DataServiceError::Runtime(RuntimeError::Graph(err.to_string())))?;
            let query_options = self.query_options.clone();
            let query = authorize_query(apply_runtime_metadata(
                self.query,
                &query_options,
                &self.child_enhancements,
            )).map_err(DataServiceError::Runtime)?;
            let mut chunks = repository.fetch_stream(&query).await?;
            while let Some(chunk) = chunks.next().await {
                for record in chunk?.rows {
                    yield R::from_record(record).map_err(DataServiceError::Entity)?;
                }
            }
        }))
    }

    pub(crate) async fn _execute_for_first<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<Option<R>, TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        let rows = self.limit(1)._execute_for_list(ctx).await?;
        Ok(rows.into_iter().next())
    }

    pub(crate) async fn _execute_for_one<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<Option<R>, TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        self._execute_for_first(ctx).await
    }


    pub(crate) async fn _execute_for_page<'a, C>(
        self,
        ctx: &'a C,
        offset: u64,
        limit: u64,
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        let total_count = self.clone()._execute_for_count(ctx).await?;
        let mut rows = self.page_offset(offset, limit)._execute_for_list(ctx).await?;
        rows.total_count = Some(total_count);
        Ok(rows)
    }

    pub(crate) async fn _execute_for_count<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<u64, TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = ctx
            .platform_repository()
            .map_err(|err| DataServiceError::Runtime(RuntimeError::Graph(err.to_string())))?;
        let mut query = self.query;
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
            .ok_or_else(|| DataServiceError::Runtime(RuntimeError::Graph(format!("count result for Platform is missing or not numeric"))))
    }

    pub(crate) async fn _execute_for_exists<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<bool, TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = ctx
            .platform_repository()
            .map_err(|err| DataServiceError::Runtime(RuntimeError::Graph(err.to_string())))?;
        let mut query = self.query.limit(1);
        query.relations.clear();
        let query = authorize_query(query).map_err(DataServiceError::Runtime)?;
        let rows = repository.fetch_all(&query).await?;
        Ok(!rows.is_empty())
    }

    pub(crate) async fn _execute_for_records<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<SmartList<Record>, TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = ctx
            .platform_repository()
            .map_err(|err| DataServiceError::Runtime(RuntimeError::Graph(err.to_string())))?;
        let query_options = self.query_options.clone();
        let outer_query = self.query.clone();
        let relation_aggregates = runtime_relation_aggregates(&query_options);
        let query = authorize_query(apply_runtime_metadata(
            self.query,
            &query_options,
            &self.child_enhancements,
        )).map_err(DataServiceError::Runtime)?;
        let mut rows = repository.fetch_smart_list_with_relation_aggregates(&query, &relation_aggregates).await?;
        let facets = execute_facets(ctx, &outer_query, &query_options)
            .await
            .map_err(DataServiceError::Runtime)?;
        attach_facets(&mut rows, facets);
        Ok(rows)
    }

    pub(crate) async fn _execute_for_record<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<Option<Record>, TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let records = self.limit(1)._execute_for_records(ctx).await?;
        Ok(records.into_iter().next())
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
        self.query = self.query.and_filter(Expr::in_list(TYPE_FIELD, types.into_iter().map(Into::into)));
        self
    }


    pub fn with_type_group(mut self) -> Self {
        self.query = self.query.project(TYPE_GROUP_FIELD);
        self
    }

    pub fn matching_any_of(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        let entity = EntityDescriptor::new(selection.query.entity.clone());
        self.query = self.query.and_filter(Expr::in_subquery("id", entity, selection.query.clone(), "id"));
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
        self.query_options.raw_sql_search_criteria.push(raw_sql.into_sql());
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
            "name" => Some("name"),
            "version" => Some("version"),
            _ => None,
        }
    }

    fn apply_dynamic_json_chain_filter(self, head: &str, tail: &str, value: &JsonValue) -> Self {
        let _ = (tail, value);
        match head {
            "repository_type_list" => {
                self.with_repository_type_list_matching(
                    crate::Q::repository_types_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "repository_format_list" => {
                self.with_repository_format_list_matching(
                    crate::Q::repository_formats_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "write_policy_list" => {
                self.with_write_policy_list_matching(
                    crate::Q::write_policies_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "blob_store_type_list" => {
                self.with_blob_store_type_list_matching(
                    crate::Q::blob_store_types_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "user_status_list" => {
                self.with_user_status_list_matching(
                    crate::Q::user_statuses_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "blob_store_configuration_list" => {
                self.with_blob_store_configuration_list_matching(
                    crate::Q::blob_store_configurations_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "repository_configuration_list" => {
                self.with_repository_configuration_list_matching(
                    crate::Q::repository_configurations_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "security_user_list" => {
                self.with_security_user_list_matching(
                    crate::Q::security_users_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "security_role_list" => {
                self.with_security_role_list_matching(
                    crate::Q::security_roles_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "security_privilege_list" => {
                self.with_security_privilege_list_matching(
                    crate::Q::security_privileges_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
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
        self.query = self.query.project("name");
        self.query = self.query.project("version");
        self
    }

    pub fn select_self_fields(self) -> Self {
        self.select_self()
    }

    pub fn select_self_without_parent(self) -> Self {
        self.select_self_fields()
    }

    pub fn select_all(self) -> Self {
        self.select_self()
    }

    pub fn select_children(self) -> Self {
        let mut request = self.select_all();
        request = request.select_repository_type_list();
        request = request.select_repository_format_list();
        request = request.select_write_policy_list();
        request = request.select_blob_store_type_list();
        request = request.select_user_status_list();
        request = request.select_blob_store_configuration_list();
        request = request.select_repository_configuration_list();
        request = request.select_security_user_list();
        request = request.select_security_role_list();
        request = request.select_security_privilege_list();
        request
    }

    pub fn select_any(self) -> Self {
        self.select_children()
    }

    pub fn group_by(mut self, field: impl Into<String>) -> Self {
        self.query = self.query.group_by(field);
        self
    }

    pub fn aggregate_count(mut self, alias: impl Into<String>) -> Self {
        self.query = self.query.count(alias);
        self
    }

    pub fn aggregate_count_field(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
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

    pub fn aggregate_stddev_pop(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
        self.query = self.query.stddev_pop(field, alias);
        self
    }

    pub fn aggregate_var_samp(mut self, field: impl Into<String>, alias: impl Into<String>) -> Self {
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
        self.query = self.query.enable_aggregation_cache_for(cache_expired_millis);
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
        request.query = request
            .query
            .project_expr(alias, Expr::column("id"));
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
        field_operator_expr(
            "id",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
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
        self.query = self.query.and_filter(Expr::in_list(
            "id",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_id_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "id",
            values.into_iter().map(Into::into),
        ));
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


    pub fn select_name(mut self) -> Self {
        self.query = self.query.project("name");
        self
    }

    pub fn project_name(self) -> Self {
        self.select_name()
    }

    pub fn select_name_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_name_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_name_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("name", raw_sql_segment));
        self
    }

    pub fn group_by_name(self) -> Self {
        self.group_by("name")
    }

    pub fn group_by_name_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("name");
        request.query = request
            .query
            .project_expr(alias, Expr::column("name"));
        request
    }

    pub fn group_by_name_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("name")
            .aggregate_with_function("name", alias, function)
    }

    pub fn count_name(self) -> Self {
        self.count_name_as("name_count")
    }

    pub fn count_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("name", alias)
    }

    pub fn sum_name(self) -> Self {
        self.sum_name_as("sum_name")
    }

    pub fn sum_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("name", alias)
    }

    pub fn avg_name(self) -> Self {
        self.avg_name_as("avg_name")
    }

    pub fn avg_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("name", alias)
    }

    pub fn min_name(self) -> Self {
        self.min_name_as("min_name")
    }

    pub fn min_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("name", alias)
    }

    pub fn max_name(self) -> Self {
        self.max_name_as("max_name")
    }

    pub fn max_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("name", alias)
    }

    pub fn unselect_name(mut self) -> Self {
        self.query.projection.retain(|field| field != "name");
        self.query_options.raw_projections.retain(|projection| projection.property_name != "name");
        self
    }


    pub fn with_name(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "name",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_name_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "name",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_name_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("name", value));
        self
    }



    pub fn with_name_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("name", value));
        self
    }

    pub fn with_name_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("name", value));
        self
    }

    pub fn with_name_greater_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gte("name", value));
        self
    }

    pub fn with_name_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("name", value));
        self
    }

    pub fn with_name_less_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lte("name", value));
        self
    }

    pub fn with_name_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("name", lower, upper));
        self
    }

    pub fn with_name_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "name",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_name_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "name",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_name_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "name",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_name_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("name", value));
        self
    }

    pub fn with_name_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("name", value));
        self
    }

    pub fn with_name_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("name", value));
        self
    }

    pub fn with_name_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("name", value));
        self
    }

    pub fn with_name_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("name", value));
        self
    }

    pub fn with_name_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("name", value));
        self
    }

    pub fn with_name_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("name", value));
        self
    }
    pub fn with_name_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("name", value));
        self
    }

    pub fn with_name_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("name", value));
        self
    }

    pub fn with_name_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("name"));
        self
    }



    pub fn with_name_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("name"));
        self
    }


    pub fn order_by_name_asc(mut self) -> Self {
        self.query = self.query.order_asc("name");
        self
    }

    pub fn order_by_name_desc(mut self) -> Self {
        self.query = self.query.order_desc("name");
        self
    }

    pub fn order_by_name_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("name");
        self
    }

    pub fn order_by_name_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("name");
        self
    }


    pub fn select_version(mut self) -> Self {
        self.query = self.query.project("version");
        self
    }

    pub fn project_version(self) -> Self {
        self.select_version()
    }

    pub fn select_version_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_version_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_version_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("version", raw_sql_segment));
        self
    }

    pub fn group_by_version(self) -> Self {
        self.group_by("version")
    }

    pub fn group_by_version_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("version");
        request.query = request
            .query
            .project_expr(alias, Expr::column("version"));
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

    pub fn unselect_version(mut self) -> Self {
        self.query.projection.retain(|field| field != "version");
        self.query_options.raw_projections.retain(|projection| projection.property_name != "version");
        self
    }


    pub fn with_version(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "version",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_version_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "version",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_version_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("version", value));
        self
    }



    pub fn with_version_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("version", value));
        self
    }

    pub fn with_version_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("version", value));
        self
    }

    pub fn with_version_greater_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gte("version", value));
        self
    }

    pub fn with_version_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("version", value));
        self
    }

    pub fn with_version_less_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lte("version", value));
        self
    }

    pub fn with_version_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("version", lower, upper));
        self
    }

    pub fn with_version_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "version",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_version_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "version",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_version_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "version",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_version_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("version", value));
        self
    }

    pub fn with_version_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("version", value));
        self
    }

    pub fn with_version_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("version", value));
        self
    }

    pub fn with_version_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("version", value));
        self
    }

    pub fn with_version_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("version", value));
        self
    }

    pub fn with_version_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("version", value));
        self
    }

    pub fn with_version_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("version", value));
        self
    }
    pub fn with_version_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("version", value));
        self
    }

    pub fn with_version_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("version", value));
        self
    }

    pub fn with_version_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("version"));
        self
    }



    pub fn with_version_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("version"));
        self
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
    pub fn name_is_nexus_repository_platform(self) -> Self {
        self.with_name_is("Nexus Repository Platform")
    }

    pub fn with_name_is_nexus_repository_platform(self) -> Self {
        self.with_name_is("Nexus Repository Platform")
    }



    pub fn with_name_is_not_nexus_repository_platform(self) -> Self {
        self.with_name_is_not("Nexus Repository Platform")
    }



    pub fn version_is_value_3_95_1_01(self) -> Self {
        self.with_version_is("3.95.1-01")
    }

    pub fn with_version_is_value_3_95_1_01(self) -> Self {
        self.with_version_is("3.95.1-01")
    }



    pub fn with_version_is_not_value_3_95_1_01(self) -> Self {
        self.with_version_is_not("3.95.1-01")
    }


    pub fn have_repository_types(self) -> Self {
        self.with_repository_type_list_matching(SelectQuery::new("RepositoryType"))
    }

    pub fn have_no_repository_types(self) -> Self {
        self.without_repository_type_list_matching(SelectQuery::new("RepositoryType"))
    }

    pub fn with_repository_type_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            <crate::RepositoryType as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("repository_type_list", selection));
        self
    }

    pub fn without_repository_type_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "id",
            <crate::RepositoryType as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("repository_type_list", selection));
        self
    }

    pub fn select_repository_type_list(mut self) -> Self {
        self.query = self.query.relation("repository_type_list");
        self
    }

    pub fn select_repository_type_list_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("repository_type_list", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("repository_type_list", selection));
        self
}

    pub fn have_repository_formats(self) -> Self {
        self.with_repository_format_list_matching(SelectQuery::new("RepositoryFormat"))
    }

    pub fn have_no_repository_formats(self) -> Self {
        self.without_repository_format_list_matching(SelectQuery::new("RepositoryFormat"))
    }

    pub fn with_repository_format_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            <crate::RepositoryFormat as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("repository_format_list", selection));
        self
    }

    pub fn without_repository_format_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "id",
            <crate::RepositoryFormat as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("repository_format_list", selection));
        self
    }

    pub fn select_repository_format_list(mut self) -> Self {
        self.query = self.query.relation("repository_format_list");
        self
    }

    pub fn select_repository_format_list_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("repository_format_list", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("repository_format_list", selection));
        self
}

    pub fn have_write_policies(self) -> Self {
        self.with_write_policy_list_matching(SelectQuery::new("WritePolicy"))
    }

    pub fn have_no_write_policies(self) -> Self {
        self.without_write_policy_list_matching(SelectQuery::new("WritePolicy"))
    }

    pub fn with_write_policy_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            <crate::WritePolicy as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("write_policy_list", selection));
        self
    }

    pub fn without_write_policy_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "id",
            <crate::WritePolicy as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("write_policy_list", selection));
        self
    }

    pub fn select_write_policy_list(mut self) -> Self {
        self.query = self.query.relation("write_policy_list");
        self
    }

    pub fn select_write_policy_list_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("write_policy_list", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("write_policy_list", selection));
        self
}

    pub fn have_blob_store_types(self) -> Self {
        self.with_blob_store_type_list_matching(SelectQuery::new("BlobStoreType"))
    }

    pub fn have_no_blob_store_types(self) -> Self {
        self.without_blob_store_type_list_matching(SelectQuery::new("BlobStoreType"))
    }

    pub fn with_blob_store_type_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            <crate::BlobStoreType as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("blob_store_type_list", selection));
        self
    }

    pub fn without_blob_store_type_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "id",
            <crate::BlobStoreType as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("blob_store_type_list", selection));
        self
    }

    pub fn select_blob_store_type_list(mut self) -> Self {
        self.query = self.query.relation("blob_store_type_list");
        self
    }

    pub fn select_blob_store_type_list_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("blob_store_type_list", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("blob_store_type_list", selection));
        self
}

    pub fn have_user_statuses(self) -> Self {
        self.with_user_status_list_matching(SelectQuery::new("UserStatus"))
    }

    pub fn have_no_user_statuses(self) -> Self {
        self.without_user_status_list_matching(SelectQuery::new("UserStatus"))
    }

    pub fn with_user_status_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            <crate::UserStatus as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("user_status_list", selection));
        self
    }

    pub fn without_user_status_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "id",
            <crate::UserStatus as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("user_status_list", selection));
        self
    }

    pub fn select_user_status_list(mut self) -> Self {
        self.query = self.query.relation("user_status_list");
        self
    }

    pub fn select_user_status_list_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("user_status_list", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("user_status_list", selection));
        self
}

    pub fn have_blob_store_configurations(self) -> Self {
        self.with_blob_store_configuration_list_matching(SelectQuery::new("BlobStoreConfiguration"))
    }

    pub fn have_no_blob_store_configurations(self) -> Self {
        self.without_blob_store_configuration_list_matching(SelectQuery::new("BlobStoreConfiguration"))
    }

    pub fn with_blob_store_configuration_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            <crate::BlobStoreConfiguration as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("blob_store_configuration_list", selection));
        self
    }

    pub fn without_blob_store_configuration_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "id",
            <crate::BlobStoreConfiguration as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("blob_store_configuration_list", selection));
        self
    }

    pub fn select_blob_store_configuration_list(mut self) -> Self {
        self.query = self.query.relation("blob_store_configuration_list");
        self
    }

    pub fn select_blob_store_configuration_list_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("blob_store_configuration_list", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("blob_store_configuration_list", selection));
        self
}

    pub fn have_repository_configurations(self) -> Self {
        self.with_repository_configuration_list_matching(SelectQuery::new("RepositoryConfiguration"))
    }

    pub fn have_no_repository_configurations(self) -> Self {
        self.without_repository_configuration_list_matching(SelectQuery::new("RepositoryConfiguration"))
    }

    pub fn with_repository_configuration_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            <crate::RepositoryConfiguration as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("repository_configuration_list", selection));
        self
    }

    pub fn without_repository_configuration_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "id",
            <crate::RepositoryConfiguration as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("repository_configuration_list", selection));
        self
    }

    pub fn select_repository_configuration_list(mut self) -> Self {
        self.query = self.query.relation("repository_configuration_list");
        self
    }

    pub fn select_repository_configuration_list_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("repository_configuration_list", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("repository_configuration_list", selection));
        self
}

    pub fn have_security_users(self) -> Self {
        self.with_security_user_list_matching(SelectQuery::new("SecurityUser"))
    }

    pub fn have_no_security_users(self) -> Self {
        self.without_security_user_list_matching(SelectQuery::new("SecurityUser"))
    }

    pub fn with_security_user_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            <crate::SecurityUser as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("security_user_list", selection));
        self
    }

    pub fn without_security_user_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "id",
            <crate::SecurityUser as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("security_user_list", selection));
        self
    }

    pub fn select_security_user_list(mut self) -> Self {
        self.query = self.query.relation("security_user_list");
        self
    }

    pub fn select_security_user_list_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("security_user_list", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("security_user_list", selection));
        self
}

    pub fn have_security_roles(self) -> Self {
        self.with_security_role_list_matching(SelectQuery::new("SecurityRole"))
    }

    pub fn have_no_security_roles(self) -> Self {
        self.without_security_role_list_matching(SelectQuery::new("SecurityRole"))
    }

    pub fn with_security_role_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            <crate::SecurityRole as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("security_role_list", selection));
        self
    }

    pub fn without_security_role_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "id",
            <crate::SecurityRole as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("security_role_list", selection));
        self
    }

    pub fn select_security_role_list(mut self) -> Self {
        self.query = self.query.relation("security_role_list");
        self
    }

    pub fn select_security_role_list_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("security_role_list", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("security_role_list", selection));
        self
}

    pub fn have_security_privileges(self) -> Self {
        self.with_security_privilege_list_matching(SelectQuery::new("SecurityPrivilege"))
    }

    pub fn have_no_security_privileges(self) -> Self {
        self.without_security_privilege_list_matching(SelectQuery::new("SecurityPrivilege"))
    }

    pub fn with_security_privilege_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            <crate::SecurityPrivilege as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("security_privilege_list", selection));
        self
    }

    pub fn without_security_privilege_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "id",
            <crate::SecurityPrivilege as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "platform_id",
        ));
        self.relation_filters.push(RelationFilter::new("security_privilege_list", selection));
        self
    }

    pub fn select_security_privilege_list(mut self) -> Self {
        self.query = self.query.relation("security_privilege_list");
        self
    }

    pub fn select_security_privilege_list_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("security_privilege_list", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("security_privilege_list", selection));
        self
}
    pub fn count_repository_types(self) -> Self {
        self.count_repository_types_as("count_repository_types")
    }

    pub fn count_repository_types_as(self, alias: impl Into<String>) -> Self {
        self.count_repository_types_with(alias, crate::Q::repository_types().unlimited())
    }

    pub fn count_repository_types_with(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "repository_type_list",
            alias,
            selection,
            true,
        ));
        self
    }

    pub fn stats_from_repository_types(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_types_as("refinements", request)
    }

    pub fn stats_from_repository_types_as(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "repository_type_list",
            alias,
            selection,
            false,
        ));
        self
    }

    pub fn group_by_repository_types_with_details(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_types(request)
    }


    pub fn sum_display_order_of_repository_types(self) -> Self {
        self.sum_display_order_of_repository_types_as("sum_display_order_of_repository_types", crate::Q::repository_types().unlimited())
    }

    pub fn sum_display_order_of_repository_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_types_as(alias, request.into().into_query().sum("display_order", "sum_display_order"))
    }
    pub fn min_display_order_of_repository_types(self) -> Self {
        self.min_display_order_of_repository_types_as("min_display_order_of_repository_types", crate::Q::repository_types().unlimited())
    }

    pub fn min_display_order_of_repository_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_types_as(alias, request.into().into_query().min("display_order", "min_display_order"))
    }
    pub fn max_display_order_of_repository_types(self) -> Self {
        self.max_display_order_of_repository_types_as("max_display_order_of_repository_types", crate::Q::repository_types().unlimited())
    }

    pub fn max_display_order_of_repository_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_types_as(alias, request.into().into_query().max("display_order", "max_display_order"))
    }
    pub fn avg_display_order_of_repository_types(self) -> Self {
        self.avg_display_order_of_repository_types_as("avg_display_order_of_repository_types", crate::Q::repository_types().unlimited())
    }

    pub fn avg_display_order_of_repository_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_types_as(alias, request.into().into_query().avg("display_order", "avg_display_order"))
    }
    pub fn standard_deviation_display_order_of_repository_types(self) -> Self {
        self.standard_deviation_display_order_of_repository_types_as("standard_deviation_display_order_of_repository_types", crate::Q::repository_types().unlimited())
    }

    pub fn standard_deviation_display_order_of_repository_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_types_as(alias, request.into().into_query().stddev("display_order", "stdDev_display_order"))
    }
    pub fn square_root_of_population_standard_deviation_display_order_of_repository_types(self) -> Self {
        self.square_root_of_population_standard_deviation_display_order_of_repository_types_as("square_root_of_population_standard_deviation_display_order_of_repository_types", crate::Q::repository_types().unlimited())
    }

    pub fn square_root_of_population_standard_deviation_display_order_of_repository_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_types_as(alias, request.into().into_query().stddev_pop("display_order", "stdDevPop_display_order"))
    }
    pub fn sample_variance_display_order_of_repository_types(self) -> Self {
        self.sample_variance_display_order_of_repository_types_as("sample_variance_display_order_of_repository_types", crate::Q::repository_types().unlimited())
    }

    pub fn sample_variance_display_order_of_repository_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_types_as(alias, request.into().into_query().var_samp("display_order", "varSamp_display_order"))
    }
    pub fn sample_population_variance_display_order_of_repository_types(self) -> Self {
        self.sample_population_variance_display_order_of_repository_types_as("sample_population_variance_display_order_of_repository_types", crate::Q::repository_types().unlimited())
    }

    pub fn sample_population_variance_display_order_of_repository_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_types_as(alias, request.into().into_query().var_pop("display_order", "varPop_display_order"))
    }

    pub fn count_repository_formats(self) -> Self {
        self.count_repository_formats_as("count_repository_formats")
    }

    pub fn count_repository_formats_as(self, alias: impl Into<String>) -> Self {
        self.count_repository_formats_with(alias, crate::Q::repository_formats().unlimited())
    }

    pub fn count_repository_formats_with(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "repository_format_list",
            alias,
            selection,
            true,
        ));
        self
    }

    pub fn stats_from_repository_formats(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_formats_as("refinements", request)
    }

    pub fn stats_from_repository_formats_as(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "repository_format_list",
            alias,
            selection,
            false,
        ));
        self
    }

    pub fn group_by_repository_formats_with_details(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_formats(request)
    }


    pub fn sum_display_order_of_repository_formats(self) -> Self {
        self.sum_display_order_of_repository_formats_as("sum_display_order_of_repository_formats", crate::Q::repository_formats().unlimited())
    }

    pub fn sum_display_order_of_repository_formats_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_formats_as(alias, request.into().into_query().sum("display_order", "sum_display_order"))
    }
    pub fn min_display_order_of_repository_formats(self) -> Self {
        self.min_display_order_of_repository_formats_as("min_display_order_of_repository_formats", crate::Q::repository_formats().unlimited())
    }

    pub fn min_display_order_of_repository_formats_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_formats_as(alias, request.into().into_query().min("display_order", "min_display_order"))
    }
    pub fn max_display_order_of_repository_formats(self) -> Self {
        self.max_display_order_of_repository_formats_as("max_display_order_of_repository_formats", crate::Q::repository_formats().unlimited())
    }

    pub fn max_display_order_of_repository_formats_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_formats_as(alias, request.into().into_query().max("display_order", "max_display_order"))
    }
    pub fn avg_display_order_of_repository_formats(self) -> Self {
        self.avg_display_order_of_repository_formats_as("avg_display_order_of_repository_formats", crate::Q::repository_formats().unlimited())
    }

    pub fn avg_display_order_of_repository_formats_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_formats_as(alias, request.into().into_query().avg("display_order", "avg_display_order"))
    }
    pub fn standard_deviation_display_order_of_repository_formats(self) -> Self {
        self.standard_deviation_display_order_of_repository_formats_as("standard_deviation_display_order_of_repository_formats", crate::Q::repository_formats().unlimited())
    }

    pub fn standard_deviation_display_order_of_repository_formats_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_formats_as(alias, request.into().into_query().stddev("display_order", "stdDev_display_order"))
    }
    pub fn square_root_of_population_standard_deviation_display_order_of_repository_formats(self) -> Self {
        self.square_root_of_population_standard_deviation_display_order_of_repository_formats_as("square_root_of_population_standard_deviation_display_order_of_repository_formats", crate::Q::repository_formats().unlimited())
    }

    pub fn square_root_of_population_standard_deviation_display_order_of_repository_formats_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_formats_as(alias, request.into().into_query().stddev_pop("display_order", "stdDevPop_display_order"))
    }
    pub fn sample_variance_display_order_of_repository_formats(self) -> Self {
        self.sample_variance_display_order_of_repository_formats_as("sample_variance_display_order_of_repository_formats", crate::Q::repository_formats().unlimited())
    }

    pub fn sample_variance_display_order_of_repository_formats_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_formats_as(alias, request.into().into_query().var_samp("display_order", "varSamp_display_order"))
    }
    pub fn sample_population_variance_display_order_of_repository_formats(self) -> Self {
        self.sample_population_variance_display_order_of_repository_formats_as("sample_population_variance_display_order_of_repository_formats", crate::Q::repository_formats().unlimited())
    }

    pub fn sample_population_variance_display_order_of_repository_formats_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_formats_as(alias, request.into().into_query().var_pop("display_order", "varPop_display_order"))
    }

    pub fn count_write_policies(self) -> Self {
        self.count_write_policies_as("count_write_policies")
    }

    pub fn count_write_policies_as(self, alias: impl Into<String>) -> Self {
        self.count_write_policies_with(alias, crate::Q::write_policies().unlimited())
    }

    pub fn count_write_policies_with(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "write_policy_list",
            alias,
            selection,
            true,
        ));
        self
    }

    pub fn stats_from_write_policies(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_write_policies_as("refinements", request)
    }

    pub fn stats_from_write_policies_as(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "write_policy_list",
            alias,
            selection,
            false,
        ));
        self
    }

    pub fn group_by_write_policies_with_details(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_write_policies(request)
    }


    pub fn sum_display_order_of_write_policies(self) -> Self {
        self.sum_display_order_of_write_policies_as("sum_display_order_of_write_policies", crate::Q::write_policies().unlimited())
    }

    pub fn sum_display_order_of_write_policies_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_write_policies_as(alias, request.into().into_query().sum("display_order", "sum_display_order"))
    }
    pub fn min_display_order_of_write_policies(self) -> Self {
        self.min_display_order_of_write_policies_as("min_display_order_of_write_policies", crate::Q::write_policies().unlimited())
    }

    pub fn min_display_order_of_write_policies_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_write_policies_as(alias, request.into().into_query().min("display_order", "min_display_order"))
    }
    pub fn max_display_order_of_write_policies(self) -> Self {
        self.max_display_order_of_write_policies_as("max_display_order_of_write_policies", crate::Q::write_policies().unlimited())
    }

    pub fn max_display_order_of_write_policies_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_write_policies_as(alias, request.into().into_query().max("display_order", "max_display_order"))
    }
    pub fn avg_display_order_of_write_policies(self) -> Self {
        self.avg_display_order_of_write_policies_as("avg_display_order_of_write_policies", crate::Q::write_policies().unlimited())
    }

    pub fn avg_display_order_of_write_policies_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_write_policies_as(alias, request.into().into_query().avg("display_order", "avg_display_order"))
    }
    pub fn standard_deviation_display_order_of_write_policies(self) -> Self {
        self.standard_deviation_display_order_of_write_policies_as("standard_deviation_display_order_of_write_policies", crate::Q::write_policies().unlimited())
    }

    pub fn standard_deviation_display_order_of_write_policies_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_write_policies_as(alias, request.into().into_query().stddev("display_order", "stdDev_display_order"))
    }
    pub fn square_root_of_population_standard_deviation_display_order_of_write_policies(self) -> Self {
        self.square_root_of_population_standard_deviation_display_order_of_write_policies_as("square_root_of_population_standard_deviation_display_order_of_write_policies", crate::Q::write_policies().unlimited())
    }

    pub fn square_root_of_population_standard_deviation_display_order_of_write_policies_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_write_policies_as(alias, request.into().into_query().stddev_pop("display_order", "stdDevPop_display_order"))
    }
    pub fn sample_variance_display_order_of_write_policies(self) -> Self {
        self.sample_variance_display_order_of_write_policies_as("sample_variance_display_order_of_write_policies", crate::Q::write_policies().unlimited())
    }

    pub fn sample_variance_display_order_of_write_policies_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_write_policies_as(alias, request.into().into_query().var_samp("display_order", "varSamp_display_order"))
    }
    pub fn sample_population_variance_display_order_of_write_policies(self) -> Self {
        self.sample_population_variance_display_order_of_write_policies_as("sample_population_variance_display_order_of_write_policies", crate::Q::write_policies().unlimited())
    }

    pub fn sample_population_variance_display_order_of_write_policies_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_write_policies_as(alias, request.into().into_query().var_pop("display_order", "varPop_display_order"))
    }

    pub fn count_blob_store_types(self) -> Self {
        self.count_blob_store_types_as("count_blob_store_types")
    }

    pub fn count_blob_store_types_as(self, alias: impl Into<String>) -> Self {
        self.count_blob_store_types_with(alias, crate::Q::blob_store_types().unlimited())
    }

    pub fn count_blob_store_types_with(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "blob_store_type_list",
            alias,
            selection,
            true,
        ));
        self
    }

    pub fn stats_from_blob_store_types(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_types_as("refinements", request)
    }

    pub fn stats_from_blob_store_types_as(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "blob_store_type_list",
            alias,
            selection,
            false,
        ));
        self
    }

    pub fn group_by_blob_store_types_with_details(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_types(request)
    }


    pub fn sum_display_order_of_blob_store_types(self) -> Self {
        self.sum_display_order_of_blob_store_types_as("sum_display_order_of_blob_store_types", crate::Q::blob_store_types().unlimited())
    }

    pub fn sum_display_order_of_blob_store_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_types_as(alias, request.into().into_query().sum("display_order", "sum_display_order"))
    }
    pub fn min_display_order_of_blob_store_types(self) -> Self {
        self.min_display_order_of_blob_store_types_as("min_display_order_of_blob_store_types", crate::Q::blob_store_types().unlimited())
    }

    pub fn min_display_order_of_blob_store_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_types_as(alias, request.into().into_query().min("display_order", "min_display_order"))
    }
    pub fn max_display_order_of_blob_store_types(self) -> Self {
        self.max_display_order_of_blob_store_types_as("max_display_order_of_blob_store_types", crate::Q::blob_store_types().unlimited())
    }

    pub fn max_display_order_of_blob_store_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_types_as(alias, request.into().into_query().max("display_order", "max_display_order"))
    }
    pub fn avg_display_order_of_blob_store_types(self) -> Self {
        self.avg_display_order_of_blob_store_types_as("avg_display_order_of_blob_store_types", crate::Q::blob_store_types().unlimited())
    }

    pub fn avg_display_order_of_blob_store_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_types_as(alias, request.into().into_query().avg("display_order", "avg_display_order"))
    }
    pub fn standard_deviation_display_order_of_blob_store_types(self) -> Self {
        self.standard_deviation_display_order_of_blob_store_types_as("standard_deviation_display_order_of_blob_store_types", crate::Q::blob_store_types().unlimited())
    }

    pub fn standard_deviation_display_order_of_blob_store_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_types_as(alias, request.into().into_query().stddev("display_order", "stdDev_display_order"))
    }
    pub fn square_root_of_population_standard_deviation_display_order_of_blob_store_types(self) -> Self {
        self.square_root_of_population_standard_deviation_display_order_of_blob_store_types_as("square_root_of_population_standard_deviation_display_order_of_blob_store_types", crate::Q::blob_store_types().unlimited())
    }

    pub fn square_root_of_population_standard_deviation_display_order_of_blob_store_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_types_as(alias, request.into().into_query().stddev_pop("display_order", "stdDevPop_display_order"))
    }
    pub fn sample_variance_display_order_of_blob_store_types(self) -> Self {
        self.sample_variance_display_order_of_blob_store_types_as("sample_variance_display_order_of_blob_store_types", crate::Q::blob_store_types().unlimited())
    }

    pub fn sample_variance_display_order_of_blob_store_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_types_as(alias, request.into().into_query().var_samp("display_order", "varSamp_display_order"))
    }
    pub fn sample_population_variance_display_order_of_blob_store_types(self) -> Self {
        self.sample_population_variance_display_order_of_blob_store_types_as("sample_population_variance_display_order_of_blob_store_types", crate::Q::blob_store_types().unlimited())
    }

    pub fn sample_population_variance_display_order_of_blob_store_types_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_types_as(alias, request.into().into_query().var_pop("display_order", "varPop_display_order"))
    }

    pub fn count_user_statuses(self) -> Self {
        self.count_user_statuses_as("count_user_statuses")
    }

    pub fn count_user_statuses_as(self, alias: impl Into<String>) -> Self {
        self.count_user_statuses_with(alias, crate::Q::user_statuses().unlimited())
    }

    pub fn count_user_statuses_with(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "user_status_list",
            alias,
            selection,
            true,
        ));
        self
    }

    pub fn stats_from_user_statuses(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_user_statuses_as("refinements", request)
    }

    pub fn stats_from_user_statuses_as(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "user_status_list",
            alias,
            selection,
            false,
        ));
        self
    }

    pub fn group_by_user_statuses_with_details(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_user_statuses(request)
    }


    pub fn sum_display_order_of_user_statuses(self) -> Self {
        self.sum_display_order_of_user_statuses_as("sum_display_order_of_user_statuses", crate::Q::user_statuses().unlimited())
    }

    pub fn sum_display_order_of_user_statuses_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_user_statuses_as(alias, request.into().into_query().sum("display_order", "sum_display_order"))
    }
    pub fn min_display_order_of_user_statuses(self) -> Self {
        self.min_display_order_of_user_statuses_as("min_display_order_of_user_statuses", crate::Q::user_statuses().unlimited())
    }

    pub fn min_display_order_of_user_statuses_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_user_statuses_as(alias, request.into().into_query().min("display_order", "min_display_order"))
    }
    pub fn max_display_order_of_user_statuses(self) -> Self {
        self.max_display_order_of_user_statuses_as("max_display_order_of_user_statuses", crate::Q::user_statuses().unlimited())
    }

    pub fn max_display_order_of_user_statuses_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_user_statuses_as(alias, request.into().into_query().max("display_order", "max_display_order"))
    }
    pub fn avg_display_order_of_user_statuses(self) -> Self {
        self.avg_display_order_of_user_statuses_as("avg_display_order_of_user_statuses", crate::Q::user_statuses().unlimited())
    }

    pub fn avg_display_order_of_user_statuses_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_user_statuses_as(alias, request.into().into_query().avg("display_order", "avg_display_order"))
    }
    pub fn standard_deviation_display_order_of_user_statuses(self) -> Self {
        self.standard_deviation_display_order_of_user_statuses_as("standard_deviation_display_order_of_user_statuses", crate::Q::user_statuses().unlimited())
    }

    pub fn standard_deviation_display_order_of_user_statuses_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_user_statuses_as(alias, request.into().into_query().stddev("display_order", "stdDev_display_order"))
    }
    pub fn square_root_of_population_standard_deviation_display_order_of_user_statuses(self) -> Self {
        self.square_root_of_population_standard_deviation_display_order_of_user_statuses_as("square_root_of_population_standard_deviation_display_order_of_user_statuses", crate::Q::user_statuses().unlimited())
    }

    pub fn square_root_of_population_standard_deviation_display_order_of_user_statuses_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_user_statuses_as(alias, request.into().into_query().stddev_pop("display_order", "stdDevPop_display_order"))
    }
    pub fn sample_variance_display_order_of_user_statuses(self) -> Self {
        self.sample_variance_display_order_of_user_statuses_as("sample_variance_display_order_of_user_statuses", crate::Q::user_statuses().unlimited())
    }

    pub fn sample_variance_display_order_of_user_statuses_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_user_statuses_as(alias, request.into().into_query().var_samp("display_order", "varSamp_display_order"))
    }
    pub fn sample_population_variance_display_order_of_user_statuses(self) -> Self {
        self.sample_population_variance_display_order_of_user_statuses_as("sample_population_variance_display_order_of_user_statuses", crate::Q::user_statuses().unlimited())
    }

    pub fn sample_population_variance_display_order_of_user_statuses_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_user_statuses_as(alias, request.into().into_query().var_pop("display_order", "varPop_display_order"))
    }

    pub fn count_blob_store_configurations(self) -> Self {
        self.count_blob_store_configurations_as("count_blob_store_configurations")
    }

    pub fn count_blob_store_configurations_as(self, alias: impl Into<String>) -> Self {
        self.count_blob_store_configurations_with(alias, crate::Q::blob_store_configurations().unlimited())
    }

    pub fn count_blob_store_configurations_with(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "blob_store_configuration_list",
            alias,
            selection,
            true,
        ));
        self
    }

    pub fn stats_from_blob_store_configurations(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as("refinements", request)
    }

    pub fn stats_from_blob_store_configurations_as(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "blob_store_configuration_list",
            alias,
            selection,
            false,
        ));
        self
    }

    pub fn group_by_blob_store_configurations_with_details(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations(request)
    }


    pub fn sum_total_size_of_blob_store_configurations(self) -> Self {
        self.sum_total_size_of_blob_store_configurations_as("sum_total_size_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn sum_total_size_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().sum("total_size", "sum_total_size"))
    }
    pub fn min_total_size_of_blob_store_configurations(self) -> Self {
        self.min_total_size_of_blob_store_configurations_as("min_total_size_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn min_total_size_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().min("total_size", "min_total_size"))
    }
    pub fn max_total_size_of_blob_store_configurations(self) -> Self {
        self.max_total_size_of_blob_store_configurations_as("max_total_size_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn max_total_size_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().max("total_size", "max_total_size"))
    }
    pub fn avg_total_size_of_blob_store_configurations(self) -> Self {
        self.avg_total_size_of_blob_store_configurations_as("avg_total_size_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn avg_total_size_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().avg("total_size", "avg_total_size"))
    }
    pub fn standard_deviation_total_size_of_blob_store_configurations(self) -> Self {
        self.standard_deviation_total_size_of_blob_store_configurations_as("standard_deviation_total_size_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn standard_deviation_total_size_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().stddev("total_size", "stdDev_total_size"))
    }
    pub fn square_root_of_population_standard_deviation_total_size_of_blob_store_configurations(self) -> Self {
        self.square_root_of_population_standard_deviation_total_size_of_blob_store_configurations_as("square_root_of_population_standard_deviation_total_size_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn square_root_of_population_standard_deviation_total_size_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().stddev_pop("total_size", "stdDevPop_total_size"))
    }
    pub fn sample_variance_total_size_of_blob_store_configurations(self) -> Self {
        self.sample_variance_total_size_of_blob_store_configurations_as("sample_variance_total_size_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn sample_variance_total_size_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().var_samp("total_size", "varSamp_total_size"))
    }
    pub fn sample_population_variance_total_size_of_blob_store_configurations(self) -> Self {
        self.sample_population_variance_total_size_of_blob_store_configurations_as("sample_population_variance_total_size_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn sample_population_variance_total_size_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().var_pop("total_size", "varPop_total_size"))
    }
    pub fn sum_blob_count_of_blob_store_configurations(self) -> Self {
        self.sum_blob_count_of_blob_store_configurations_as("sum_blob_count_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn sum_blob_count_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().sum("blob_count", "sum_blob_count"))
    }
    pub fn min_blob_count_of_blob_store_configurations(self) -> Self {
        self.min_blob_count_of_blob_store_configurations_as("min_blob_count_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn min_blob_count_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().min("blob_count", "min_blob_count"))
    }
    pub fn max_blob_count_of_blob_store_configurations(self) -> Self {
        self.max_blob_count_of_blob_store_configurations_as("max_blob_count_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn max_blob_count_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().max("blob_count", "max_blob_count"))
    }
    pub fn avg_blob_count_of_blob_store_configurations(self) -> Self {
        self.avg_blob_count_of_blob_store_configurations_as("avg_blob_count_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn avg_blob_count_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().avg("blob_count", "avg_blob_count"))
    }
    pub fn standard_deviation_blob_count_of_blob_store_configurations(self) -> Self {
        self.standard_deviation_blob_count_of_blob_store_configurations_as("standard_deviation_blob_count_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn standard_deviation_blob_count_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().stddev("blob_count", "stdDev_blob_count"))
    }
    pub fn square_root_of_population_standard_deviation_blob_count_of_blob_store_configurations(self) -> Self {
        self.square_root_of_population_standard_deviation_blob_count_of_blob_store_configurations_as("square_root_of_population_standard_deviation_blob_count_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn square_root_of_population_standard_deviation_blob_count_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().stddev_pop("blob_count", "stdDevPop_blob_count"))
    }
    pub fn sample_variance_blob_count_of_blob_store_configurations(self) -> Self {
        self.sample_variance_blob_count_of_blob_store_configurations_as("sample_variance_blob_count_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn sample_variance_blob_count_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().var_samp("blob_count", "varSamp_blob_count"))
    }
    pub fn sample_population_variance_blob_count_of_blob_store_configurations(self) -> Self {
        self.sample_population_variance_blob_count_of_blob_store_configurations_as("sample_population_variance_blob_count_of_blob_store_configurations", crate::Q::blob_store_configurations().unlimited())
    }

    pub fn sample_population_variance_blob_count_of_blob_store_configurations_as(self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_blob_store_configurations_as(alias, request.into().into_query().var_pop("blob_count", "varPop_blob_count"))
    }

    pub fn count_repository_configurations(self) -> Self {
        self.count_repository_configurations_as("count_repository_configurations")
    }

    pub fn count_repository_configurations_as(self, alias: impl Into<String>) -> Self {
        self.count_repository_configurations_with(alias, crate::Q::repository_configurations().unlimited())
    }

    pub fn count_repository_configurations_with(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "repository_configuration_list",
            alias,
            selection,
            true,
        ));
        self
    }

    pub fn stats_from_repository_configurations(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_configurations_as("refinements", request)
    }

    pub fn stats_from_repository_configurations_as(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "repository_configuration_list",
            alias,
            selection,
            false,
        ));
        self
    }

    pub fn group_by_repository_configurations_with_details(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_repository_configurations(request)
    }




    pub fn count_security_users(self) -> Self {
        self.count_security_users_as("count_security_users")
    }

    pub fn count_security_users_as(self, alias: impl Into<String>) -> Self {
        self.count_security_users_with(alias, crate::Q::security_users().unlimited())
    }

    pub fn count_security_users_with(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "security_user_list",
            alias,
            selection,
            true,
        ));
        self
    }

    pub fn stats_from_security_users(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_security_users_as("refinements", request)
    }

    pub fn stats_from_security_users_as(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "security_user_list",
            alias,
            selection,
            false,
        ));
        self
    }

    pub fn group_by_security_users_with_details(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_security_users(request)
    }




    pub fn count_security_roles(self) -> Self {
        self.count_security_roles_as("count_security_roles")
    }

    pub fn count_security_roles_as(self, alias: impl Into<String>) -> Self {
        self.count_security_roles_with(alias, crate::Q::security_roles().unlimited())
    }

    pub fn count_security_roles_with(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "security_role_list",
            alias,
            selection,
            true,
        ));
        self
    }

    pub fn stats_from_security_roles(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_security_roles_as("refinements", request)
    }

    pub fn stats_from_security_roles_as(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "security_role_list",
            alias,
            selection,
            false,
        ));
        self
    }

    pub fn group_by_security_roles_with_details(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_security_roles(request)
    }




    pub fn count_security_privileges(self) -> Self {
        self.count_security_privileges_as("count_security_privileges")
    }

    pub fn count_security_privileges_as(self, alias: impl Into<String>) -> Self {
        self.count_security_privileges_with(alias, crate::Q::security_privileges().unlimited())
    }

    pub fn count_security_privileges_with(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "security_privilege_list",
            alias,
            selection,
            true,
        ));
        self
    }

    pub fn stats_from_security_privileges(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_security_privileges_as("refinements", request)
    }

    pub fn stats_from_security_privileges_as(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "security_privilege_list",
            alias,
            selection,
            false,
        ));
        self
    }

    pub fn group_by_security_privileges_with_details(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_security_privileges(request)
    }



}

impl<R> Default for PlatformRequest<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> From< PlatformRequest<R> > for SelectQuery {
    fn from(request: PlatformRequest<R>) -> Self {
        QuerySelection::from(request).into_query()
    }
}

impl<R> From< PlatformRequest<R> > for QuerySelection {
    fn from(request: PlatformRequest<R>) -> Self {
        Self {
            query: request.query,
            relation_selections: request.relation_selections,
            relation_filters: request.relation_filters,
            child_enhancements: request.child_enhancements,
            query_options: request.query_options,
        }
    }
}


impl<'a, C> crate::request_support::AuditedSave<'a, C> for teaql_core::Audited<crate::Platform> 
where C: crate::request_support::TeaqlRepositoryProvider + ?Sized + 'a
{
    type Error = crate::TeaqlDataServiceError<C::PlatformRepository<'a>>;
    fn save(self, ctx: &'a C) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<teaql_runtime::GraphNode, Self::Error>> + '_>> {
        Box::pin(async move {
            teaql_runtime::save_audited_ledger_entity(self, ctx.user_context())
                .await
                .map_err(DataServiceError::Runtime)
        })
    }
}

impl<R: teaql_core::Entity> crate::PurposedQuery<PlatformRequest<R>> {
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.inner.query_options.comment = Some(comment.into());
        self
    }

    pub fn new_entity<C>(&self, ctx: &C) -> crate::Platform
    where
        C: crate::TeaqlRuntime + ?Sized,
    {
        let mut entity = crate::Platform::runtime_new(ctx.user_context().entity_root());
        if let Ok(id) = ctx.user_context().next_id(crate::Platform::ENTITY_NAME) {
            entity.update_id(id);
        }
        entity
    }

    fn into_inner_with_trace(mut self) -> PlatformRequest<R> {
        self.inner.query.trace_chain.push(teaql_core::TraceNode::new(
            self.inner.query.entity.clone(),
            None,
            self.purpose,
        ));
        self.inner
    }

    pub async fn execute_for_page<'a, C>(
        self,
        ctx: &'a C,
        offset: u64,
        limit: u64,
    ) -> Result<teaql_core::SmartList<R>, crate::request_support::TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_page(ctx, offset, limit).await
    }

    pub async fn execute_for_exists<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<bool, crate::request_support::TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_exists(ctx).await
    }

    pub async fn execute_for_list<'a, C>(self, ctx: &'a C) -> Result<teaql_core::SmartList<R>, crate::request_support::TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_list(ctx).await
    }

    /// Execute query as a lazy entity stream without materializing the result set.
    /// Set chunk size via .stream(chunk_size) or .stream_default() on the query.
    pub async fn execute_for_stream<'a, C>(self, ctx: &'a C) -> Result<crate::request_support::TeaqlEntityStream<'a, R, crate::request_support::TeaqlDataServiceError<C::PlatformRepository<'a>>>, crate::request_support::TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity + 'a,
    {
        self.into_inner_with_trace()._execute_for_stream(ctx).await
    }

    pub async fn execute_for_first<'a, C>(self, ctx: &'a C) -> Result<Option<R>, crate::request_support::TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_first(ctx).await
    }

    pub async fn execute_for_one<'a, C>(self, ctx: &'a C) -> Result<Option<R>, crate::request_support::TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_one(ctx).await
    }


    pub async fn execute_for_records<'a, C>(self, ctx: &'a C) -> Result<teaql_core::SmartList<teaql_core::Record>, crate::request_support::TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_records(ctx).await
    }

    pub async fn execute_for_record<'a, C>(self, ctx: &'a C) -> Result<Option<teaql_core::Record>, crate::request_support::TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_record(ctx).await
    }

    pub async fn execute_for_count<'a, C>(self, ctx: &'a C) -> Result<u64, crate::request_support::TeaqlDataServiceError<C::PlatformRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_count(ctx).await
    }
}
