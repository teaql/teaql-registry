use std::marker::PhantomData;

use serde_json::Value as JsonValue;
use teaql_core::{Aggregate, AggregateFunction, EntityDescriptor, Expr, Record, SelectQuery, SmartList};
use teaql_runtime::{DataServiceError, RuntimeError};

use crate::request_support::*;

impl EntityReference for crate::AssetBlob {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(&self)
    }
}

impl EntityReference for &crate::AssetBlob {
    fn entity_id_value(self) -> teaql_core::Value {
        teaql_core::IdentifiableEntity::id_value(self)
    }
}

// ⛔ AI agents: DO NOT read this file for API discovery. Instead run: cargo teaql --input modeling/MODEL.xml rust-assist-query/asset_blob
#[derive(Debug)]
pub struct AssetBlobRequest<R = crate::AssetBlob> {
    query: SelectQuery,
    relation_selections: Vec<RelationSelection>,
    relation_filters: Vec<RelationFilter>,
    child_enhancements: Vec<QuerySelection>,
    query_options: QueryOptions,
    marker: PhantomData<R>,
}

impl<R> Clone for AssetBlobRequest<R> {
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

impl<R> AssetBlobRequest<R> {
    pub(crate) fn new() -> Self {
        Self {
            query: SelectQuery::new("AssetBlob")
                .project("id")
                .project("version"),
            relation_selections: Vec::new(),
            relation_filters: Vec::new(),
            child_enhancements: Vec::new(),
            query_options: QueryOptions::default(),
            marker: PhantomData,
        }
    }

