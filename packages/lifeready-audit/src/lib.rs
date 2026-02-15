use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub created_at: String,
    pub actor_principal_id: String,
    pub action: String,
    pub tier: String,
    pub request_id: Option<String>,
    pub case_id: Option<String>,
    pub payload: Value,
}

impl AuditEvent {
    pub fn new(
        actor_principal_id: impl Into<String>,
        action: impl Into<String>,
        tier: impl Into<String>,
        request_id: Option<Uuid>,
        case_id: Option<String>,
        payload: Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            created_at: Utc::now().to_rfc3339(),
            actor_principal_id: actor_principal_id.into(),
            action: action.into(),
            tier: tier.into(),
            request_id: request_id.map(|value| value.to_string()),
            case_id,
            payload,
        }
    }
}

/// Result type for audit operations
pub type AuditResult<T> = Result<T, AuditError>;

/// Error type for audit operations
#[derive(Debug, Clone)]
pub struct AuditError {
    pub message: String,
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AuditError: {}", self.message)
    }
}

impl std::error::Error for AuditError {}

impl AuditError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Trait for audit event recording
///
/// This trait defines the interface for emitting audit events.
/// Services can use this to record auth decisions, access denied events,
/// and other auditable actions without direct network coupling.
pub trait AuditClient: Send + Sync {
    /// Record an audit event
    fn record(&self, event: AuditEvent) -> AuditResult<()>;

    /// Record an auth decision (access granted/denied)
    fn record_auth_decision(
        &self,
        actor_principal_id: &str,
        action: &str,
        resource: &str,
        granted: bool,
        reason: Option<&str>,
        request_id: Option<Uuid>,
    ) -> AuditResult<()> {
        let event = AuditEvent::new(
            actor_principal_id,
            if granted {
                format!("auth.granted:{}", action)
            } else {
                format!("auth.denied:{}", action)
            },
            "green",
            request_id,
            None,
            serde_json::json!({
                "resource": resource,
                "granted": granted,
                "reason": reason,
            }),
        );
        self.record(event)
    }

    /// Record an access denied event
    fn record_access_denied(
        &self,
        actor_principal_id: &str,
        resource: &str,
        reason: &str,
        request_id: Option<Uuid>,
    ) -> AuditResult<()> {
        self.record_auth_decision(
            actor_principal_id,
            "access",
            resource,
            false,
            Some(reason),
            request_id,
        )
    }
}

/// No-op implementation of AuditClient for use in tests or when audit is disabled
#[derive(Clone, Default)]
pub struct NoopAuditClient;

impl AuditClient for NoopAuditClient {
    fn record(&self, _event: AuditEvent) -> AuditResult<()> {
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct InMemoryAuditSink {
    events: Arc<Mutex<Vec<AuditEvent>>>,
}

impl InMemoryAuditSink {
    pub fn record(&self, event: AuditEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }

    pub fn snapshot(&self) -> Vec<AuditEvent> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }
}

impl AuditClient for InMemoryAuditSink {
    fn record(&self, event: AuditEvent) -> AuditResult<()> {
        InMemoryAuditSink::record(self, event);
        Ok(())
    }
}

/// Returns a 64-character string of zeros, used as the initial hash for audit chains.
pub fn zero_hash() -> String {
    "0".repeat(64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_audit_client_accepts_events() {
        let client = NoopAuditClient;
        let event = AuditEvent::new("actor", "action", "green", None, None, serde_json::json!({}));
        assert!(client.record(event).is_ok());
    }

    #[test]
    fn in_memory_sink_implements_audit_client() {
        let sink = InMemoryAuditSink::default();
        let client: &dyn AuditClient = &sink;
        let event = AuditEvent::new("actor", "action", "green", None, None, serde_json::json!({}));
        assert!(client.record(event).is_ok());
        assert_eq!(sink.snapshot().len(), 1);
    }

    #[test]
    fn record_auth_decision_sets_correct_action() {
        let sink = InMemoryAuditSink::default();
        sink.record_auth_decision("actor", "read", "/resource", true, None, None)
            .unwrap();
        let events = sink.snapshot();
        assert_eq!(events.len(), 1);
        assert!(events[0].action.starts_with("auth.granted:"));
    }

    #[test]
    fn record_access_denied_sets_reason() {
        let sink = InMemoryAuditSink::default();
        sink.record_access_denied("actor", "/resource", "insufficient_role", None)
            .unwrap();
        let events = sink.snapshot();
        assert_eq!(events.len(), 1);
        assert!(events[0].action.starts_with("auth.denied:"));
        let payload = &events[0].payload;
        assert_eq!(payload["reason"], "insufficient_role");
    }

    #[test]
    fn zero_hash_is_64_zeros() {
        let h = zero_hash();
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c == '0'));
    }
}
