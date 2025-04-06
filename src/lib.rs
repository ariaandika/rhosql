

pub mod connection;

pub mod error;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

