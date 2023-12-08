use base64::Engine;
use leptos::create_action;
use leptos::Action;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::PushManager;
use web_sys::PushSubscription;
use web_sys::ServiceWorkerRegistration;

static PUSH_SERVER_PUBLIC_KEY: &'static str = env!("PUSH_SERVER_PUBLIC_KEY");

// Main function to start the app
// #[wasm_bindgen(start)]
// pub async fn main() -> Result<(), JsValue> {
//     let window = web_sys::window().expect("no global `window` exists");
//     let navigator = window.navigator();

//     if !navigator.service_worker().is_supported() || !PushManager::is_supported() {
//         return Err(JsValue::from_str(
//             "Service workers or Push API not supported",
//         ));
//     }

//     let sw_container: ServiceWorkerContainer = navigator.service_worker();

//     // Register service worker
//     let promise = sw_container.register("/service-worker.js");
//     let sw_registration: web_sys::ServiceWorkerRegistration =
//         wasm_bindgen_futures::JsFuture::from(promise)
//             .await?
//             .dyn_into()?;

//     // Request permission and handle push subscription
//     initialise_push_notifications(sw_registration).await?;

//     Ok(())
// }

/// A leptos action which asynchronously creates or updates and than retrieves the ServiceWorkerRegistration.
pub fn create_action_create_or_update_subscription(
) -> Action<PushManager, Result<PushSubscription, JsValue>> {
    create_action(move |push_manager: &PushManager| {
        let pm = push_manager.clone();
        async move {
            let key = key_conversion(PUSH_SERVER_PUBLIC_KEY)
                .map(|k| JsValue::from(k))
                .map_err(|_| JsValue::from_str("Could not convert public key"))?;

            let mut options = web_sys::PushSubscriptionOptionsInit::new();
            options.user_visible_only(true);
            options.application_server_key(Some(&key));

            let subscribe_promise = pm.subscribe_with_options(&options)?;
            wasm_bindgen_futures::JsFuture::from(subscribe_promise)
                .await
                .and_then(|ok| ok.dyn_into::<PushSubscription>())
        }
    })
}


pub fn create_action_see_subscription(
) -> Action<PushManager, Result<PushSubscription, JsValue>> {
    create_action(move |push_manager: &PushManager| {
        let pm = push_manager.clone();
        async move {
            let subscribe_promise = pm.get_subscription()?;
            wasm_bindgen_futures::JsFuture::from(subscribe_promise)
                .await
                .and_then(|ok| ok.dyn_into::<PushSubscription>())
        }
    })
}

fn key_conversion(key: &str) -> Result<js_sys::Uint8Array, base64::DecodeError> {
    let sk = {
        let e = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        e.decode(key).unwrap()
    };
    let uia = js_sys::Uint8Array::new_with_length(sk.len() as u32);
    uia.copy_from(&sk);
    Ok(uia)
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
/// The permission to send notifications
pub enum NotificationPermission {
    /// Notification has not been requested. In effect this is the same as `Denied`.
    #[default]
    Default,
    /// You are allowed to send notifications
    Granted,
    /// You are *not* allowed to send notifications
    Denied,
}

impl From<web_sys::NotificationPermission> for NotificationPermission {
    fn from(permission: web_sys::NotificationPermission) -> Self {
        match permission {
            web_sys::NotificationPermission::Default => Self::Default,
            web_sys::NotificationPermission::Granted => Self::Granted,
            web_sys::NotificationPermission::Denied => Self::Denied,
            web_sys::NotificationPermission::__Nonexhaustive => Self::Default,
        }
    }
}

/// Use `window.Notification.requestPosition()`. Returns a future that should be awaited
/// at least once before using [`use_web_notification`] to make sure
/// you have the permission to send notifications.
pub async fn request_web_notification_permission() -> NotificationPermission {
    if let Ok(notification_permission) = web_sys::Notification::request_permission() {
        let _ = wasm_bindgen_futures::JsFuture::from(notification_permission).await;
    }

    web_sys::Notification::permission().into()
}
// if let Some(navigator) = use_window().navigator() {
//     let promise = navigator.service_worker().register(script_url.as_str());
//     wasm_bindgen_futures::JsFuture::from(promise)
//         .await
//         .and_then(|ok| ok.dyn_into::<ServiceWorkerRegistration>())
// } else {
//     Err(JsValue::from_str("no navigator"))
// }
// /// A leptos action which asynchronously creates or updates and than retrieves the ServiceWorkerRegistration.
// fn create_action_create_or_update_registration(
// ) -> Action<ServiceWorkerScriptUrl, Result<ServiceWorkerRegistration, JsValue>> {
//     create_action(move |script_url: &ServiceWorkerScriptUrl| {
//         let script_url = script_url.0.to_owned();
//         async move {
//             if let Some(navigator) = use_window().navigator() {
//                 let promise = navigator.service_worker().register(script_url.as_str());
//                 wasm_bindgen_futures::JsFuture::from(promise)
//                     .await
//                     .and_then(|ok| ok.dyn_into::<ServiceWorkerRegistration>())
//             } else {
//                 Err(JsValue::from_str("no navigator"))
//             }
//         }
//     })
// }
