use teaql_registry::webhook::{WebhookEventPayload, WebhookService};

#[tokio::test]
async fn test_webhook_subscription_lifecycle() {
    let sub = WebhookService::register(
        "https://example.com/webhook",
        vec!["artifact.published".to_string()],
        Some("super-secret-key".to_string()),
    );

    assert_eq!(sub.target_url, "https://example.com/webhook");
    assert!(sub.enabled);

    let list = WebhookService::list();
    assert!(list.iter().any(|s| s.id == sub.id));

    // Dispatch event (dispatches non-blocking async task)
    let payload = WebhookEventPayload {
        event_id: "evt-001".to_string(),
        event_type: "artifact.published".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        repository: "maven-releases".to_string(),
        format: "maven2".to_string(),
        path: Some("/com/example/lib/1.0/lib-1.0.jar".to_string()),
        component_name: Some("lib".to_string()),
        version: Some("1.0".to_string()),
    };

    WebhookService::dispatch(payload).await;

    // Unregister
    let removed = WebhookService::unregister(&sub.id);
    assert!(removed);
}
