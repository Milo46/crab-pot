use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::{query_params::LogQueryParams, SchemaQueryParams};

macro_rules! impl_common_builder_methods {
    ($builder:ty) => {
        impl<'a> $builder {
            fn add_condition(&mut self) {
                if !self.conditions_added {
                    self.query.push(" WHERE ");
                    self.conditions_added = true;
                } else {
                    self.query.push(" AND ");
                }
            }

            pub fn order_by(mut self, column: &str, direction: &str) -> Self {
                let valid_columns = ["id", "created_at", "name", "version"];
                let valid_directions = ["ASC", "DESC"];

                assert!(valid_columns.contains(&column), "Invalid column");
                assert!(valid_directions.contains(&direction), "Invalid direction");

                self.query.push(" ORDER BY ");
                self.query.push(column);
                self.query.push(" ");
                self.query.push(direction);
                self
            }

            pub fn then_order_by(mut self, column: &str, direction: &str) -> Self {
                self.query.push(", ");
                self.query.push(column);
                self.query.push(" ");
                self.query.push(direction);
                self
            }

            pub fn limit(mut self, limit: i32) -> Self {
                self.query.push(" LIMIT ");
                self.query.push_bind(limit);
                self
            }

            pub fn build(self) -> QueryBuilder<'a, Postgres> {
                self.query
            }
        }
    };
}

pub struct SchemaQueryBuilder<'a> {
    query: QueryBuilder<'a, Postgres>,
    conditions_added: bool,
}

impl<'a> SchemaQueryBuilder<'a> {
    pub fn select() -> Self {
        let query = QueryBuilder::new("SELECT * FROM schemas");
        Self {
            query,
            conditions_added: false,
        }
    }

    pub fn filters(mut self, params: Option<&'a SchemaQueryParams>) -> Self {
        if let Some(query_params) = params {
            if let Some(name) = &query_params.name {
                self.add_condition();
                self.query.push("name = ");
                self.query.push_bind(name);
            }
            if let Some(version) = &query_params.version {
                self.add_condition();
                self.query.push("version = ");
                self.query.push_bind(version);
            }
        }
        self
    }

    pub fn cursor(mut self, cursor_id: Option<Uuid>) -> Self {
        if let Some(id) = cursor_id {
            self.add_condition();
            self.query
                .push("(created_at < (SELECT created_at FROM schemas WHERE id = ");
            self.query.push_bind(id);
            self.query
                .push(") OR (created_at = (SELECT created_at FROM schemas WHERE id = ");
            self.query.push_bind(id);
            self.query.push(") AND id < ");
            self.query.push_bind(id);
            self.query.push("))");
        }
        self
    }
}

impl_common_builder_methods!(SchemaQueryBuilder<'a>);

pub struct LogQueryBuilder<'a> {
    query: QueryBuilder<'a, Postgres>,
    conditions_added: bool,
}

impl<'a> LogQueryBuilder<'a> {
    pub fn select() -> Self {
        let query = QueryBuilder::new("SELECT * FROM logs");
        Self {
            query,
            conditions_added: false,
        }
    }

    pub fn count() -> Self {
        let query = QueryBuilder::new("SELECT COUNT(*) FROM logs");
        Self {
            query,
            conditions_added: false,
        }
    }

    pub fn schema_id(mut self, schema_id: Uuid) -> Self {
        self.add_condition();
        self.query.push("schema_id = ");
        self.query.push_bind(schema_id);
        self
    }

    pub fn filters(mut self, params: Option<&'a LogQueryParams>) -> Self {
        if let Some(query_params) = params {
            if let Some(filter_value) = &query_params.json_filters {
                if filter_value.as_object().is_some() {
                    self.add_condition();
                    self.query.push("log_data @> ");
                    self.query.push_bind(filter_value);
                }
            }
            if let (Some(begin), Some(end)) = (query_params.date_begin, query_params.date_end) {
                self.add_condition();
                self.query.push("created_at BETWEEN ");
                self.query.push_bind(begin);
                self.query.push(" AND ");
                self.query.push_bind(end);
            }
        }
        self
    }

    pub fn cursor(mut self, cursor_id: Option<i32>) -> Self {
        if let Some(id) = cursor_id {
            self.add_condition();
            self.query
                .push("(created_at < (SELECT created_at FROM logs WHERE id = ");
            self.query.push_bind(id);
            self.query
                .push(") OR (created_at = (SELECT created_at FROM logs WHERE id = ");
            self.query.push_bind(id);
            self.query.push(") AND id < ");
            self.query.push_bind(id);
            self.query.push("))");
        }
        self
    }
}

impl_common_builder_methods!(LogQueryBuilder<'a>);
