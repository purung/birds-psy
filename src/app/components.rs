use js_sys::JSON;

use leptos::spawn_local;
use leptos::*;
use leptos_icons::*;

// use leptos_animated_for::AnimatedFor;
use leptos_router::ActionForm;
use leptos_use::use_service_worker;
use leptos_use::use_supported;

use leptos_use::ServiceWorkerRegistrationError;
use leptos_use::UseServiceWorkerReturn;
use wasm_bindgen::JsValue;
use web_sys::ServiceWorkerRegistration;

use crate::app::all_contact_requests;
use crate::app::please::*;
use crate::push;

use super::Contact;

#[component]
pub fn Updates() -> impl IntoView {
    let the_moment_has_come = RwSignal::new(false);
    create_effect(move |_| the_moment_has_come.set(true));

    // show_class="opacity-100 duration-1000 transition transform -translate-y-20"
    // hide_class="transform transition translate-y-20  opacity-0"
    view! {
        <Show
            when=the_moment_has_come
        >
            <PushCompability />
        </Show>

    }
}

#[component]
pub fn PushCompability() -> impl IntoView {
    let supported = use_supported(move || JsValue::from("PushManager").js_in(&window()));
    let UseServiceWorkerReturn { registration, .. } = use_service_worker();
    leptos::logging::log!("Push is supported: {}", supported());
    view! {
        <Show when=supported >
            <AskAboutUpdates registration=registration />
        </Show>
    }
}

type SwReg = Signal<Result<ServiceWorkerRegistration, ServiceWorkerRegistrationError>>;

#[component]
pub fn AskAboutUpdates(registration: SwReg) -> impl IntoView {
    use crate::push::*;
    let permission = RwSignal::new(web_sys::Notification::permission().into());
    let subscribe = push::create_action_create_or_update_subscription();
    let unsubscribe = push::create_action_undo_subscription();
    let current_subscription = push::create_action_see_subscription();

    let push_enabled = Signal::derive(move || {
        current_subscription
            .value()
            .get()
            .is_some_and(|v| v.is_ok())
    });

    let refresh_current_subscription = move || {
        if let Ok(rw) = registration() {
            if let Ok(pm) = rw.push_manager() {
                leptos::logging::log!("Inspekterar nuvarande pushprenumeration");
                current_subscription.dispatch(pm);
            }
        }
    };

    let toggle_push = move |_| {
        if push_enabled() {
            // Det finns redan en push-prenumeration
            if let Some(Ok(sub)) = current_subscription.value().get() {
                leptos::logging::log!("Avslutar push");
                unsubscribe.dispatch(sub.clone()); // Ta bort i browsern
                let json = sub.to_json().unwrap();
                let json = JSON::stringify(&json).unwrap();
                let json = json.as_string().unwrap();
                spawn_local(async move {
                    let _ = unsubscribe_to_push(json).await; // Ta bort på servern
                });
            }
        } else {
            // Det finns inte en push-prenumeration
            // Men det finns tillåtelse att skicka notifikationer
            leptos::logging::log!("Startar push");
            if NotificationPermission::Granted == permission() {
                if let Ok(rw) = registration() {
                    if let Ok(pm) = rw.push_manager() {
                        subscribe.dispatch(pm);
                    }
                }
            } else {
                // Be om tillåtelse att skicka notifikationer
                spawn_local(async move {
                    permission.set(request_web_notification_permission().await);
                })
            }
        }
    };

    // Centralisera förändringar i prenumeration till current
    create_effect(move |_| {
        subscribe.value().get();
        unsubscribe.value().get();
        refresh_current_subscription();
    });

    create_effect(move |prev_permission| {
        leptos::logging::log!("Reagerar på förändring av permission");
        let new_permission = permission.get();
        let allowed = new_permission == NotificationPermission::Granted;
        let from_previously_disallowed = new_permission != prev_permission.unwrap_or_default();
        let not_subscribed = current_subscription
            .value()
            .get_untracked()
            .is_some_and(|s| s.is_err());

        if allowed && from_previously_disallowed && not_subscribed {
            let Ok(Ok(pm)) = registration.get_untracked().map(|rg| rg.push_manager()) else {
                return new_permission;
            };
            // Startar en ny prenumeration
            leptos::logging::log!("Startar en ny prenumeration");
            subscribe.dispatch(pm);
        }

        new_permission
    });

    // Reagerar på förändringar i prenumeration
    create_effect(move |_| {
        if let Some(Ok(subscription)) = subscribe.value().get() {
            leptos::logging::log!("Lagrar automatiskt en ny prenumeration!");
            let json = subscription.to_json().unwrap();
            let json = JSON::stringify(&json).unwrap();
            let json = json.as_string().unwrap();
            spawn_local(async move {
                let _ = subscribe_to_push(json).await;
            });
        }
    });

    view! {
        <button on:click=toggle_push type="button" class="btn btn-ghost text-primary btn-circle hover:text-accent">
            <Show when=push_enabled >
                <Icon icon=Icon::from(IoIcon::IoNotificationsCircle) class="w-full h-full" />
            </Show>
            <Show when=move||!push_enabled() >
                <Icon icon=Icon::from(IoIcon::IoNotificationsOffCircle) class="w-full h-full" />
            </Show>
        </button>
    }
}

