use crate::error_template::{AppError, ErrorTemplate};
use chrono::{DateTime, Local};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use strum::{AsRefStr, EnumIter, EnumString, IntoStaticStr};
use ulid::Ulid;

use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
pub mod db;

pub mod components;
pub use components::*;
pub mod please;
pub use please::*;
pub mod errors;
pub use errors::*;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Contact {
    stamp: Ulid,
    name: String,
    tel: String,
    special: String,
    timestamp: DateTime<Local>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MaybeUser {
    User(User),
    Rejected,
    None,
}

impl MaybeUser {
    pub fn is_logged_in(&self) -> Result<(), EyeError> {
        match self {
            MaybeUser::User(_) => Ok(()),
            _ => Err(EyeError::AuthError),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Eq,
    EnumString,
    IntoStaticStr,
    EnumIter,
    AsRefStr,
)]
#[strum(ascii_case_insensitive)]
pub enum User {
    Maria,
    Admin,
}

impl Contact {
    fn new(name: String, tel: String, special: Option<String>) -> Self {
        Self {
            name,
            tel,
            special: special.unwrap_or_else(|| "Inga speciella önskemål".to_owned()),
            timestamp: Local::now(),
            stamp: Ulid::new(),
        }
    }
    fn human_timestamp(&self) -> String {
        format!("{}", self.timestamp.format("%d %b %Y kl. %H:%M"))
    }
    fn tel_link(&self) -> String {
        format!("tel:{}", self.tel)
    }
}

impl Default for Contact {
    fn default() -> Self {
        let name = "Inigo Montoya".to_owned();
        let tel = "070 666 666".to_owned();
        let special = "You killed my father. Prepare to die.".to_owned();
        Self::new(name, tel, Some(special))
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/birds-psy.css"/>

        // sets the document title
        <Title text="Ovanifrånvy"/>
        <Link rel="preconnect" href="https://fonts.googleapis.com"/>
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="crossorigin"/>
        <Link rel="manifest" href="/manifest.json" />
        <Link
            href="https://fonts.googleapis.com/css2?family=Noto+Sans:ital,wght@0,300;0,400;0,700;1,400;1,700&display=swap"
            rel="stylesheet"
        />
        // content for this welcome page

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage  />
                    <Route path="/login" view=Login />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <main class="bg-base-100 relative min-h-[100svh] pb-12">
            <div class="absolute -top-10 right-2" >
                <Updates />
            </div>
            <Suspense fallback=move || view! {<p>"Loading..."</p>}>
                <UserLand />
            </Suspense>
        </main>

    }
}

/// Renders the home page of your application.
#[component]
fn UserLand() -> impl IntoView {
    let (logged_in, set_logged_in) = create_signal(false);
    let user = create_blocking_resource(move || logged_in(), |_| async move { current_user().await });
    let login_required = Signal::derive(move || {
        user.with(|u| u.as_ref().is_some_and(|r| r.is_err()))
    });
    view! {
        <Suspense fallback=move || view! {<p>"Loading..."</p>}>
            <Show when=login_required>
                <LoginPortal auth=set_logged_in />
            </Show>
        </Suspense>
        <Suspense>
            <CardCollection auth=logged_in />
        </Suspense>
    }
}
