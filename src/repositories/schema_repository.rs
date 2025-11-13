use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::Schema;
use anyhow::Result;

#[async_trait]
pub trait SchemaRepositoryTrait {
    async fn get_all(&self) -> Result<Vec<Schema>>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Schema>>;
    async fn get_by_name_and_version(&self, name: &str, version: &str) -> Result<Option<Schema>>;
    async fn create(&self, schema: &Schema) -> Result<Schema>;
    async fn update(&self, id: Uuid, schema: &Schema) -> Result<Option<Schema>>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
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
    async fn get_all(&self) -> Result<Vec<Schema>> {
        let schemas = sqlx::query_as::<_, Schema>("SELECT * FROM schemas ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        Ok(schemas)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Schema>> {
        let schema = sqlx::query_as::<_, Schema>("SELECT * FROM schemas WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(schema)
    }

    async fn get_by_name_and_version(&self, name: &str, version: &str) -> Result<Option<Schema>> {
        let schema = sqlx::query_as::<_, Schema>("SELECT * FROM schemas WHERE name = $1 AND version = $2")
            .bind(name)
            .bind(version)
            .fetch_optional(&self.pool)
            .await?;
        Ok(schema)
    }

    async fn create(&self, schema: &Schema) -> Result<Schema> {
        let created_schema = sqlx::query_as::<_, Schema>(
            r#"
            INSERT INTO schemas (id, name, version, description, schema_definition, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(&schema.id)
        .bind(&schema.name)
        .bind(&schema.version)
        .bind(&schema.description)
        .bind(&schema.schema_definition)
        .bind(&schema.created_at)
        .bind(&schema.updated_at)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(created_schema)
    }

    async fn update(&self, id: Uuid, schema: &Schema) -> Result<Option<Schema>> {
        let updated_schema = sqlx::query_as::<_, Schema>(
            r#"
            UPDATE schemas 
            SET name = $2, version = $3, description = $4, schema_definition = $5, updated_at = $6
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(&schema.name)
        .bind(&schema.version)
        .bind(&schema.description)
        .bind(&schema.schema_definition)
        .bind(&schema.updated_at)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(updated_schema)
    }

    async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM schemas WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
}
