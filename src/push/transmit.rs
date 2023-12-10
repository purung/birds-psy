use std::env;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::error;

use web_push::SubscriptionInfo;
use web_push::WebPushClient;
use web_push::WebPushMessageBuilder;
use web_push::{IsahcWebPushClient, WebPushError, WebPushMessage};

use leptos::use_context;
use once_cell::sync::Lazy;
use web_push::{PartialVapidSignatureBuilder, VapidSignatureBuilder};

use crate::app::EyeError;

static KEY: Lazy<PartialVapidSignatureBuilder> = Lazy::new(|| {
    let config = web_push::URL_SAFE_NO_PAD;
    VapidSignatureBuilder::from_base64_no_sub(
        &env::var("PUSH_SERVER_PRIVATE_KEY").expect("Subscription key to be set"),
        config,
    )
    .expect("Push key to initialise")
});

async fn client() -> Option<PushClient> {
    use_context::<PushClient>()
}

pub async fn notify(sub: &SubscriptionInfo, msg: Option<String>) -> Result<(), WebPushError> {
    let client = client().await.ok_or(WebPushError::Other(
        "Could not aquire push client".to_owned(),
    ))?;
    let vapid = KEY.clone().add_sub_info(sub).build()?;
    let mut message = WebPushMessageBuilder::new(sub);
    message.set_vapid_signature(vapid);
    let bytes = msg.as_ref().map(String::as_bytes);
    if let Some(bts) = bytes {
        message.set_payload(web_push::ContentEncoding::Aes128Gcm, bts);
    };
    let message = message.build()?;
    client.send(message).await
}

#[derive(Clone)]
pub struct PushClient {
    inner: Arc<RwLock<IsahcWebPushClient>>,
}

impl PushClient {
    pub fn new() -> Result<Self, EyeError> {
        let client = IsahcWebPushClient::new().map_err(|_| EyeError::ConfigError)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(client)),
        })
    }
    async fn send(&self, msg: WebPushMessage) -> Result<(), WebPushError> {
        let lock = self.inner.read();
        match lock {
            Ok(c) => c.send(msg).await,
            Err(e) => {
                error!("The client could not be aquired for read: {:?}", e);
                Err(WebPushError::Other(
                    "Unable to aquire read lock on client".to_owned(),
                ))
            }
        }
    }
}
