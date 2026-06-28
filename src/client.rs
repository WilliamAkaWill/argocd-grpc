use http::header::CONTENT_TYPE;
use std::collections::HashMap;
use tower::Layer;
use tonic::metadata::MetadataValue;
use tonic_web::GrpcWebClientLayer;
use crate::proto::application::application_service_client::ApplicationServiceClient;
use crate::proto::application::ApplicationQuery;
use crate::proto::github::com::argoproj::argo_cd::v3::pkg::apis::application::v1alpha1::{
    Application as ProtoApplication,
    ApplicationCondition as ProtoApplicationCondition,
    ApplicationDestination as ProtoApplicationDestination,
    ApplicationSource as ProtoApplicationSource,
    ApplicationSpec as ProtoApplicationSpec,
    ApplicationStatus as ProtoApplicationStatus,
    ApplicationSummary as ProtoApplicationSummary,
    Info as ProtoInfo,
    Operation as ProtoOperation,
    OperationInitiator as ProtoOperationInitiator,
    OperationState as ProtoOperationState,
};
use crate::proto::k8s::io::apimachinery::pkg::apis::meta::v1::{ObjectMeta as ProtoObjectMeta, Time as ProtoTime};

#[derive(Clone)]
struct OverrideContentTypeLayer;

#[derive(Clone)]
struct OverrideContentTypeService<S>(S);

impl<S> Layer<S> for OverrideContentTypeLayer {
    type Service = OverrideContentTypeService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        OverrideContentTypeService(inner)
    }
}

impl<S, B> tower::Service<http::Request<B>> for OverrideContentTypeService<S>
where
    S: tower::Service<http::Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<B>) -> Self::Future {
        req.headers_mut().insert(
            CONTENT_TYPE,
            http::HeaderValue::from_static("application/grpc-web+proto"),
        );
        self.0.call(req)
    }
}

pub struct ArgoClient {
    url: String,
    token: String,
}

