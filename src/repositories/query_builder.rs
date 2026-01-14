use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

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

    fn add_condition(&mut self) {
        if !self.conditions_added {
            self.query.push(" WHERE ");
            self.conditions_added = true;
        } else {
            self.query.push(" AND ");
        }
    }

    pub fn schema_id(mut self, schema_id: Uuid) -> Self {
        self.add_condition();
        self.query.push("schema_id = ");
        self.query.push_bind(schema_id);
        self
    }

    pub fn filters(mut self, filters: Option<&'a Value>) -> Self {
        if let Some(filter_value) = filters {
            if filter_value.as_object().is_some() {
                self.add_condition();
                self.query.push("log_data @> ");
                self.query.push_bind(filter_value);
            }
        }
        self
    }

    pub fn date_range(
        mut self,
        date_begin: Option<DateTime<Utc>>,
        date_end: Option<DateTime<Utc>>,
    ) -> Self {
        if let (Some(begin), Some(end)) = (date_begin, date_end) {
            self.add_condition();
            self.query.push("created_at BETWEEN ");
            self.query.push_bind(begin);
            self.query.push(" AND ");
            self.query.push_bind(end);
        }
        self
    }

    pub fn order_by(mut self, column: &str, direction: &str) -> Self {
        self.query.push(" ORDER BY ");
        self.query.push(column);
        self.query.push(" ");
        self.query.push(direction);
        self
    }

    pub fn paginate(mut self, page: i32, limit: i32) -> Self {
        let offset = (page - 1) * limit;
        self.query.push(" LIMIT ");
        self.query.push_bind(limit);
        self.query.push(" OFFSET ");
        self.query.push_bind(offset);
        self
    }

    pub fn build(self) -> QueryBuilder<'a, Postgres> {
        self.query
    }
}
