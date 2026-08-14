use std::marker::PhantomData;

use serde_json::Value as JsonValue;
use teaql_core::{Aggregate, AggregateFunction, EntityDescriptor, Expr, Record, SelectQuery, SmartList};
use teaql_runtime::{DataServiceError, RuntimeError};

use crate::request_support::*;

impl EntityReference for crate::RepositoryConfiguration {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(&self)
    }
}

impl EntityReference for &crate::RepositoryConfiguration {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(self)
    }
}

// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/repository_configuration
#[derive(Debug)]
pub struct RepositoryConfigurationRequest<R = crate::RepositoryConfiguration> {
    query: SelectQuery,
    relation_selections: Vec<RelationSelection>,
    relation_filters: Vec<RelationFilter>,
    child_enhancements: Vec<QuerySelection>,
    query_options: QueryOptions,
    marker: PhantomData<R>,
}

impl<R> Clone for RepositoryConfigurationRequest<R> {
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

impl<R> RepositoryConfigurationRequest<R> {
    pub(crate) fn new() -> Self {
        Self {
            query: SelectQuery::new("RepositoryConfiguration")
                .project("id")
                .project("version"),
            relation_selections: Vec::new(),
            relation_filters: Vec::new(),
            child_enhancements: Vec::new(),
            query_options: QueryOptions::default(),
            marker: PhantomData,
        }
    }

