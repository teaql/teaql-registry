use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use tracing::{error, info};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSubscription {
    pub id: String,
    pub target_url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEventPayload {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub repository: String,
    pub format: String,
    pub path: Option<String>,
    pub component_name: Option<String>,
    pub version: Option<String>,
}

static WEBHOOK_STORE: LazyLock<Arc<RwLock<HashMap<String, WebhookSubscription>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

pub struct WebhookService;

impl WebhookService {
    pub fn register(
        target_url: &str,
        events: Vec<String>,
        secret: Option<String>,
    ) -> WebhookSubscription {
        let sub = WebhookSubscription {
            id: Uuid::new_v4().to_string(),
            target_url: target_url.to_string(),
            events,
            secret,
            enabled: true,
        };

        let mut store = WEBHOOK_STORE.write().unwrap();
        store.insert(sub.id.clone(), sub.clone());
        sub
    }

    pub fn list() -> Vec<WebhookSubscription> {
        let store = WEBHOOK_STORE.read().unwrap();
        store.values().cloned().collect()
    }

    pub fn unregister(id: &str) -> bool {
        let mut store = WEBHOOK_STORE.write().unwrap();
        store.remove(id).is_some()
    }

    pub async fn dispatch(event: WebhookEventPayload) {
        let subscriptions = Self::list();
        if subscriptions.is_empty() {
            return;
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let json_body = match serde_json::to_string(&event) {
            Ok(j) => j,
            Err(_) => return,
        };

        for sub in subscriptions {
            if !sub.enabled {
                continue;
            }

            if !sub.events.iter().any(|e| e == "*" || e == &event.event_type) {
                continue;
            }

            let mut req = client
                .post(&sub.target_url)
                .header("Content-Type", "application/json")
                .header("User-Agent", "TeaQL-Registry-Webhook/1.0")
                .header("X-TeaQL-Event", &event.event_type)
                .header("X-TeaQL-Delivery", &event.event_id);

            if let Some(ref secret) = sub.secret {
                let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take any size");
                mac.update(json_body.as_bytes());
                let signature = hex::encode(mac.finalize().into_bytes());
                req = req.header("X-TeaQL-Signature", format!("sha256={}", signature));
            }

            let req_with_body = req.body(json_body.clone());
            tokio::spawn(async move {
                match req_with_body.send().await {
                    Ok(resp) => {
                        info!("Webhook delivered to {} -> Status {}", sub.target_url, resp.status());
                    }
                    Err(e) => {
                        error!("Failed to deliver webhook to {}: {}", sub.target_url, e);
                    }
                }
            });
        }
    }
}
