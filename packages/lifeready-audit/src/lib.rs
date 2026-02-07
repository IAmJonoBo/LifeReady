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