    pub fn return_type<T>(self) -> RepositoryConfigurationRequest<T> {
        RepositoryConfigurationRequest {
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
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        let repository = ctx
            .repository_configuration_repository()
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
    ) -> Result<TeaqlEntityStream<'a, R, TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>, TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity + 'a,
    {
        Ok(Box::pin(async_stream::try_stream! {
            use futures_util::StreamExt;
            let repository = ctx
                .repository_configuration_repository()
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
    ) -> Result<Option<R>, TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
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
    ) -> Result<Option<R>, TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
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
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
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
    ) -> Result<u64, TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = ctx
            .repository_configuration_repository()
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
            .ok_or_else(|| DataServiceError::Runtime(RuntimeError::Graph(format!("count result for RepositoryConfiguration is missing or not numeric"))))
    }

    pub(crate) async fn _execute_for_exists<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<bool, TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = ctx
            .repository_configuration_repository()
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
    ) -> Result<SmartList<Record>, TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = ctx
            .repository_configuration_repository()
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
    ) -> Result<Option<Record>, TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
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
            "recipe_name" => Some("recipe_name"),
            "online" => Some("online"),
            "remote_url" => Some("remote_url"),
            "version" => Some("version"),
            "tenant" | "tenant_id" => Some("tenant_id"),
            "repository_type" | "repository_type_id" => Some("repository_type_id"),
            "repository_format" | "repository_format_id" => Some("repository_format_id"),
            "write_policy" | "write_policy_id" => Some("write_policy_id"),
            "blob_store" | "blob_store_id" => Some("blob_store_id"),
            _ => None,
        }
    }

    fn apply_dynamic_json_chain_filter(self, head: &str, tail: &str, value: &JsonValue) -> Self {
        let _ = (tail, value);
        match head {
            "tenant" => {
                self.with_tenant_matching(
                    crate::Q::tenants_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "repository_type" => {
                self.with_repository_type_matching(
                    crate::Q::repository_types_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "repository_format" => {
                self.with_repository_format_matching(
                    crate::Q::repository_formats_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "write_policy" => {
                self.with_write_policy_matching(
                    crate::Q::write_policies_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "blob_store" => {
                self.with_blob_store_matching(
                    crate::Q::blob_store_configurations_minimal()
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
        self.query = self.query.project("recipe_name");
        self.query = self.query.project("online");
        self.query = self.query.project("remote_url");
        self.query = self.query.project("version");
        self.query = self.query.project("tenant_id");
        self.query = self.query.project("repository_type_id");
        self.query = self.query.project("repository_format_id");
        self.query = self.query.project("write_policy_id");
        self.query = self.query.project("blob_store_id");
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
        request = request.select_repository_type();
        request = request.select_repository_format();
        request = request.select_write_policy();
        request = request.select_blob_store();
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


    pub fn select_recipe_name(mut self) -> Self {
        self.query = self.query.project("recipe_name");
        self
    }

    pub fn project_recipe_name(self) -> Self {
        self.select_recipe_name()
    }

    pub fn select_recipe_name_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_recipe_name_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_recipe_name_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("recipe_name", raw_sql_segment));
        self
    }

    pub fn group_by_recipe_name(self) -> Self {
        self.group_by("recipe_name")
    }

    pub fn group_by_recipe_name_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("recipe_name");
        request.query = request
            .query
            .project_expr(alias, Expr::column("recipe_name"));
        request
    }

    pub fn group_by_recipe_name_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("recipe_name")
            .aggregate_with_function("recipe_name", alias, function)
    }

    pub fn count_recipe_name(self) -> Self {
        self.count_recipe_name_as("recipe_name_count")
    }

    pub fn count_recipe_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("recipe_name", alias)
    }

    pub fn sum_recipe_name(self) -> Self {
        self.sum_recipe_name_as("sum_recipe_name")
    }

    pub fn sum_recipe_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("recipe_name", alias)
    }

    pub fn avg_recipe_name(self) -> Self {
        self.avg_recipe_name_as("avg_recipe_name")
    }

    pub fn avg_recipe_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("recipe_name", alias)
    }

    pub fn min_recipe_name(self) -> Self {
        self.min_recipe_name_as("min_recipe_name")
    }

    pub fn min_recipe_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("recipe_name", alias)
    }

    pub fn max_recipe_name(self) -> Self {
        self.max_recipe_name_as("max_recipe_name")
    }

    pub fn max_recipe_name_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("recipe_name", alias)
    }

    pub fn unselect_recipe_name(mut self) -> Self {
        self.query.projection.retain(|field| field != "recipe_name");
        self.query_options.raw_projections.retain(|projection| projection.property_name != "recipe_name");
        self
    }


    pub fn with_recipe_name(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "recipe_name",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_recipe_name_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "recipe_name",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_recipe_name_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("recipe_name", value));
        self
    }



    pub fn with_recipe_name_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("recipe_name", value));
        self
    }

    pub fn with_recipe_name_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("recipe_name", value));
        self
    }

    pub fn with_recipe_name_greater_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gte("recipe_name", value));
        self
    }

    pub fn with_recipe_name_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("recipe_name", value));
        self
    }

    pub fn with_recipe_name_less_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lte("recipe_name", value));
        self
    }

    pub fn with_recipe_name_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("recipe_name", lower, upper));
        self
    }

    pub fn with_recipe_name_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "recipe_name",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_recipe_name_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "recipe_name",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_recipe_name_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "recipe_name",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_recipe_name_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("recipe_name", value));
        self
    }

    pub fn with_recipe_name_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("recipe_name", value));
        self
    }

    pub fn with_recipe_name_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("recipe_name", value));
        self
    }

    pub fn with_recipe_name_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("recipe_name", value));
        self
    }

    pub fn with_recipe_name_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("recipe_name", value));
        self
    }

    pub fn with_recipe_name_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("recipe_name", value));
        self
    }

    pub fn with_recipe_name_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("recipe_name", value));
        self
    }
    pub fn with_recipe_name_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("recipe_name", value));
        self
    }

    pub fn with_recipe_name_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("recipe_name", value));
        self
    }

    pub fn with_recipe_name_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("recipe_name"));
        self
    }



    pub fn with_recipe_name_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("recipe_name"));
        self
    }


    pub fn order_by_recipe_name_asc(mut self) -> Self {
        self.query = self.query.order_asc("recipe_name");
        self
    }

    pub fn order_by_recipe_name_desc(mut self) -> Self {
        self.query = self.query.order_desc("recipe_name");
        self
    }

    pub fn order_by_recipe_name_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("recipe_name");
        self
    }

    pub fn order_by_recipe_name_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("recipe_name");
        self
    }


    pub fn select_online(mut self) -> Self {
        self.query = self.query.project("online");
        self
    }

    pub fn project_online(self) -> Self {
        self.select_online()
    }

    pub fn select_online_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_online_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_online_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("online", raw_sql_segment));
        self
    }

    pub fn group_by_online(self) -> Self {
        self.group_by("online")
    }

    pub fn group_by_online_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("online");
        request.query = request
            .query
            .project_expr(alias, Expr::column("online"));
        request
    }

    pub fn group_by_online_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("online")
            .aggregate_with_function("online", alias, function)
    }

    pub fn count_online(self) -> Self {
        self.count_online_as("online_count")
    }

    pub fn count_online_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("online", alias)
    }

    pub fn sum_online(self) -> Self {
        self.sum_online_as("sum_online")
    }

    pub fn sum_online_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("online", alias)
    }

    pub fn avg_online(self) -> Self {
        self.avg_online_as("avg_online")
    }

    pub fn avg_online_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("online", alias)
    }

    pub fn min_online(self) -> Self {
        self.min_online_as("min_online")
    }

    pub fn min_online_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("online", alias)
    }

    pub fn max_online(self) -> Self {
        self.max_online_as("max_online")
    }

    pub fn max_online_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("online", alias)
    }

    pub fn unselect_online(mut self) -> Self {
        self.query.projection.retain(|field| field != "online");
        self.query_options.raw_projections.retain(|projection| projection.property_name != "online");
        self
    }

    pub fn which_are_online(mut self) -> Self {
        self.query = self.query.and_filter(Expr::eq("online", true));
        self
    }

    pub fn which_are_not_online(mut self) -> Self {
        self.query = self.query.and_filter(Expr::eq("online", false));
        self
    }
    pub fn order_by_online_asc(mut self) -> Self {
        self.query = self.query.order_asc("online");
        self
    }

    pub fn order_by_online_desc(mut self) -> Self {
        self.query = self.query.order_desc("online");
        self
    }

    pub fn order_by_online_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("online");
        self
    }

    pub fn order_by_online_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("online");
        self
    }


    pub fn select_remote_url(mut self) -> Self {
        self.query = self.query.project("remote_url");
        self
    }

    pub fn project_remote_url(self) -> Self {
        self.select_remote_url()
    }

    pub fn select_remote_url_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_remote_url_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_remote_url_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("remote_url", raw_sql_segment));
        self
    }

    pub fn group_by_remote_url(self) -> Self {
        self.group_by("remote_url")
    }

    pub fn group_by_remote_url_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("remote_url");
        request.query = request
            .query
            .project_expr(alias, Expr::column("remote_url"));
        request
    }

    pub fn group_by_remote_url_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("remote_url")
            .aggregate_with_function("remote_url", alias, function)
    }

    pub fn count_remote_url(self) -> Self {
        self.count_remote_url_as("remote_url_count")
    }

    pub fn count_remote_url_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("remote_url", alias)
    }

    pub fn sum_remote_url(self) -> Self {
        self.sum_remote_url_as("sum_remote_url")
    }

    pub fn sum_remote_url_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("remote_url", alias)
    }

    pub fn avg_remote_url(self) -> Self {
        self.avg_remote_url_as("avg_remote_url")
    }

    pub fn avg_remote_url_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("remote_url", alias)
    }

    pub fn min_remote_url(self) -> Self {
        self.min_remote_url_as("min_remote_url")
    }

    pub fn min_remote_url_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("remote_url", alias)
    }

    pub fn max_remote_url(self) -> Self {
        self.max_remote_url_as("max_remote_url")
    }

    pub fn max_remote_url_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("remote_url", alias)
    }

    pub fn unselect_remote_url(mut self) -> Self {
        self.query.projection.retain(|field| field != "remote_url");
        self.query_options.raw_projections.retain(|projection| projection.property_name != "remote_url");
        self
    }


    pub fn with_remote_url(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "remote_url",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_remote_url_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "remote_url",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_remote_url_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("remote_url", value));
        self
    }



    pub fn with_remote_url_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("remote_url", value));
        self
    }

    pub fn with_remote_url_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("remote_url", value));
        self
    }

    pub fn with_remote_url_greater_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gte("remote_url", value));
        self
    }

    pub fn with_remote_url_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("remote_url", value));
        self
    }

    pub fn with_remote_url_less_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lte("remote_url", value));
        self
    }

    pub fn with_remote_url_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("remote_url", lower, upper));
        self
    }

    pub fn with_remote_url_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "remote_url",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_remote_url_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "remote_url",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_remote_url_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "remote_url",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_remote_url_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("remote_url", value));
        self
    }

    pub fn with_remote_url_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("remote_url", value));
        self
    }

    pub fn with_remote_url_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("remote_url", value));
        self
    }

    pub fn with_remote_url_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("remote_url", value));
        self
    }

    pub fn with_remote_url_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("remote_url", value));
        self
    }

    pub fn with_remote_url_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("remote_url", value));
        self
    }

    pub fn with_remote_url_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("remote_url", value));
        self
    }
    pub fn with_remote_url_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("remote_url", value));
        self
    }

    pub fn with_remote_url_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("remote_url", value));
        self
    }

    pub fn with_remote_url_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("remote_url"));
        self
    }



    pub fn with_remote_url_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("remote_url"));
        self
    }


    pub fn order_by_remote_url_asc(mut self) -> Self {
        self.query = self.query.order_asc("remote_url");
        self
    }

    pub fn order_by_remote_url_desc(mut self) -> Self {
        self.query = self.query.order_desc("remote_url");
        self
    }

    pub fn order_by_remote_url_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("remote_url");
        self
    }

    pub fn order_by_remote_url_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("remote_url");
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
        self.query = self.query.and_filter(Expr::eq("tenant_id", value.entity_id_value()));
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
        self.relation_filters.push(RelationFilter::new("tenant", selection));
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
        self.relation_filters.push(RelationFilter::new("tenant", selection));
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
        request.query = request
            .query
            .project_expr(alias, Expr::column("tenant_id"));
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
        self.query.relations.retain(|relation| relation.name != "tenant");
        self
    }


    /// Please use `with_repository_type_is` instead
    pub(crate) fn filter_by_repository_type(mut self, value: impl EntityReference) -> Self {
        self.query = self.query.and_filter(Expr::eq("repository_type_id", value.entity_id_value()));
        self
    }
    /// Complex relation filter for `repository_type`.
    ///
    /// **Usage Priority:**
    ///
    /// 1. **Preferred**: If you only want to filter by specific known constants, please **prefer** the generated semantic shortcut methods, such as:
    ///    - [`Self::with_repository_type_is_xxx`]
    ///
    ///    This gives the best code readability.
    ///
    /// 2. **Advanced**: Only use this method when you need to perform advanced searches, dynamic subqueries, or filter based on complex relation conditions.
    ///
    /// # Example
    /// ```text
    /// // Only use when building dynamic queries
    /// let dynamic_query = crate::Q::repository_types_minimal().filter(...);
    /// let request = crate::Q::repository_configurations().with_repository_type_matching(dynamic_query);
    /// ```
    pub fn with_repository_type_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "repository_type_id",
            <crate::RepositoryType as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters.push(RelationFilter::new("repository_type", selection));
        self
    }


    /// Complex relation filter for `repository_type`.
    ///
    /// **Usage Priority:**
    ///
    /// 1. **Preferred**: If you only want to filter by specific known constants, please **prefer** the generated semantic shortcut methods, such as:
    ///    - [`Self::with_repository_type_is_not_xxx`]
    ///
    ///    This gives the best code readability.
    ///
    /// 2. **Advanced**: Only use this method when you need to perform advanced searches, dynamic subqueries, or filter based on complex relation conditions.
    ///
    /// # Example
    /// ```text
    /// // Only use when building dynamic queries
    /// let dynamic_query = crate::Q::repository_types_minimal().filter(...);
    /// let request = crate::Q::repository_configurations().without_repository_type_matching(dynamic_query);
    /// ```
    pub fn without_repository_type_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "repository_type_id",
            <crate::RepositoryType as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters.push(RelationFilter::new("repository_type", selection));
        self
    }


    pub fn have_repository_type(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("repository_type_id"));
        self
    }

    pub fn have_no_repository_type(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("repository_type_id"));
        self
    }


    pub fn group_by_repository_type(self) -> Self {
        self.group_by("repository_type_id")
    }

    pub fn group_by_repository_type_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("repository_type_id");
        request.query = request
            .query
            .project_expr(alias, Expr::column("repository_type_id"));
        request
    }

    pub fn group_by_repository_type_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("repository_type_id")
            .aggregate_with_function("repository_type_id", alias, function)
    }

    pub fn group_by_repository_type_with(mut self, request: impl Into<QuerySelection>) -> Self {
        self.query = self.query.group_by("repository_type_id");
        self.query_options.object_group_bys.push(ObjectGroupBy::new(
            "repository_type",
            "repository_type_id",
            request,
        ));
        self
    }

    pub fn group_by_repository_type_with_details(self) -> Self {
        self.group_by_repository_type_with_details_from(crate::Q::repository_types().unlimited())
    }

    pub fn group_by_repository_type_with_details_from(self, request: impl Into<QuerySelection>) -> Self {
        self.group_by_repository_type_with(request)
    }


    pub fn roll_up_to_repository_type(self) -> Self {
        self.roll_up_to_repository_type_with(crate::Q::repository_types().unlimited())
    }

    pub fn roll_up_to_repository_type_with(self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.with_repository_type_matching(selection.clone())
            .group_by_repository_type_with(selection)
    }

    pub fn count_repository_type(self) -> Self {
        self.count_repository_type_as("repository_type_count")
    }

    pub fn count_repository_type_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("repository_type_id", alias)
    }

    pub fn unselect_repository_type(mut self) -> Self {
        self.query.projection.retain(|field| field != "repository_type_id");
        self.query.relations.retain(|relation| relation.name != "repository_type");
        self
    }


    /// Please use `with_repository_format_is` instead
    pub(crate) fn filter_by_repository_format(mut self, value: impl EntityReference) -> Self {
        self.query = self.query.and_filter(Expr::eq("repository_format_id", value.entity_id_value()));
        self
    }
    /// Complex relation filter for `repository_format`.
    ///
    /// **Usage Priority:**
    ///
    /// 1. **Preferred**: If you only want to filter by specific known constants, please **prefer** the generated semantic shortcut methods, such as:
    ///    - [`Self::with_repository_format_is_xxx`]
    ///
    ///    This gives the best code readability.
    ///
    /// 2. **Advanced**: Only use this method when you need to perform advanced searches, dynamic subqueries, or filter based on complex relation conditions.
    ///
    /// # Example
    /// ```text
    /// // Only use when building dynamic queries
    /// let dynamic_query = crate::Q::repository_formats_minimal().filter(...);
    /// let request = crate::Q::repository_configurations().with_repository_format_matching(dynamic_query);
    /// ```
    pub fn with_repository_format_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "repository_format_id",
            <crate::RepositoryFormat as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters.push(RelationFilter::new("repository_format", selection));
        self
    }


    /// Complex relation filter for `repository_format`.
    ///
    /// **Usage Priority:**
    ///
    /// 1. **Preferred**: If you only want to filter by specific known constants, please **prefer** the generated semantic shortcut methods, such as:
    ///    - [`Self::with_repository_format_is_not_xxx`]
    ///
    ///    This gives the best code readability.
    ///
    /// 2. **Advanced**: Only use this method when you need to perform advanced searches, dynamic subqueries, or filter based on complex relation conditions.
    ///
    /// # Example
    /// ```text
    /// // Only use when building dynamic queries
    /// let dynamic_query = crate::Q::repository_formats_minimal().filter(...);
    /// let request = crate::Q::repository_configurations().without_repository_format_matching(dynamic_query);
    /// ```
    pub fn without_repository_format_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "repository_format_id",
            <crate::RepositoryFormat as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters.push(RelationFilter::new("repository_format", selection));
        self
    }


    pub fn have_repository_format(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("repository_format_id"));
        self
    }

    pub fn have_no_repository_format(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("repository_format_id"));
        self
    }


    pub fn group_by_repository_format(self) -> Self {
        self.group_by("repository_format_id")
    }

    pub fn group_by_repository_format_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("repository_format_id");
        request.query = request
            .query
            .project_expr(alias, Expr::column("repository_format_id"));
        request
    }

    pub fn group_by_repository_format_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("repository_format_id")
            .aggregate_with_function("repository_format_id", alias, function)
    }

    pub fn group_by_repository_format_with(mut self, request: impl Into<QuerySelection>) -> Self {
        self.query = self.query.group_by("repository_format_id");
        self.query_options.object_group_bys.push(ObjectGroupBy::new(
            "repository_format",
            "repository_format_id",
            request,
        ));
        self
    }

    pub fn group_by_repository_format_with_details(self) -> Self {
        self.group_by_repository_format_with_details_from(crate::Q::repository_formats().unlimited())
    }

    pub fn group_by_repository_format_with_details_from(self, request: impl Into<QuerySelection>) -> Self {
        self.group_by_repository_format_with(request)
    }


    pub fn roll_up_to_repository_format(self) -> Self {
        self.roll_up_to_repository_format_with(crate::Q::repository_formats().unlimited())
    }

    pub fn roll_up_to_repository_format_with(self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.with_repository_format_matching(selection.clone())
            .group_by_repository_format_with(selection)
    }

    pub fn count_repository_format(self) -> Self {
        self.count_repository_format_as("repository_format_count")
    }

    pub fn count_repository_format_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("repository_format_id", alias)
    }

    pub fn unselect_repository_format(mut self) -> Self {
        self.query.projection.retain(|field| field != "repository_format_id");
        self.query.relations.retain(|relation| relation.name != "repository_format");
        self
    }


    /// Please use `with_write_policy_is` instead
    pub(crate) fn filter_by_write_policy(mut self, value: impl EntityReference) -> Self {
        self.query = self.query.and_filter(Expr::eq("write_policy_id", value.entity_id_value()));
        self
    }
    /// Complex relation filter for `write_policy`.
    ///
    /// **Usage Priority:**
    ///
    /// 1. **Preferred**: If you only want to filter by specific known constants, please **prefer** the generated semantic shortcut methods, such as:
    ///    - [`Self::with_write_policy_is_xxx`]
    ///
    ///    This gives the best code readability.
    ///
    /// 2. **Advanced**: Only use this method when you need to perform advanced searches, dynamic subqueries, or filter based on complex relation conditions.
    ///
    /// # Example
    /// ```text
    /// // Only use when building dynamic queries
    /// let dynamic_query = crate::Q::write_policies_minimal().filter(...);
    /// let request = crate::Q::repository_configurations().with_write_policy_matching(dynamic_query);
    /// ```
    pub fn with_write_policy_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "write_policy_id",
            <crate::WritePolicy as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters.push(RelationFilter::new("write_policy", selection));
        self
    }


    /// Complex relation filter for `write_policy`.
    ///
    /// **Usage Priority:**
    ///
    /// 1. **Preferred**: If you only want to filter by specific known constants, please **prefer** the generated semantic shortcut methods, such as:
    ///    - [`Self::with_write_policy_is_not_xxx`]
    ///
    ///    This gives the best code readability.
    ///
    /// 2. **Advanced**: Only use this method when you need to perform advanced searches, dynamic subqueries, or filter based on complex relation conditions.
    ///
    /// # Example
    /// ```text
    /// // Only use when building dynamic queries
    /// let dynamic_query = crate::Q::write_policies_minimal().filter(...);
    /// let request = crate::Q::repository_configurations().without_write_policy_matching(dynamic_query);
    /// ```
    pub fn without_write_policy_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "write_policy_id",
            <crate::WritePolicy as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters.push(RelationFilter::new("write_policy", selection));
        self
    }


    pub fn have_write_policy(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("write_policy_id"));
        self
    }

    pub fn have_no_write_policy(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("write_policy_id"));
        self
    }


    pub fn group_by_write_policy(self) -> Self {
        self.group_by("write_policy_id")
    }

    pub fn group_by_write_policy_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("write_policy_id");
        request.query = request
            .query
            .project_expr(alias, Expr::column("write_policy_id"));
        request
    }

    pub fn group_by_write_policy_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("write_policy_id")
            .aggregate_with_function("write_policy_id", alias, function)
    }

    pub fn group_by_write_policy_with(mut self, request: impl Into<QuerySelection>) -> Self {
        self.query = self.query.group_by("write_policy_id");
        self.query_options.object_group_bys.push(ObjectGroupBy::new(
            "write_policy",
            "write_policy_id",
            request,
        ));
        self
    }

    pub fn group_by_write_policy_with_details(self) -> Self {
        self.group_by_write_policy_with_details_from(crate::Q::write_policies().unlimited())
    }

    pub fn group_by_write_policy_with_details_from(self, request: impl Into<QuerySelection>) -> Self {
        self.group_by_write_policy_with(request)
    }


    pub fn roll_up_to_write_policy(self) -> Self {
        self.roll_up_to_write_policy_with(crate::Q::write_policies().unlimited())
    }

    pub fn roll_up_to_write_policy_with(self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.with_write_policy_matching(selection.clone())
            .group_by_write_policy_with(selection)
    }

    pub fn count_write_policy(self) -> Self {
        self.count_write_policy_as("write_policy_count")
    }

    pub fn count_write_policy_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("write_policy_id", alias)
    }

    pub fn unselect_write_policy(mut self) -> Self {
        self.query.projection.retain(|field| field != "write_policy_id");
        self.query.relations.retain(|relation| relation.name != "write_policy");
        self
    }


    pub fn filter_by_blob_store(mut self, value: impl EntityReference) -> Self {
        self.query = self.query.and_filter(Expr::eq("blob_store_id", value.entity_id_value()));
        self
    }

    pub fn with_blob_store_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "blob_store_id",
            <crate::BlobStoreConfiguration as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters.push(RelationFilter::new("blob_store", selection));
        self
    }


    pub fn without_blob_store_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "blob_store_id",
            <crate::BlobStoreConfiguration as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "id",
        ));
        self.relation_filters.push(RelationFilter::new("blob_store", selection));
        self
    }


    pub fn have_blob_store(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("blob_store_id"));
        self
    }

    pub fn have_no_blob_store(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("blob_store_id"));
        self
    }


    pub fn group_by_blob_store(self) -> Self {
        self.group_by("blob_store_id")
    }

    pub fn group_by_blob_store_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("blob_store_id");
        request.query = request
            .query
            .project_expr(alias, Expr::column("blob_store_id"));
        request
    }

    pub fn group_by_blob_store_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("blob_store_id")
            .aggregate_with_function("blob_store_id", alias, function)
    }

    pub fn group_by_blob_store_with(mut self, request: impl Into<QuerySelection>) -> Self {
        self.query = self.query.group_by("blob_store_id");
        self.query_options.object_group_bys.push(ObjectGroupBy::new(
            "blob_store",
            "blob_store_id",
            request,
        ));
        self
    }

    pub fn group_by_blob_store_with_details(self) -> Self {
        self.group_by_blob_store_with_details_from(crate::Q::blob_store_configurations().unlimited())
    }

    pub fn group_by_blob_store_with_details_from(self, request: impl Into<QuerySelection>) -> Self {
        self.group_by_blob_store_with(request)
    }


    pub fn roll_up_to_blob_store(self) -> Self {
        self.roll_up_to_blob_store_with(crate::Q::blob_store_configurations().unlimited())
    }

    pub fn roll_up_to_blob_store_with(self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.with_blob_store_matching(selection.clone())
            .group_by_blob_store_with(selection)
    }

    pub fn count_blob_store(self) -> Self {
        self.count_blob_store_as("blob_store_count")
    }

    pub fn count_blob_store_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("blob_store_id", alias)
    }

    pub fn unselect_blob_store(mut self) -> Self {
        self.query.projection.retain(|field| field != "blob_store_id");
        self.query.relations.retain(|relation| relation.name != "blob_store");
        self
    }
    pub fn repository_type_is_hosted(self) -> Self {
        self.filter_by_repository_type(1001_u64)
    }

    pub fn with_repository_type_is_hosted(self) -> Self {
        self.filter_by_repository_type(1001_u64)
    }



    pub fn with_repository_type_is_not_hosted(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_type_id", 1001_u64));
        self
    }


    pub fn repository_type_is_proxy(self) -> Self {
        self.filter_by_repository_type(1002_u64)
    }

    pub fn with_repository_type_is_proxy(self) -> Self {
        self.filter_by_repository_type(1002_u64)
    }



    pub fn with_repository_type_is_not_proxy(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_type_id", 1002_u64));
        self
    }


    pub fn repository_type_is_group(self) -> Self {
        self.filter_by_repository_type(1003_u64)
    }

    pub fn with_repository_type_is_group(self) -> Self {
        self.filter_by_repository_type(1003_u64)
    }



    pub fn with_repository_type_is_not_group(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_type_id", 1003_u64));
        self
    }



    pub fn repository_format_is_maven2(self) -> Self {
        self.filter_by_repository_format(1001_u64)
    }

    pub fn with_repository_format_is_maven2(self) -> Self {
        self.filter_by_repository_format(1001_u64)
    }



    pub fn with_repository_format_is_not_maven2(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_format_id", 1001_u64));
        self
    }


    pub fn repository_format_is_raw(self) -> Self {
        self.filter_by_repository_format(1002_u64)
    }

    pub fn with_repository_format_is_raw(self) -> Self {
        self.filter_by_repository_format(1002_u64)
    }



    pub fn with_repository_format_is_not_raw(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_format_id", 1002_u64));
        self
    }


    pub fn repository_format_is_docker(self) -> Self {
        self.filter_by_repository_format(1003_u64)
    }

    pub fn with_repository_format_is_docker(self) -> Self {
        self.filter_by_repository_format(1003_u64)
    }



    pub fn with_repository_format_is_not_docker(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_format_id", 1003_u64));
        self
    }


    pub fn repository_format_is_npm(self) -> Self {
        self.filter_by_repository_format(1004_u64)
    }

    pub fn with_repository_format_is_npm(self) -> Self {
        self.filter_by_repository_format(1004_u64)
    }



    pub fn with_repository_format_is_not_npm(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_format_id", 1004_u64));
        self
    }


    pub fn repository_format_is_pypi(self) -> Self {
        self.filter_by_repository_format(1005_u64)
    }

    pub fn with_repository_format_is_pypi(self) -> Self {
        self.filter_by_repository_format(1005_u64)
    }



    pub fn with_repository_format_is_not_pypi(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_format_id", 1005_u64));
        self
    }


    pub fn repository_format_is_gomod(self) -> Self {
        self.filter_by_repository_format(1006_u64)
    }

    pub fn with_repository_format_is_gomod(self) -> Self {
        self.filter_by_repository_format(1006_u64)
    }



    pub fn with_repository_format_is_not_gomod(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_format_id", 1006_u64));
        self
    }


    pub fn repository_format_is_cargo(self) -> Self {
        self.filter_by_repository_format(1007_u64)
    }

    pub fn with_repository_format_is_cargo(self) -> Self {
        self.filter_by_repository_format(1007_u64)
    }



    pub fn with_repository_format_is_not_cargo(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_format_id", 1007_u64));
        self
    }


    pub fn repository_format_is_nuget(self) -> Self {
        self.filter_by_repository_format(1008_u64)
    }

    pub fn with_repository_format_is_nuget(self) -> Self {
        self.filter_by_repository_format(1008_u64)
    }



    pub fn with_repository_format_is_not_nuget(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("repository_format_id", 1008_u64));
        self
    }



    pub fn write_policy_is_allow_write(self) -> Self {
        self.filter_by_write_policy(1001_u64)
    }

    pub fn with_write_policy_is_allow_write(self) -> Self {
        self.filter_by_write_policy(1001_u64)
    }



    pub fn with_write_policy_is_not_allow_write(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("write_policy_id", 1001_u64));
        self
    }


    pub fn write_policy_is_allow_once(self) -> Self {
        self.filter_by_write_policy(1002_u64)
    }

    pub fn with_write_policy_is_allow_once(self) -> Self {
        self.filter_by_write_policy(1002_u64)
    }



    pub fn with_write_policy_is_not_allow_once(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("write_policy_id", 1002_u64));
        self
    }


    pub fn write_policy_is_read_only(self) -> Self {
        self.filter_by_write_policy(1003_u64)
    }

    pub fn with_write_policy_is_read_only(self) -> Self {
        self.filter_by_write_policy(1003_u64)
    }



    pub fn with_write_policy_is_not_read_only(mut self) -> Self {
        self.query = self.query.and_filter(Expr::ne("write_policy_id", 1003_u64));
        self
    }




    pub fn select_tenant(mut self) -> Self {
        self.query = self.query.relation("tenant");
        self
    }

    pub fn select_tenant_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("tenant", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("tenant", selection));
        self
}

    pub fn facet_by_tenant_as(self, facet_name: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
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

    pub fn select_repository_type(mut self) -> Self {
        self.query = self.query.relation("repository_type");
        self
    }

    pub fn select_repository_type_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("repository_type", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("repository_type", selection));
        self
}

    pub fn facet_by_repository_type_as(self, facet_name: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.facet_by_repository_type_as_with_options(facet_name, request, true)
    }

    pub fn facet_by_repository_type_as_with_options(
        mut self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
        include_all_facets: bool,
    ) -> Self {
        self.query_options.facets.push(FacetRequest::new(
            facet_name,
            "repository_type",
            request,
            include_all_facets,
        ));
        self
    }

    pub fn select_repository_format(mut self) -> Self {
        self.query = self.query.relation("repository_format");
        self
    }

    pub fn select_repository_format_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("repository_format", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("repository_format", selection));
        self
}

    pub fn facet_by_repository_format_as(self, facet_name: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.facet_by_repository_format_as_with_options(facet_name, request, true)
    }

    pub fn facet_by_repository_format_as_with_options(
        mut self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
        include_all_facets: bool,
    ) -> Self {
        self.query_options.facets.push(FacetRequest::new(
            facet_name,
            "repository_format",
            request,
            include_all_facets,
        ));
        self
    }

    pub fn select_write_policy(mut self) -> Self {
        self.query = self.query.relation("write_policy");
        self
    }

    pub fn select_write_policy_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("write_policy", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("write_policy", selection));
        self
}

    pub fn facet_by_write_policy_as(self, facet_name: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.facet_by_write_policy_as_with_options(facet_name, request, true)
    }

    pub fn facet_by_write_policy_as_with_options(
        mut self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
        include_all_facets: bool,
    ) -> Self {
        self.query_options.facets.push(FacetRequest::new(
            facet_name,
            "write_policy",
            request,
            include_all_facets,
        ));
        self
    }

    pub fn select_blob_store(mut self) -> Self {
        self.query = self.query.relation("blob_store");
        self
    }

    pub fn select_blob_store_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("blob_store", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("blob_store", selection));
        self
}

    pub fn facet_by_blob_store_as(self, facet_name: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        self.facet_by_blob_store_as_with_options(facet_name, request, true)
    }

    pub fn facet_by_blob_store_as_with_options(
        mut self,
        facet_name: impl Into<String>,
        request: impl Into<QuerySelection>,
        include_all_facets: bool,
    ) -> Self {
        self.query_options.facets.push(FacetRequest::new(
            facet_name,
            "blob_store",
            request,
            include_all_facets,
        ));
        self
    }
}

