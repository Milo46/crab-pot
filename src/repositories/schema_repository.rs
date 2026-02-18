use crate::error::AppResult;
use crate::models::{Schema, SchemaQueryParams};
use crate::repositories::query_builder::SchemaQueryBuilder;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait SchemaRepositoryTrait {
    async fn get_all(&self, params: Option<SchemaQueryParams>) -> AppResult<Vec<Schema>>;
    async fn get_all_with_cursor(
        &self,
        cursor: Option<Uuid>,
        limit: i32,
        filters: SchemaQueryParams,
    ) -> AppResult<Vec<Schema>>;
    async fn get_by_id(&self, id: Uuid) -> AppResult<Option<Schema>>;
    async fn get_by_name_latest(&self, name: &str) -> AppResult<Option<Schema>>;
    async fn get_by_name_and_version(&self, name: &str, version: &str)
        -> AppResult<Option<Schema>>;
    async fn get_latest_schema_id(&self) -> AppResult<Option<Uuid>>;
    async fn create(&self, schema: &Schema) -> AppResult<Schema>;
    async fn update(&self, id: Uuid, schema: &Schema) -> AppResult<Option<Schema>>;
    async fn delete(&self, id: Uuid) -> AppResult<Option<Schema>>;
}

#[derive(Clone)]
pub struct SchemaRepository {
    pool: PgPool,
}

impl SchemaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SchemaRepositoryTrait for SchemaRepository {
    async fn get_all(&self, params: Option<SchemaQueryParams>) -> AppResult<Vec<Schema>> {
        let schemas = SchemaQueryBuilder::select()
            .filters(params.as_ref())
            .order_by("created_at", "DESC")
            .build()
            .build_query_as::<Schema>()
            .fetch_all(&self.pool)
            .await?;

        Ok(schemas)
    }

    async fn get_all_with_cursor(
        &self,
        cursor: Option<Uuid>,
        limit: i32,
        filters: SchemaQueryParams,
    ) -> AppResult<Vec<Schema>> {
        let fetch_limit = limit + 1;

        let schemas = SchemaQueryBuilder::select()
            .filters(Some(&filters))
            .cursor(cursor)
            .order_by("created_at", "DESC")
            .then_order_by("id", "DESC")
            .limit(fetch_limit)
            .build()
            .build_query_as::<Schema>()
            .fetch_all(&self.pool)
            .await?;

        Ok(schemas)
    }

    async fn get_by_name_latest(&self, name: &str) -> AppResult<Option<Schema>> {
        let schema = sqlx::query_as::<_, Schema>(
            r#"
            SELECT *
            FROM schemas
            WHERE name = $1
            ORDER BY
                (string_to_array(version, '.'))[1]::int DESC,
                (string_to_array(version, '.'))[2]::int DESC,
                (string_to_array(version, '.'))[3]::int DESC
            LIMIT 1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(schema)
    }

    async fn get_by_id(&self, id: Uuid) -> AppResult<Option<Schema>> {
        let schema = sqlx::query_as::<_, Schema>("SELECT * FROM schemas WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(schema)
    }

    async fn get_by_name_and_version(
        &self,
        name: &str,
        version: &str,
    ) -> AppResult<Option<Schema>> {
        let schema =
            sqlx::query_as::<_, Schema>("SELECT * FROM schemas WHERE name = $1 AND version = $2")
                .bind(name)
                .bind(version)
                .fetch_optional(&self.pool)
                .await?;

        Ok(schema)
    }

    async fn get_latest_schema_id(&self) -> AppResult<Option<Uuid>> {
        let result = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id FROM schemas
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn create(&self, schema: &Schema) -> AppResult<Schema> {
        let created_schema = sqlx::query_as::<_, Schema>(
            r#"
            INSERT INTO schemas (id, name, version, description, schema_definition, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(schema.id)
        .bind(&schema.name)
        .bind(&schema.version)
        .bind(&schema.description)
        .bind(&schema.schema_definition)
        .bind(schema.created_at)
        .bind(schema.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(created_schema)
    }

    async fn update(&self, id: Uuid, schema: &Schema) -> AppResult<Option<Schema>> {
        let updated_schema = sqlx::query_as::<_, Schema>(
            r#"
            UPDATE schemas 
            SET name = $2, version = $3, description = $4, schema_definition = $5, updated_at = $6
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&schema.name)
        .bind(&schema.version)
        .bind(&schema.description)
        .bind(&schema.schema_definition)
        .bind(schema.updated_at)
        .fetch_optional(&self.pool)
        .await?;

        Ok(updated_schema)
    }

    async fn delete(&self, id: Uuid) -> AppResult<Option<Schema>> {
        let deleted_schema =
            sqlx::query_as::<_, Schema>("DELETE FROM schemas WHERE id = $1 RETURNING *")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(deleted_schema)
    }
}
