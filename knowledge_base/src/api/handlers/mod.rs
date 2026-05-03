pub mod query;
pub mod write;

// Re-export AppError so routes.rs can use it.
pub use query::AppError;
