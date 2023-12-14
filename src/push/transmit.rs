use futures::future::join_all;
use tracing::warn;
use std::env;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::task::spawn_local;
use tracing::error;
use tracing::info;

use web_push::SubscriptionInfo;
use web_push::WebPushClient;
use web_push::WebPushMessageBuilder;
use web_push::{IsahcWebPushClient, WebPushError, WebPushMessage};

use leptos::use_context;
use once_cell::sync::Lazy;
use web_push::{PartialVapidSignatureBuilder, VapidSignatureBuilder};

use crate::app::Communicate;
use crate::app::EyeError;
use crate::app::User;

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

pub async fn notify_all(
    subscribers: Vec<SubscriptionInfo>,
    msg: Option<String>,
) -> Result<(), WebPushError> {
    let mut futures = Vec::new();

    for sub in subscribers.clone() {
        let msg = msg.clone();
        let future = spawn_local(async move { notify(&sub, msg).await });
        futures.push(future)
    }
    let results = join_all(futures).await;

    let mut to_unsubscibe = Vec::new();

    for (sub, res) in subscribers.into_iter().zip(results) {
        match res {
            Ok(r) => match r {
                Ok(_) => continue,
                Err(e) => {
                    use WebPushError as WPE;
                    match e {
                        WPE::Unauthorized
                        | WPE::MissingCryptoKeys
                        | WPE::InvalidCryptoKeys
                        | WPE::InvalidUri
                        | WPE::EndpointNotValid
                        | WPE::EndpointNotFound => {
                            info!(">>>>> A problem with a subscription! Going to unsubscribe it... {}", e);
                            to_unsubscibe.push(sub);
                            continue;
                        }
                        _ => {
                            error!(">>>>>>>>>>>> When pushing a notification: {:?}", e);
                            continue;
                        }
                    }
                }
            },
            Err(e) => {
                error!(
                    ">>>> Something happened during join of push futures: {:?}",
                    e
                )
            }
        }
    }

    let mut future_unsubscribes = Vec::new();

    for sub in to_unsubscibe {
        let future =
            spawn_local(async move { <(SubscriptionInfo, User)>::destroy(sub.endpoint).await });
        future_unsubscribes.push(future)
    }

    let result = join_all(future_unsubscribes).await;

    for res in result {
        match res {
            Ok(r) => match r {
                Ok(_) => (),
                Err(e) => warn!(">>>>> Något gick fel med att ta bort prenumeration: {}", e),
            },
            Err(e) => error!(">>>>> Internt fel med att ta bort prenumeration: {}", e),
        }
    }

    Ok(())
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
