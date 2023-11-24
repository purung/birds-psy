use crate::error_template::{AppError, ErrorTemplate};
use chrono::{DateTime, Local};
use leptos::*;
use leptos_meta::*;
use leptos_query::provide_query_client;
use leptos_router::*;
use ulid::Ulid;

use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
mod db;

mod components;
use components::*;
mod please;
use please::*;
mod errors;
use errors::*;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Contact {
    stamp: Ulid,
    name: String,
    tel: String,
    special: String,
    timestamp: DateTime<Local>,
}

impl Contact {
    fn new(name: String, tel: String, special: String) -> Self {
        Self {
            name,
            tel,
            special,
            timestamp: Local::now(),
            stamp: Ulid::new(),
        }
    }
}

impl Default for Contact {
    fn default() -> Self {
        let name = "Inigo Montoya".to_owned();
        let tel = "070 666 666".to_owned();
        let special = "You killed my father. Prepare to die.".to_owned();
        Self::new(name, tel, special)
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    provide_query_client();

    view! {
        <Stylesheet id="leptos" href="/pkg/birds-psy.css"/>

        // sets the document title
        <Title text="Ovanifrånvy"/>
        <Link rel="preconnect" href="https://fonts.googleapis.com"/>
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="crossorigin"/>
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
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <main class="bg-primary h-[100svh] grid">
            <Transition fallback=move || view! { <div class="place-self-center loading loading-dots"></div> }>
                <ErrorBoundary fallback=move |_| view! {
                    <div role="alert" class="alert alert-error place-self-center">
                      <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                      <span>"Error! Task failed successfully."</span>
                    </div>
                     } >
                    <CardCollection /> 
                </ErrorBoundary>
            </Transition>
        </main>
    }
}
