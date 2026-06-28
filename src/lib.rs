mod proto;

pub mod models;
pub mod client;

pub use client::ArgoClient;
pub use models::{Application, HealthStatus, SyncStatus, SyncOptions};