    pub fn return_type<T>(self) -> AssetBlobRequest<T> {
        AssetBlobRequest {
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
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity,
    {
        let repository = ctx
            .asset_blob_repository()
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
    ) -> Result<TeaqlEntityStream<'a, R, TeaqlDataServiceError<C::AssetBlobRepository<'a>>>, TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity + 'a,
    {
        Ok(Box::pin(async_stream::try_stream! {
            use futures_util::StreamExt;
            let repository = ctx
                .asset_blob_repository()
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
    ) -> Result<Option<R>, TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
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
    ) -> Result<Option<R>, TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
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
    ) -> Result<SmartList<R>, TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
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
    ) -> Result<u64, TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = ctx
            .asset_blob_repository()
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
            .ok_or_else(|| DataServiceError::Runtime(RuntimeError::Graph(format!("count result for AssetBlob is missing or not numeric"))))
    }

    pub(crate) async fn _execute_for_exists<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<bool, TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = ctx
            .asset_blob_repository()
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
    ) -> Result<SmartList<Record>, TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: TeaqlRepositoryProvider + ?Sized,
    {
        let repository = ctx
            .asset_blob_repository()
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
    ) -> Result<Option<Record>, TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
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
            "blob_ref" => Some("blob_ref"),
            "blob_size" => Some("blob_size"),
            "content_type" => Some("content_type"),
            "sha1_checksum" => Some("sha1_checksum"),
            "sha256_checksum" => Some("sha256_checksum"),
            "md5_checksum" => Some("md5_checksum"),
            "version" => Some("version"),
            "blob_store" | "blob_store_id" => Some("blob_store_id"),
            _ => None,
        }
    }

    fn apply_dynamic_json_chain_filter(self, head: &str, tail: &str, value: &JsonValue) -> Self {
        let _ = (tail, value);
        match head {
            "blob_store" => {
                self.with_blob_store_matching(
                    crate::Q::blob_store_configurations_minimal()
                        .apply_dynamic_json_filter(tail, value),
                )
            }
            "asset_list" => {
                self.with_asset_list_matching(
                    crate::Q::assets_minimal()
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
        self.query = self.query.project("blob_ref");
        self.query = self.query.project("blob_size");
        self.query = self.query.project("content_type");
        self.query = self.query.project("sha1_checksum");
        self.query = self.query.project("sha256_checksum");
        self.query = self.query.project("md5_checksum");
        self.query = self.query.project("version");
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
        request = request.select_blob_store();
        request
    }

    pub fn select_children(self) -> Self {
        let mut request = self.select_all();
        request = request.select_asset_list();
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


    pub fn select_blob_ref(mut self) -> Self {
        self.query = self.query.project("blob_ref");
        self
    }

    pub fn project_blob_ref(self) -> Self {
        self.select_blob_ref()
    }

    pub fn select_blob_ref_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_blob_ref_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_blob_ref_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("blob_ref", raw_sql_segment));
        self
    }

    pub fn group_by_blob_ref(self) -> Self {
        self.group_by("blob_ref")
    }

    pub fn group_by_blob_ref_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("blob_ref");
        request.query = request
            .query
            .project_expr(alias, Expr::column("blob_ref"));
        request
    }

    pub fn group_by_blob_ref_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("blob_ref")
            .aggregate_with_function("blob_ref", alias, function)
    }

    pub fn count_blob_ref(self) -> Self {
        self.count_blob_ref_as("blob_ref_count")
    }

    pub fn count_blob_ref_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("blob_ref", alias)
    }

    pub fn sum_blob_ref(self) -> Self {
        self.sum_blob_ref_as("sum_blob_ref")
    }

    pub fn sum_blob_ref_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("blob_ref", alias)
    }

    pub fn avg_blob_ref(self) -> Self {
        self.avg_blob_ref_as("avg_blob_ref")
    }

    pub fn avg_blob_ref_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("blob_ref", alias)
    }

    pub fn min_blob_ref(self) -> Self {
        self.min_blob_ref_as("min_blob_ref")
    }

    pub fn min_blob_ref_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("blob_ref", alias)
    }

    pub fn max_blob_ref(self) -> Self {
        self.max_blob_ref_as("max_blob_ref")
    }

    pub fn max_blob_ref_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("blob_ref", alias)
    }

    pub fn unselect_blob_ref(mut self) -> Self {
        self.query.projection.retain(|field| field != "blob_ref");
        self.query_options.raw_projections.retain(|projection| projection.property_name != "blob_ref");
        self
    }


    pub fn with_blob_ref(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "blob_ref",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_blob_ref_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "blob_ref",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_blob_ref_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("blob_ref", value));
        self
    }



    pub fn with_blob_ref_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("blob_ref", value));
        self
    }

    pub fn with_blob_ref_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("blob_ref", value));
        self
    }

    pub fn with_blob_ref_greater_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gte("blob_ref", value));
        self
    }

    pub fn with_blob_ref_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("blob_ref", value));
        self
    }

    pub fn with_blob_ref_less_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lte("blob_ref", value));
        self
    }

    pub fn with_blob_ref_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("blob_ref", lower, upper));
        self
    }

    pub fn with_blob_ref_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "blob_ref",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_blob_ref_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "blob_ref",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_blob_ref_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "blob_ref",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_blob_ref_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("blob_ref", value));
        self
    }

    pub fn with_blob_ref_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("blob_ref", value));
        self
    }

    pub fn with_blob_ref_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("blob_ref", value));
        self
    }

    pub fn with_blob_ref_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("blob_ref", value));
        self
    }

    pub fn with_blob_ref_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("blob_ref", value));
        self
    }

    pub fn with_blob_ref_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("blob_ref", value));
        self
    }

    pub fn with_blob_ref_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("blob_ref", value));
        self
    }
    pub fn with_blob_ref_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("blob_ref", value));
        self
    }

    pub fn with_blob_ref_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("blob_ref", value));
        self
    }

    pub fn with_blob_ref_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("blob_ref"));
        self
    }



    pub fn with_blob_ref_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("blob_ref"));
        self
    }


    pub fn order_by_blob_ref_asc(mut self) -> Self {
        self.query = self.query.order_asc("blob_ref");
        self
    }

    pub fn order_by_blob_ref_desc(mut self) -> Self {
        self.query = self.query.order_desc("blob_ref");
        self
    }

    pub fn order_by_blob_ref_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("blob_ref");
        self
    }

    pub fn order_by_blob_ref_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("blob_ref");
        self
    }


    pub fn select_blob_size(mut self) -> Self {
        self.query = self.query.project("blob_size");
        self
    }

    pub fn project_blob_size(self) -> Self {
        self.select_blob_size()
    }

    pub fn select_blob_size_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_blob_size_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_blob_size_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("blob_size", raw_sql_segment));
        self
    }

    pub fn select_blob_size_with_function(self, function: AggregateFunction) -> Self {
        self.select_blob_size_as_with_function("blob_size", function)
    }

    pub fn select_blob_size_as_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.aggregate_with_function("blob_size", alias, function)
    }

    pub fn group_by_blob_size(self) -> Self {
        self.group_by("blob_size")
    }

    pub fn group_by_blob_size_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("blob_size");
        request.query = request
            .query
            .project_expr(alias, Expr::column("blob_size"));
        request
    }

    pub fn group_by_blob_size_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("blob_size")
            .aggregate_with_function("blob_size", alias, function)
    }

    pub fn count_blob_size(self) -> Self {
        self.count_blob_size_as("blob_size_count")
    }

    pub fn count_blob_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("blob_size", alias)
    }

    pub fn sum_blob_size(self) -> Self {
        self.sum_blob_size_as("sum_blob_size")
    }

    pub fn sum_blob_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("blob_size", alias)
    }

    pub fn avg_blob_size(self) -> Self {
        self.avg_blob_size_as("avg_blob_size")
    }

    pub fn avg_blob_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("blob_size", alias)
    }

    pub fn min_blob_size(self) -> Self {
        self.min_blob_size_as("min_blob_size")
    }

    pub fn min_blob_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("blob_size", alias)
    }

    pub fn max_blob_size(self) -> Self {
        self.max_blob_size_as("max_blob_size")
    }

    pub fn max_blob_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("blob_size", alias)
    }

    pub fn standard_deviation_blob_size(self) -> Self {
        self.standard_deviation_blob_size_as("stdDev_blob_size")
    }

    pub fn standard_deviation_blob_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_stddev("blob_size", alias)
    }

    pub fn square_root_of_population_standard_deviation_blob_size(self) -> Self {
        self.square_root_of_population_standard_deviation_blob_size_as("stdDevPop_blob_size")
    }

    pub fn square_root_of_population_standard_deviation_blob_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_stddev_pop("blob_size", alias)
    }

    pub fn sample_variance_blob_size(self) -> Self {
        self.sample_variance_blob_size_as("varSamp_blob_size")
    }

    pub fn sample_variance_blob_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_var_samp("blob_size", alias)
    }

    pub fn sample_population_variance_blob_size(self) -> Self {
        self.sample_population_variance_blob_size_as("varPop_blob_size")
    }

    pub fn sample_population_variance_blob_size_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_var_pop("blob_size", alias)
    }

    pub fn unselect_blob_size(mut self) -> Self {
        self.query.projection.retain(|field| field != "blob_size");
        self.query_options.raw_projections.retain(|projection| projection.property_name != "blob_size");
        self
    }


    pub fn with_blob_size(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "blob_size",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_blob_size_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "blob_size",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_blob_size_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("blob_size", value));
        self
    }



    pub fn with_blob_size_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("blob_size", value));
        self
    }

    pub fn with_blob_size_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("blob_size", value));
        self
    }

    pub fn with_blob_size_greater_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gte("blob_size", value));
        self
    }

    pub fn with_blob_size_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("blob_size", value));
        self
    }

    pub fn with_blob_size_less_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lte("blob_size", value));
        self
    }

    pub fn with_blob_size_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("blob_size", lower, upper));
        self
    }

    pub fn with_blob_size_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "blob_size",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_blob_size_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "blob_size",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_blob_size_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "blob_size",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_blob_size_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("blob_size", value));
        self
    }

    pub fn with_blob_size_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("blob_size", value));
        self
    }

    pub fn with_blob_size_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("blob_size"));
        self
    }



    pub fn with_blob_size_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("blob_size"));
        self
    }


    pub fn order_by_blob_size_asc(mut self) -> Self {
        self.query = self.query.order_asc("blob_size");
        self
    }

    pub fn order_by_blob_size_desc(mut self) -> Self {
        self.query = self.query.order_desc("blob_size");
        self
    }

    pub fn order_by_blob_size_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("blob_size");
        self
    }

    pub fn order_by_blob_size_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("blob_size");
        self
    }


    pub fn select_content_type(mut self) -> Self {
        self.query = self.query.project("content_type");
        self
    }

    pub fn project_content_type(self) -> Self {
        self.select_content_type()
    }

    pub fn select_content_type_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_content_type_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_content_type_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("content_type", raw_sql_segment));
        self
    }

    pub fn group_by_content_type(self) -> Self {
        self.group_by("content_type")
    }

    pub fn group_by_content_type_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("content_type");
        request.query = request
            .query
            .project_expr(alias, Expr::column("content_type"));
        request
    }

    pub fn group_by_content_type_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("content_type")
            .aggregate_with_function("content_type", alias, function)
    }

    pub fn count_content_type(self) -> Self {
        self.count_content_type_as("content_type_count")
    }

    pub fn count_content_type_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("content_type", alias)
    }

    pub fn sum_content_type(self) -> Self {
        self.sum_content_type_as("sum_content_type")
    }

    pub fn sum_content_type_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("content_type", alias)
    }

    pub fn avg_content_type(self) -> Self {
        self.avg_content_type_as("avg_content_type")
    }

    pub fn avg_content_type_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("content_type", alias)
    }

    pub fn min_content_type(self) -> Self {
        self.min_content_type_as("min_content_type")
    }

    pub fn min_content_type_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("content_type", alias)
    }

    pub fn max_content_type(self) -> Self {
        self.max_content_type_as("max_content_type")
    }

    pub fn max_content_type_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("content_type", alias)
    }

    pub fn unselect_content_type(mut self) -> Self {
        self.query.projection.retain(|field| field != "content_type");
        self.query_options.raw_projections.retain(|projection| projection.property_name != "content_type");
        self
    }


    pub fn with_content_type(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "content_type",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_content_type_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "content_type",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_content_type_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("content_type", value));
        self
    }



    pub fn with_content_type_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("content_type", value));
        self
    }

    pub fn with_content_type_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("content_type", value));
        self
    }

    pub fn with_content_type_greater_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gte("content_type", value));
        self
    }

    pub fn with_content_type_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("content_type", value));
        self
    }

    pub fn with_content_type_less_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lte("content_type", value));
        self
    }

    pub fn with_content_type_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("content_type", lower, upper));
        self
    }

    pub fn with_content_type_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "content_type",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_content_type_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "content_type",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_content_type_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "content_type",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_content_type_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("content_type", value));
        self
    }

    pub fn with_content_type_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("content_type", value));
        self
    }

    pub fn with_content_type_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("content_type", value));
        self
    }

    pub fn with_content_type_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("content_type", value));
        self
    }

    pub fn with_content_type_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("content_type", value));
        self
    }

    pub fn with_content_type_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("content_type", value));
        self
    }

    pub fn with_content_type_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("content_type", value));
        self
    }
    pub fn with_content_type_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("content_type", value));
        self
    }

    pub fn with_content_type_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("content_type", value));
        self
    }

    pub fn with_content_type_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("content_type"));
        self
    }



    pub fn with_content_type_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("content_type"));
        self
    }


    pub fn order_by_content_type_asc(mut self) -> Self {
        self.query = self.query.order_asc("content_type");
        self
    }

    pub fn order_by_content_type_desc(mut self) -> Self {
        self.query = self.query.order_desc("content_type");
        self
    }

    pub fn order_by_content_type_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("content_type");
        self
    }

    pub fn order_by_content_type_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("content_type");
        self
    }


    pub fn select_sha1_checksum(mut self) -> Self {
        self.query = self.query.project("sha1_checksum");
        self
    }

    pub fn project_sha1_checksum(self) -> Self {
        self.select_sha1_checksum()
    }

    pub fn select_sha1_checksum_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_sha1_checksum_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_sha1_checksum_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("sha1_checksum", raw_sql_segment));
        self
    }

    pub fn group_by_sha1_checksum(self) -> Self {
        self.group_by("sha1_checksum")
    }

    pub fn group_by_sha1_checksum_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("sha1_checksum");
        request.query = request
            .query
            .project_expr(alias, Expr::column("sha1_checksum"));
        request
    }

    pub fn group_by_sha1_checksum_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("sha1_checksum")
            .aggregate_with_function("sha1_checksum", alias, function)
    }

    pub fn count_sha1_checksum(self) -> Self {
        self.count_sha1_checksum_as("sha1_checksum_count")
    }

    pub fn count_sha1_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("sha1_checksum", alias)
    }

    pub fn sum_sha1_checksum(self) -> Self {
        self.sum_sha1_checksum_as("sum_sha1_checksum")
    }

    pub fn sum_sha1_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("sha1_checksum", alias)
    }

    pub fn avg_sha1_checksum(self) -> Self {
        self.avg_sha1_checksum_as("avg_sha1_checksum")
    }

    pub fn avg_sha1_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("sha1_checksum", alias)
    }

    pub fn min_sha1_checksum(self) -> Self {
        self.min_sha1_checksum_as("min_sha1_checksum")
    }

    pub fn min_sha1_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("sha1_checksum", alias)
    }

    pub fn max_sha1_checksum(self) -> Self {
        self.max_sha1_checksum_as("max_sha1_checksum")
    }

    pub fn max_sha1_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("sha1_checksum", alias)
    }

    pub fn unselect_sha1_checksum(mut self) -> Self {
        self.query.projection.retain(|field| field != "sha1_checksum");
        self.query_options.raw_projections.retain(|projection| projection.property_name != "sha1_checksum");
        self
    }


    pub fn with_sha1_checksum(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "sha1_checksum",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_sha1_checksum_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "sha1_checksum",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_sha1_checksum_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("sha1_checksum", value));
        self
    }



    pub fn with_sha1_checksum_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_greater_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gte("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_less_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lte("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("sha1_checksum", lower, upper));
        self
    }

    pub fn with_sha1_checksum_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "sha1_checksum",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_sha1_checksum_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "sha1_checksum",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_sha1_checksum_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "sha1_checksum",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_sha1_checksum_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("sha1_checksum", value));
        self
    }
    pub fn with_sha1_checksum_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("sha1_checksum", value));
        self
    }

    pub fn with_sha1_checksum_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("sha1_checksum"));
        self
    }



    pub fn with_sha1_checksum_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("sha1_checksum"));
        self
    }


    pub fn order_by_sha1_checksum_asc(mut self) -> Self {
        self.query = self.query.order_asc("sha1_checksum");
        self
    }

    pub fn order_by_sha1_checksum_desc(mut self) -> Self {
        self.query = self.query.order_desc("sha1_checksum");
        self
    }

    pub fn order_by_sha1_checksum_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("sha1_checksum");
        self
    }

    pub fn order_by_sha1_checksum_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("sha1_checksum");
        self
    }


    pub fn select_sha256_checksum(mut self) -> Self {
        self.query = self.query.project("sha256_checksum");
        self
    }

    pub fn project_sha256_checksum(self) -> Self {
        self.select_sha256_checksum()
    }

    pub fn select_sha256_checksum_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_sha256_checksum_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_sha256_checksum_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("sha256_checksum", raw_sql_segment));
        self
    }

    pub fn group_by_sha256_checksum(self) -> Self {
        self.group_by("sha256_checksum")
    }

    pub fn group_by_sha256_checksum_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("sha256_checksum");
        request.query = request
            .query
            .project_expr(alias, Expr::column("sha256_checksum"));
        request
    }

    pub fn group_by_sha256_checksum_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("sha256_checksum")
            .aggregate_with_function("sha256_checksum", alias, function)
    }

    pub fn count_sha256_checksum(self) -> Self {
        self.count_sha256_checksum_as("sha256_checksum_count")
    }

    pub fn count_sha256_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("sha256_checksum", alias)
    }

    pub fn sum_sha256_checksum(self) -> Self {
        self.sum_sha256_checksum_as("sum_sha256_checksum")
    }

    pub fn sum_sha256_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("sha256_checksum", alias)
    }

    pub fn avg_sha256_checksum(self) -> Self {
        self.avg_sha256_checksum_as("avg_sha256_checksum")
    }

    pub fn avg_sha256_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("sha256_checksum", alias)
    }

    pub fn min_sha256_checksum(self) -> Self {
        self.min_sha256_checksum_as("min_sha256_checksum")
    }

    pub fn min_sha256_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("sha256_checksum", alias)
    }

    pub fn max_sha256_checksum(self) -> Self {
        self.max_sha256_checksum_as("max_sha256_checksum")
    }

    pub fn max_sha256_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("sha256_checksum", alias)
    }

    pub fn unselect_sha256_checksum(mut self) -> Self {
        self.query.projection.retain(|field| field != "sha256_checksum");
        self.query_options.raw_projections.retain(|projection| projection.property_name != "sha256_checksum");
        self
    }


    pub fn with_sha256_checksum(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "sha256_checksum",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_sha256_checksum_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "sha256_checksum",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_sha256_checksum_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("sha256_checksum", value));
        self
    }



    pub fn with_sha256_checksum_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_greater_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gte("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_less_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lte("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("sha256_checksum", lower, upper));
        self
    }

    pub fn with_sha256_checksum_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "sha256_checksum",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_sha256_checksum_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "sha256_checksum",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_sha256_checksum_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "sha256_checksum",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_sha256_checksum_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("sha256_checksum", value));
        self
    }
    pub fn with_sha256_checksum_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("sha256_checksum", value));
        self
    }

    pub fn with_sha256_checksum_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("sha256_checksum"));
        self
    }



    pub fn with_sha256_checksum_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("sha256_checksum"));
        self
    }


    pub fn order_by_sha256_checksum_asc(mut self) -> Self {
        self.query = self.query.order_asc("sha256_checksum");
        self
    }

    pub fn order_by_sha256_checksum_desc(mut self) -> Self {
        self.query = self.query.order_desc("sha256_checksum");
        self
    }

    pub fn order_by_sha256_checksum_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("sha256_checksum");
        self
    }

    pub fn order_by_sha256_checksum_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("sha256_checksum");
        self
    }


    pub fn select_md5_checksum(mut self) -> Self {
        self.query = self.query.project("md5_checksum");
        self
    }

    pub fn project_md5_checksum(self) -> Self {
        self.select_md5_checksum()
    }

    pub fn select_md5_checksum_raw(self, raw_sql_segment: impl Into<String>) -> Self {
        self.select_md5_checksum_unsafe_raw(UnsafeRawSqlSegment::trusted(raw_sql_segment))
    }

    pub fn select_md5_checksum_unsafe_raw(mut self, raw_sql_segment: UnsafeRawSqlSegment) -> Self {
        self.query_options
            .raw_projections
            .push(RawProjection::new("md5_checksum", raw_sql_segment));
        self
    }

    pub fn group_by_md5_checksum(self) -> Self {
        self.group_by("md5_checksum")
    }

    pub fn group_by_md5_checksum_as(self, alias: impl Into<String>) -> Self {
        let alias = alias.into();
        let mut request = self.group_by("md5_checksum");
        request.query = request
            .query
            .project_expr(alias, Expr::column("md5_checksum"));
        request
    }

    pub fn group_by_md5_checksum_with_function(
        self,
        alias: impl Into<String>,
        function: AggregateFunction,
    ) -> Self {
        self.group_by("md5_checksum")
            .aggregate_with_function("md5_checksum", alias, function)
    }

    pub fn count_md5_checksum(self) -> Self {
        self.count_md5_checksum_as("md5_checksum_count")
    }

    pub fn count_md5_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_count_field("md5_checksum", alias)
    }

    pub fn sum_md5_checksum(self) -> Self {
        self.sum_md5_checksum_as("sum_md5_checksum")
    }

    pub fn sum_md5_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_sum("md5_checksum", alias)
    }

    pub fn avg_md5_checksum(self) -> Self {
        self.avg_md5_checksum_as("avg_md5_checksum")
    }

    pub fn avg_md5_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_avg("md5_checksum", alias)
    }

    pub fn min_md5_checksum(self) -> Self {
        self.min_md5_checksum_as("min_md5_checksum")
    }

    pub fn min_md5_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_min("md5_checksum", alias)
    }

    pub fn max_md5_checksum(self) -> Self {
        self.max_md5_checksum_as("max_md5_checksum")
    }

    pub fn max_md5_checksum_as(self, alias: impl Into<String>) -> Self {
        self.aggregate_max("md5_checksum", alias)
    }

    pub fn unselect_md5_checksum(mut self) -> Self {
        self.query.projection.retain(|field| field != "md5_checksum");
        self.query_options.raw_projections.retain(|projection| projection.property_name != "md5_checksum");
        self
    }


    pub fn with_md5_checksum(
        mut self,
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(field_operator_expr(
            "md5_checksum",
            operator,
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    pub fn create_md5_checksum_criteria(
        operator: FieldOperator,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Expr {
        field_operator_expr(
            "md5_checksum",
            operator,
            values.into_iter().map(Into::into).collect(),
        )
    }

    pub fn with_md5_checksum_is(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::eq("md5_checksum", value));
        self
    }



    pub fn with_md5_checksum_is_not(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::ne("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_greater_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_greater_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gte("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_less_than(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_less_than_or_equal_to(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lte("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_between(
        mut self,
        lower: impl Into<teaql_core::Value>,
        upper: impl Into<teaql_core::Value>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::between("md5_checksum", lower, upper));
        self
    }

    pub fn with_md5_checksum_between_range<T>(mut self, range: DateRange<T>) -> Self
    where
        T: Into<teaql_core::Value>,
    {
        self.query = self.query.and_filter(Expr::between(
            "md5_checksum",
            range.start,
            range.end,
        ));
        self
    }

    pub fn with_md5_checksum_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::in_list(
            "md5_checksum",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_md5_checksum_not_in(
        mut self,
        values: impl IntoIterator<Item = impl Into<teaql_core::Value>>,
    ) -> Self {
        self.query = self.query.and_filter(Expr::not_in_list(
            "md5_checksum",
            values.into_iter().map(Into::into),
        ));
        self
    }

    pub fn with_md5_checksum_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::contain("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_not_containing(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_contain("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::begin_with("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_not_starting_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_begin_with("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::end_with("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_not_ending_with(mut self, value: impl Into<String>) -> Self {
        self.query = self.query.and_filter(Expr::not_end_with("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_sounding_like(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::sound_like("md5_checksum", value));
        self
    }
    pub fn with_md5_checksum_before(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::lt("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_after(mut self, value: impl Into<teaql_core::Value>) -> Self {
        self.query = self.query.and_filter(Expr::gt("md5_checksum", value));
        self
    }

    pub fn with_md5_checksum_is_unknown(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_null("md5_checksum"));
        self
    }



    pub fn with_md5_checksum_is_known(mut self) -> Self {
        self.query = self.query.and_filter(Expr::is_not_null("md5_checksum"));
        self
    }


    pub fn order_by_md5_checksum_asc(mut self) -> Self {
        self.query = self.query.order_asc("md5_checksum");
        self
    }

    pub fn order_by_md5_checksum_desc(mut self) -> Self {
        self.query = self.query.order_desc("md5_checksum");
        self
    }

    pub fn order_by_md5_checksum_asc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_asc("md5_checksum");
        self
    }

    pub fn order_by_md5_checksum_desc_using_gbk(mut self) -> Self {
        self.query = self.query.order_gbk_desc("md5_checksum");
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
    pub fn have_assets(self) -> Self {
        self.with_asset_list_matching(SelectQuery::new("Asset"))
    }

    pub fn have_no_assets(self) -> Self {
        self.without_asset_list_matching(SelectQuery::new("Asset"))
    }

    pub fn with_asset_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::in_subquery(
            "id",
            <crate::Asset as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "asset_blob_id",
        ));
        self.relation_filters.push(RelationFilter::new("asset_list", selection));
        self
    }

    pub fn without_asset_list_matching(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.and_filter(Expr::not_in_subquery(
            "id",
            <crate::Asset as teaql_core::TeaqlEntity>::entity_descriptor(),
            selection.query.clone(),
            "asset_blob_id",
        ));
        self.relation_filters.push(RelationFilter::new("asset_list", selection));
        self
    }

    pub fn select_asset_list(mut self) -> Self {
        self.query = self.query.relation("asset_list");
        self
    }

    pub fn select_asset_list_with(mut self, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query = self.query.relation_query("asset_list", selection.clone().into_query());
        self.relation_selections.push(RelationSelection::new("asset_list", selection));
        self
}
    pub fn count_assets(self) -> Self {
        self.count_assets_as("count_assets")
    }

    pub fn count_assets_as(self, alias: impl Into<String>) -> Self {
        self.count_assets_with(alias, crate::Q::assets().unlimited())
    }

    pub fn count_assets_with(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "asset_list",
            alias,
            selection,
            true,
        ));
        self
    }

    pub fn stats_from_assets(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_assets_as("refinements", request)
    }

    pub fn stats_from_assets_as(mut self, alias: impl Into<String>, request: impl Into<QuerySelection>) -> Self {
        let selection = request.into();
        self.query_options.relation_aggregates.push(RelationAggregate::new(
            "asset_list",
            alias,
            selection,
            false,
        ));
        self
    }

    pub fn group_by_assets_with_details(self, request: impl Into<QuerySelection>) -> Self {
        self.stats_from_assets(request)
    }



}

impl<R> Default for AssetBlobRequest<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> From< AssetBlobRequest<R> > for SelectQuery {
    fn from(request: AssetBlobRequest<R>) -> Self {
        QuerySelection::from(request).into_query()
    }
}

impl<R> From< AssetBlobRequest<R> > for QuerySelection {
    fn from(request: AssetBlobRequest<R>) -> Self {
        Self {
            query: request.query,
            relation_selections: request.relation_selections,
            relation_filters: request.relation_filters,
            child_enhancements: request.child_enhancements,
            query_options: request.query_options,
        }
    }
}


impl<'a, C> crate::request_support::AuditedSave<'a, C> for teaql_core::Audited<crate::AssetBlob> 
where C: crate::request_support::TeaqlRepositoryProvider + ?Sized + 'a
{
    type Error = crate::TeaqlDataServiceError<C::AssetBlobRepository<'a>>;
    fn save(self, ctx: &'a C) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<teaql_runtime::GraphNode, Self::Error>> + '_>> {
        Box::pin(async move {
            teaql_runtime::save_audited_ledger_entity(self, ctx.user_context())
                .await
                .map_err(DataServiceError::Runtime)
        })
    }
}

impl<R: teaql_core::Entity> crate::PurposedQuery<AssetBlobRequest<R>> {
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.inner.query_options.comment = Some(comment.into());
        self
    }

    pub fn new_entity<C>(&self, ctx: &C) -> crate::AssetBlob
    where
        C: crate::TeaqlRuntime + ?Sized,
    {
        let mut entity = crate::AssetBlob::runtime_new(ctx.user_context().entity_root());
        if let Ok(id) = ctx.user_context().next_id(crate::AssetBlob::ENTITY_NAME) {
            entity.update_id(id);
        }
        entity
    }

    fn into_inner_with_trace(mut self) -> AssetBlobRequest<R> {
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
    ) -> Result<teaql_core::SmartList<R>, crate::request_support::TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_page(ctx, offset, limit).await
    }

    pub async fn execute_for_exists<'a, C>(
        self,
        ctx: &'a C,
    ) -> Result<bool, crate::request_support::TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_exists(ctx).await
    }

    pub async fn execute_for_list<'a, C>(self, ctx: &'a C) -> Result<teaql_core::SmartList<R>, crate::request_support::TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_list(ctx).await
    }

    /// Execute query as a lazy entity stream without materializing the result set.
    /// Set chunk size via .stream(chunk_size) or .stream_default() on the query.
    pub async fn execute_for_stream<'a, C>(self, ctx: &'a C) -> Result<crate::request_support::TeaqlEntityStream<'a, R, crate::request_support::TeaqlDataServiceError<C::AssetBlobRepository<'a>>>, crate::request_support::TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
        R: teaql_core::Entity + 'a,
    {
        self.into_inner_with_trace()._execute_for_stream(ctx).await
    }

    pub async fn execute_for_first<'a, C>(self, ctx: &'a C) -> Result<Option<R>, crate::request_support::TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_first(ctx).await
    }

    pub async fn execute_for_one<'a, C>(self, ctx: &'a C) -> Result<Option<R>, crate::request_support::TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_one(ctx).await
    }


    pub async fn execute_for_records<'a, C>(self, ctx: &'a C) -> Result<teaql_core::SmartList<teaql_core::Record>, crate::request_support::TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_records(ctx).await
    }

    pub async fn execute_for_record<'a, C>(self, ctx: &'a C) -> Result<Option<teaql_core::Record>, crate::request_support::TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_record(ctx).await
    }

    pub async fn execute_for_count<'a, C>(self, ctx: &'a C) -> Result<u64, crate::request_support::TeaqlDataServiceError<C::AssetBlobRepository<'a>>>
    where
        C: crate::request_support::TeaqlRepositoryProvider + ?Sized,
    {
        self.into_inner_with_trace()._execute_for_count(ctx).await
    }
}
