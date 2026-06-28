use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Application {
    pub metadata: ApplicationMetadata,
    pub spec: Option<ApplicationSpec>,
    pub status: Option<ApplicationStatus>,
    pub operation: Option<ApplicationOperation>,

    pub name: String,
    pub namespace: String,
    pub self_link: String,
    pub uid: String,
    pub project: String,
    pub health: HealthStatus,
    pub sync: SyncStatus,
    pub repo_url: String,
    pub path: String,
    pub target_revision: String,
    pub image: Option<String>,
    pub external_urls: Vec<String>,
    pub source_types: Vec<String>,
    pub is_app_of_apps: bool,
    pub source_type: String,
    pub controller_namespace: String,
    pub resource_health_source: String,
    pub revision_history_limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationMetadata {
    pub name: String,
    pub namespace: String,
    pub self_link: String,
    pub uid: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub resource_version: String,
    pub generation: i64,
    pub creation_timestamp: Option<KubeTime>,
    pub deletion_timestamp: Option<KubeTime>,
    pub finalizers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationSpec {
    pub project: String,
    pub source: Option<ApplicationSource>,
    pub destination: Option<ApplicationDestination>,
    pub revision_history_limit: Option<i64>,
    pub sources: Vec<ApplicationSource>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationSource {
    pub repo_url: String,
    pub path: String,
    pub target_revision: String,
    pub chart: String,
    pub r#ref: String,
    pub name: String,
    pub tag_prefix: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationDestination {
    pub server: String,
    pub namespace: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationStatus {
    pub sync: SyncStatus,
    pub health: HealthStatus,
    pub conditions: Vec<ApplicationCondition>,
    pub reconciled_at: Option<KubeTime>,
    pub observed_at: Option<KubeTime>,
    pub source_type: String,
    pub summary: ApplicationSummary,
    pub source_types: Vec<String>,
    pub controller_namespace: String,
    pub resource_health_source: String,
    pub operation_state: Option<ApplicationOperationState>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationSummary {
    pub external_urls: Vec<String>,
    pub images: Vec<String>,
    pub is_app_of_apps: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationOperationState {
    pub phase: String,
    pub message: String,
    pub retry_count: Option<i64>,
    pub started_at: Option<KubeTime>,
    pub finished_at: Option<KubeTime>,
    pub operation: Option<ApplicationOperation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationOperation {
    pub initiated_by: Option<OperationInitiator>,
    pub info: Vec<ApplicationInfo>,
    pub retry_limit: Option<i64>,
    pub retry_refresh: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationInitiator {
    pub username: String,
    pub automated: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationInfo {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationCondition {
    pub r#type: String,
    pub message: String,
    pub last_transition_time: Option<KubeTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KubeTime {
    pub seconds: Option<i64>,
    pub nanos: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Progressing,
    Suspended,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Synced,
    OutOfSync,
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy     => write!(f, "Healthy"),
            Self::Degraded    => write!(f, "Degraded"),
            Self::Progressing => write!(f, "Progressing"),
            Self::Suspended   => write!(f, "Suspended"),
            Self::Missing     => write!(f, "Missing"),
            Self::Unknown     => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Synced    => write!(f, "Synced"),
            Self::OutOfSync => write!(f, "OutOfSync"),
            Self::Unknown   => write!(f, "Unknown"),
        }
    }
}