#[component]
pub fn Login() -> impl IntoView {
    let gogogo = create_server_action::<LogMeIn>();
    let error = Signal::derive(move || match gogogo.value().get() {
        Some(r) => match r {
            Ok(_) => None,
            Err(e) => e.to_string().split_once(": ").map(|s| s.1.to_owned()),
        },
        None => None,
    });
    view! {
        <main class="bg-base-100 min-h-[100svh] grid">
            <div class="place-self-center px-8 py-12 w-[90vw] max-w-sm bg-primary rounded-lg shadow-lg">
                <ActionForm action=gogogo class="flex flex-col gap-8">
                    <h4 class="text-lg text-primary-content text-center font-bold">Logga in</h4>
                    <p class="text-center text-base-100">{ move || error.get() }</p>
                    <div class="form-control">
                        <label class="label text-primary-content">Namn</label>
                        <input
                            name="user"
                            inputmode="text"
                            autocomplete="username"
                            required
                            class="input input-bordered input-secondary w-full"
                        />
                    </div>
                    <div class="form-control">
                        <label class="label text-primary-content">Lösenord</label>
                        <input
                            name="password"
                            type="password"
                            autocomplete="current-password"
                            required
                            class="input input-bordered input-secondary w-full"
                        />
                    </div>
                    <div class="form-control grow">
                        <button
                            class="btn btn-secondary btn-outline text-accent hover:btn-active"
                            type="submit"
                            id="skickaKnapp"
                            >
                            <Show when=move || gogogo.pending().get() fallback=move || view! {  <span id="skicka">Logga in</span> }>
                                <span id="laddar" class="loading loading-dots loading-sm" > </span>
                            </Show>
                        </button>
                    </div>
                </ActionForm>
            </div>
        </main>
    }
}

#[component]
pub fn ContactCard(card: Contact, reversion: Callback<()>) -> impl IntoView {
    let dispose = create_server_action::<DeleteContactRequest>();
    create_effect(move |_| {
        dispose.version().get();
        leptos::Callable::call(&reversion, ());
    });
    let tel_link = card.tel_link();
    let human_ts = card.human_timestamp();
    view! {
        <div class="card w-96 max-w-full relative bg-primary text-primary-content rounded-lg shadow-lg self-stretch">
          <div class="card-body">
            <h2 class="card-title mt-6 mb-3">{ card.name }</h2>
            <p>{ card.tel }</p>
            <p class="text-xs absolute right-0 top-0 p-4 text-secondary">{ human_ts }</p>
            <p class="my-6">{ card.special }</p>
            <div class="card-actions mt-24 justify-between content-center">
        <a href=tel_link class="btn btn-ghost btn-circle text-base-100/60 hover:text-base-100">
        <Icon icon=Icon::from(IoIcon::IoCall) class="w-full h-full" />
        </a>

        <ActionForm action=dispose >
            <input type="hidden" name="ulid" value=move || card.stamp.to_string() />

            <button type="submit" class="btn btn-md btn-circle text-base-100/60 hover:text-base-100 btn-ghost">
                <Icon class="w-full h-full" icon=Icon::from(IoIcon::IoCheckmarkDoneCircleSharp) />
            </button>
        </ActionForm>
            </div>
          </div>
        </div>
    }
}

#[component]
pub fn CardCollection() -> impl IntoView {
    let r = create_resource(
        || (),
        |_| async move { all_contact_requests().await.unwrap_or_default() },
    );
    let cards = Signal::derive(move || r.get().unwrap_or_default());
    let reversion = Callback::new(move |()| r.refetch());
    view! {
        <Transition fallback=move || view! { <div class="place-self-center loading loading-dots text-base-100"></div> }>
            <ErrorBoundary fallback=move |_| view! {
                <div role="alert" class="alert alert-error place-self-center">
                  <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                  <span>"Error! Task failed successfully."</span>
                </div>
             } >

                <h1 class="text-3xl select-none text-center text-primary m-12 font-bold">Uppringningslista</h1>
                <div class="m-2 grid grid-cols-1 sm:grid-cols-2 w-fit xl:grid-cols-3 gap-4 max-w-screen-xl px-2 xl:px-0 mx-auto">
                     <For
                        each=cards
                        key=|card| card.stamp
                        children=move |card| view! { <ContactCard card reversion /> }
                    />
                </div>
            </ErrorBoundary>
        </Transition>
    }
}