impl<R> Default for RepositoryConfigurationRequest<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> From< RepositoryConfigurationRequest<R> > for SelectQuery {
    fn from(request: RepositoryConfigurationRequest<R>) -> Self {
        QuerySelection::from(request).into_query()
    }
}

impl<R> From< RepositoryConfigurationRequest<R> > for QuerySelection {
    fn from(request: RepositoryConfigurationRequest<R>) -> Self {
        Self {
            query: request.query,
            relation_selections: request.relation_selections,
            relation_filters: request.relation_filters,
            child_enhancements: request.child_enhancements,
            query_options: request.query_options,
        }
    }
}


impl<'a, C> crate::request_support::AuditedSave<'a, C> for teaql_core::Audited<crate::RepositoryConfiguration> 
where C: crate::request_support::TeaqlRepositoryProvider + ?Sized + 'a
{
    type Error = crate::TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>;
    fn save(self, ctx: &'a C) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<teaql_runtime::GraphNode, Self::Error>> + '_>> {
        Box::pin(async move {
            teaql_runtime::save_audited_ledger_entity(self, ctx.user_context())
                .await
                .map_err(DataServiceError::Runtime)
        })
    }
}

impl<R: teaql_core::Entity> crate::PurposedQuery<RepositoryConfigurationRequest<R>> {
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.inner.query_options.comment = Some(comment.into());
        self
    }

    pub fn new_entity<C>(&self, ctx: &C) -> crate::RepositoryConfiguration
    where
        C: crate::TeaqlRuntime + ?Sized,
    {
        let mut entity = crate::RepositoryConfiguration::runtime_new(ctx.user_context().entity_root());
        if let Ok(id) = ctx.user_context().next_id(crate::RepositoryConfiguration::ENTITY_NAME) {
            entity.update_id(id);
        }
        entity
    }

    fn into_inner_with_trace(mut self) -> RepositoryConfigurationRequest<R> {
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
    ) -> Result<teaql_core::SmartList<R>, crate::request_support::TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_page(ctx, offset, limit).await
    }

    pub async fn execute_for_exists<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<bool, crate::request_support::TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_exists(ctx).await
    }

    pub async fn execute_for_list<'a, C>(self, ctx: &'a C) -> Result<teaql_core::SmartList<R>, crate::request_support::TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_list(ctx).await
    }

    /// Execute query as a lazy entity stream without materializing the result set.
    /// Set chunk size via .stream(chunk_size) or .stream_default() on the query.
    pub async fn execute_for_stream<'a, C>(self, ctx: &'a C) -> Result<crate::request_support::TeaqlEntityStream<'a, R, crate::request_support::TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>, crate::request_support::TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity + 'a,
    {
        self.into_inner_with_trace()._execute_for_stream(ctx).await
    }

    pub async fn execute_for_first<'a, C>(self, ctx: &'a C) -> Result<Option<R>, crate::request_support::TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_first(ctx).await
    }

    pub async fn execute_for_one<'a, C>(self, ctx: &'a C) -> Result<Option<R>, crate::request_support::TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_one(ctx).await
    }


    pub async fn execute_for_records<'a, C>(self, ctx: &'a C) -> Result<teaql_core::SmartList<teaql_core::Record>, crate::request_support::TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_records(ctx).await
    }

    pub async fn execute_for_record<'a, C>(self, ctx: &'a C) -> Result<Option<teaql_core::Record>, crate::request_support::TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_record(ctx).await
    }

    pub async fn execute_for_count<'a, C>(self, ctx: &'a C) -> Result<u64, crate::request_support::TeaqlDataServiceError<C::RepositoryConfigurationRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_count(ctx).await
    }
}