impl ArgoClient {
    pub fn new(url: &str, token: &str) -> anyhow::Result<Self> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        Ok(Self {
            url: url.to_string(),
            token: token.to_string(),
        })
    }

    fn request<T>(&self, body: T) -> tonic::Request<T> {
        let mut req = tonic::Request::new(body);
        let auth: MetadataValue<_> = format!("Bearer {}", self.token).parse().unwrap();
        req.metadata_mut().insert("authorization", auth);
        req.metadata_mut().insert("x-grpc-web", MetadataValue::from_static("1"));
        req
    }

    pub async fn list_applications(&self) -> anyhow::Result<Vec<crate::models::Application>> {
        // Client direkt hier bauen – Typ wird korrekt inferiert
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()?
            .https_only()
            .enable_http2()
            .build();

        let hyper_client = hyper_util::client::legacy::Client::builder(
            hyper_util::rt::TokioExecutor::new()
        ).build(connector);

        let svc = tower::ServiceBuilder::new()
            .layer(GrpcWebClientLayer::new())
            .layer(OverrideContentTypeLayer)
            .service(hyper_client);

        let mut client = ApplicationServiceClient::with_origin(svc, self.url.parse()?);

        let response = client
            .list(self.request(ApplicationQuery {
                projects: vec![],
                ..Default::default()
            }))
            .await?
            .into_inner();

        Ok(response.items.iter().map(Self::map_application).collect())
    }

    pub async fn get_application(&self, name: &str) -> anyhow::Result<crate::models::Application> {
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()?
            .https_only()
            .enable_http2()
            .build();

        let hyper_client = hyper_util::client::legacy::Client::builder(
            hyper_util::rt::TokioExecutor::new()
        ).build(connector);

        let svc = tower::ServiceBuilder::new()
            .layer(GrpcWebClientLayer::new())
            .layer(OverrideContentTypeLayer)
            .service(hyper_client);

        let mut client = ApplicationServiceClient::with_origin(svc, self.url.parse()?);

        let response = client
            .get(self.request(ApplicationQuery {
                name: Some(name.to_string()),
                ..Default::default()
            }))
            .await?
            .into_inner();

        Ok(Self::map_application(&response))
    }

    fn map_application(app: &ProtoApplication) -> crate::models::Application {
        let status = app.status.as_ref();

        crate::models::Application {
            metadata: app
                .metadata
                .as_ref()
                .map(Self::map_metadata)
                .unwrap_or_else(Self::empty_metadata),
            spec: app.spec.as_ref().map(Self::map_spec),
            status: status.map(Self::map_status),
            operation: app.operation.as_ref().map(Self::map_operation),
            name: app.metadata.as_ref().and_then(|m| m.name.as_deref()).unwrap_or("").to_string(),
            namespace: app.metadata.as_ref().and_then(|m| m.namespace.as_deref()).unwrap_or("").to_string(),
            self_link: app.metadata.as_ref().and_then(|m| m.self_link.as_deref()).unwrap_or("").to_string(),
            uid: app.metadata.as_ref().and_then(|m| m.uid.as_deref()).unwrap_or("").to_string(),
            project: app.spec.as_ref().and_then(|s| s.project.as_deref()).unwrap_or("").to_string(),
            health: Self::map_health(status),
            sync: Self::map_sync(status),
            repo_url: app.spec.as_ref().and_then(|s| s.source.as_ref()).and_then(|src| src.repo_url.as_deref()).unwrap_or("").to_string(),
            path: app.spec.as_ref().and_then(|s| s.source.as_ref()).and_then(|src| src.path.as_deref()).unwrap_or("").to_string(),
            target_revision: app.spec.as_ref().and_then(|s| s.source.as_ref()).and_then(|src| src.target_revision.as_deref()).unwrap_or("").to_string(),
            image: status.and_then(|s| s.summary.as_ref()).and_then(|summary| summary.images.first()).cloned(),
            external_urls: status
                .and_then(|s| s.summary.as_ref())
                .map(|summary| summary.external_ur_ls.clone())
                .unwrap_or_default(),
            source_types: status.map(|s| s.source_types.clone()).unwrap_or_default(),
            is_app_of_apps: status
                .and_then(|s| s.summary.as_ref())
                .and_then(|summary| summary.is_app_of_apps)
                .unwrap_or(false),
            source_type: status.and_then(|s| s.source_type.as_deref()).unwrap_or("").to_string(),
            controller_namespace: status
                .and_then(|s| s.controller_namespace.as_deref())
                .unwrap_or("")
                .to_string(),
            resource_health_source: status
                .and_then(|s| s.resource_health_source.as_deref())
                .unwrap_or("")
                .to_string(),
            revision_history_limit: app.spec.as_ref().and_then(|s| s.revision_history_limit),
        }
    }

    fn empty_metadata() -> crate::models::ApplicationMetadata {
        crate::models::ApplicationMetadata {
            name: String::new(),
            namespace: String::new(),
            self_link: String::new(),
            uid: String::new(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            resource_version: String::new(),
            generation: 0,
            creation_timestamp: None,
            deletion_timestamp: None,
            finalizers: Vec::new(),
        }
    }

    fn map_metadata(meta: &ProtoObjectMeta) -> crate::models::ApplicationMetadata {
        crate::models::ApplicationMetadata {
            name: meta.name.clone().unwrap_or_default(),
            namespace: meta.namespace.clone().unwrap_or_default(),
            self_link: meta.self_link.clone().unwrap_or_default(),
            uid: meta.uid.clone().unwrap_or_default(),
            labels: meta.labels.clone(),
            annotations: meta.annotations.clone(),
            resource_version: meta.resource_version.clone().unwrap_or_default(),
            generation: meta.generation.unwrap_or_default(),
            creation_timestamp: meta.creation_timestamp.as_ref().map(Self::map_time),
            deletion_timestamp: meta.deletion_timestamp.as_ref().map(Self::map_time),
            finalizers: meta.finalizers.clone(),
        }
    }

    fn map_spec(spec: &ProtoApplicationSpec) -> crate::models::ApplicationSpec {
        crate::models::ApplicationSpec {
            project: spec.project.clone().unwrap_or_default(),
            source: spec.source.as_ref().map(Self::map_source),
            destination: spec.destination.as_ref().map(Self::map_destination),
            revision_history_limit: spec.revision_history_limit,
            sources: spec.sources.iter().map(Self::map_source).collect(),
        }
    }

    fn map_source(source: &ProtoApplicationSource) -> crate::models::ApplicationSource {
        crate::models::ApplicationSource {
            repo_url: source.repo_url.clone().unwrap_or_default(),
            path: source.path.clone().unwrap_or_default(),
            target_revision: source.target_revision.clone().unwrap_or_default(),
            chart: source.chart.clone().unwrap_or_default(),
            r#ref: source.r#ref.clone().unwrap_or_default(),
            name: source.name.clone().unwrap_or_default(),
            tag_prefix: source.tag_prefix.clone().unwrap_or_default(),
        }
    }

    fn map_destination(destination: &ProtoApplicationDestination) -> crate::models::ApplicationDestination {
        crate::models::ApplicationDestination {
            server: destination.server.clone().unwrap_or_default(),
            namespace: destination.namespace.clone().unwrap_or_default(),
            name: destination.name.clone().unwrap_or_default(),
        }
    }

    fn map_status(status: &ProtoApplicationStatus) -> crate::models::ApplicationStatus {
        crate::models::ApplicationStatus {
            sync: Self::map_sync(Some(status)),
            health: Self::map_health(Some(status)),
            conditions: status.conditions.iter().map(Self::map_condition).collect(),
            reconciled_at: status.reconciled_at.as_ref().map(Self::map_time),
            observed_at: status.observed_at.as_ref().map(Self::map_time),
            source_type: status.source_type.clone().unwrap_or_default(),
            summary: status.summary.as_ref().map(Self::map_summary).unwrap_or_else(|| crate::models::ApplicationSummary {
                external_urls: Vec::new(),
                images: Vec::new(),
                is_app_of_apps: false,
            }),
            source_types: status.source_types.clone(),
            controller_namespace: status.controller_namespace.clone().unwrap_or_default(),
            resource_health_source: status.resource_health_source.clone().unwrap_or_default(),
            operation_state: status.operation_state.as_ref().map(Self::map_operation_state),
        }
    }

    fn map_summary(summary: &ProtoApplicationSummary) -> crate::models::ApplicationSummary {
        crate::models::ApplicationSummary {
            external_urls: summary.external_ur_ls.clone(),
            images: summary.images.clone(),
            is_app_of_apps: summary.is_app_of_apps.unwrap_or(false),
        }
    }

    fn map_operation_state(state: &ProtoOperationState) -> crate::models::ApplicationOperationState {
        crate::models::ApplicationOperationState {
            phase: state.phase.clone().unwrap_or_default(),
            message: state.message.clone().unwrap_or_default(),
            retry_count: state.retry_count,
            started_at: state.started_at.as_ref().map(Self::map_time),
            finished_at: state.finished_at.as_ref().map(Self::map_time),
            operation: state.operation.as_ref().map(Self::map_operation),
        }
    }

    fn map_operation(operation: &ProtoOperation) -> crate::models::ApplicationOperation {
        crate::models::ApplicationOperation {
            initiated_by: operation.initiated_by.as_ref().map(Self::map_initiator),
            info: operation.info.iter().map(Self::map_info).collect(),
            retry_limit: operation.retry.as_ref().and_then(|retry| retry.limit),
            retry_refresh: operation.retry.as_ref().and_then(|retry| retry.refresh),
        }
    }

    fn map_initiator(initiator: &ProtoOperationInitiator) -> crate::models::OperationInitiator {
        crate::models::OperationInitiator {
            username: initiator.username.clone().unwrap_or_default(),
            automated: initiator.automated.unwrap_or(false),
        }
    }

    fn map_info(info: &ProtoInfo) -> crate::models::ApplicationInfo {
        crate::models::ApplicationInfo {
            name: info.name.clone().unwrap_or_default(),
            value: info.value.clone().unwrap_or_default(),
        }
    }

    fn map_condition(condition: &ProtoApplicationCondition) -> crate::models::ApplicationCondition {
        crate::models::ApplicationCondition {
            r#type: condition.r#type.clone().unwrap_or_default(),
            message: condition.message.clone().unwrap_or_default(),
            last_transition_time: condition.last_transition_time.as_ref().map(Self::map_time),
        }
    }

    fn map_time(time: &ProtoTime) -> crate::models::KubeTime {
        crate::models::KubeTime {
            seconds: time.seconds,
            nanos: time.nanos,
        }
    }

    fn map_health(status: Option<&ProtoApplicationStatus>) -> crate::models::HealthStatus {
        match status.and_then(|s| s.health.as_ref()).and_then(|h| h.status.as_deref()) {
            Some("Healthy") => crate::models::HealthStatus::Healthy,
            Some("Degraded") => crate::models::HealthStatus::Degraded,
            Some("Progressing") => crate::models::HealthStatus::Progressing,
            Some("Suspended") => crate::models::HealthStatus::Suspended,
            Some("Missing") => crate::models::HealthStatus::Missing,
            _ => crate::models::HealthStatus::Unknown,
        }
    }

    fn map_sync(status: Option<&ProtoApplicationStatus>) -> crate::models::SyncStatus {
        match status.and_then(|s| s.sync.as_ref()).and_then(|s| s.status.as_deref()) {
            Some("Synced") => crate::models::SyncStatus::Synced,
            Some("OutOfSync") => crate::models::SyncStatus::OutOfSync,
            _ => crate::models::SyncStatus::Unknown,
        }
    }
}
