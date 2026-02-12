use lifeready_audit::{AuditEvent, InMemoryAuditSink};
use serde_json::json;
use uuid::Uuid;

#[test]
fn audit_event_sets_fields() {
    let request_id = Uuid::new_v4();
    let event = AuditEvent::new(
        "actor-1",
        "case.export",
        "green",
        Some(request_id),
        Some("case-1".to_string()),
        json!({"ok": true}),
    );

    assert_eq!(event.actor_principal_id, "actor-1");
    assert_eq!(event.action, "case.export");
    assert_eq!(event.tier, "green");
    let request_str = request_id.to_string();
    assert_eq!(event.request_id.as_deref(), Some(request_str.as_str()));
    assert_eq!(event.case_id.as_deref(), Some("case-1"));
    assert!(event.event_id.len() > 10);
    assert!(event.created_at.contains('T'));
}

#[test]
fn in_memory_sink_records_events() {
    let sink = InMemoryAuditSink::default();
    let event = AuditEvent::new(
        "actor-2",
        "audit.append",
        "amber",
        None,
        None,
        json!({"value": 1}),
    );
    sink.record(event);

    let snapshot = sink.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].action, "audit.append");
}
