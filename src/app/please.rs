use cfg_if::cfg_if;

cfg_if!( if #[cfg(feature = "ssr")] {

use once_cell::sync::Lazy;
use async_trait::async_trait;
use leptos_axum::redirect;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::info;

use super::errors::EyeError;
use crate::push::transmit;

static HOMEPAGE: Lazy<String> =
    Lazy::new(|| {
        std::env::var("HOMEPAGE")
        .expect("homepage var to be set in the environment")
    }
);

fn reject_strangers() -> Option<User> {
    let Some(MaybeUser::User(u)) = use_context::<MaybeUser>() else {
        redirect("/login");
        return None
    };
    Some(u)
}

#[async_trait]
pub trait Communicate<Subject, Dialect>
where
    Subject: Serialize + DeserializeOwned,
{
    async fn power() -> Result<Dialect, EyeError>;
    async fn birth(&self) -> Result<(), EyeError>;
    async fn destroy(id: String) -> Result<(), EyeError>;
    async fn all() -> Result<Vec<Subject>, EyeError>;
}

    }
);


use leptos::*;
use ulid::Ulid;

use super::{Contact, MaybeUser, User};

#[server(prefix = "/api", endpoint = "boka")]
pub async fn add_contact_request(
    name: String,
    tel: String,
    special: String,
) -> Result<(), ServerFnError> {
    Contact::new(name, tel, Some(special)).birth().await?;
    redirect(&format!("{}/succe", *HOMEPAGE));
    Ok(())
}

#[server]
pub async fn delete_contact_request(ulid: Ulid) -> Result<(), ServerFnError> {
    if reject_strangers().is_some() {
        Contact::destroy(ulid.to_string()).await?;
    };
    Ok(())
}

#[server]
pub async fn all_contact_requests() -> Result<Vec<Contact>, ServerFnError> {
    if reject_strangers().is_some() {
        Ok(Contact::all().await?)
    } else {
        Ok(Vec::new())
    }
}

#[server]
pub async fn log_me_in(user: String, password: String) -> Result<(), ServerFnError> {
    use crate::auth::*;
    confirm_login_for(user, password).await?;
    redirect("/");
    Ok(())
}

#[server]
pub async fn current_user() -> Result<User, ServerFnError> {
    Ok(reject_strangers().ok_or(EyeError::AuthError)?)
}

#[server]
pub async fn perhaps_user() -> Result<MaybeUser, ServerFnError> {
    use_context::<MaybeUser>().ok_or(EyeError::AuthError.into())
}

#[server(prefix = "/api", endpoint = "push-me")]
pub async fn subscribe_to_push(json: String) -> Result<(), ServerFnError> {
    if let Some(user) = reject_strangers() {
        use web_push::SubscriptionInfo;
        let info: SubscriptionInfo = serde_json::from_str(&json).unwrap();
        // let demo = notify(&info, Some("Working".to_owned())).await;
        // info!("Demo did: {:?}", &demo);
        (info, user).birth().await?;
        info!("Lagrat en ny prenumeration");
    }
    Ok(())
}

#[server(prefix = "/api", endpoint = "push-me-not")]
pub async fn unsubscribe_to_push(json: String) -> Result<(), ServerFnError> {
    if reject_strangers().is_some() {
        use web_push::SubscriptionInfo;
        let info: SubscriptionInfo = serde_json::from_str(&json).unwrap();
        <(SubscriptionInfo, User)>::destroy(info.endpoint).await?;
        info!("Tagit bort en prenumeration");
    }
    Ok(())
}

#[server]
pub async fn demo_push() -> Result<(), ServerFnError> {
    use web_push::SubscriptionInfo;
    let subs = <(SubscriptionInfo, User)>::all().await?;
    for (sub, _) in subs {
       transmit::notify(&sub, Some("Demo av notifikation".to_owned())).await?;
    }
    Ok(())
}
