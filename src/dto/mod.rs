pub mod common;
pub mod log_dto;
pub mod schema_dto;

pub use common::ErrorResponse;

pub use schema_dto::{
    // Requests
    CreateSchemaRequest,
    DeleteSchemaQuery,
    // Queries
    GetSchemasQuery,
    // Responses
    SchemaResponse,
    UpdateSchemaRequest,
};

pub use log_dto::{
    // Requests
    CreateLogRequest,
    // Responses
    LogResponse,
};
