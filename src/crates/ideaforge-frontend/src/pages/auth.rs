use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use web_sys::HtmlInputElement;

use crate::api;
use crate::state::auth::AuthState;

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let error = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let navigate = use_navigate();

    let email_ref = NodeRef::<leptos::html::Input>::new();
    let password_ref = NodeRef::<leptos::html::Input>::new();

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if loading.get_untracked() {
            return;
        }

        let email = email_ref.get().map(|el| {
            let el: &HtmlInputElement = &el;
            el.value()
        }).unwrap_or_default();
        let password = password_ref.get().map(|el| {
            let el: &HtmlInputElement = &el;
            el.value()
        }).unwrap_or_default();

        if email.is_empty() || password.is_empty() {
            error.set("Email and password are required".into());
            return;
        }

        loading.set(true);
        error.set(String::new());

        let navigate = navigate.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api::auth::login(email, password).await {
                Ok(resp) => {
                    auth.set_from_token_response(&resp.user_id);
                    auth.load_user().await;
                    navigate("/", Default::default());
                }
                Err(e) => {
                    error.set(e.message);
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <h1 class="auth-title">"Welcome Back"</h1>
                <p class="auth-subtitle">"Sign in to your forge"</p>

                {move || {
                    let err = error.get();
                    if err.is_empty() {
                        view! { <div></div> }.into_any()
                    } else {
                        view! { <div class="form-error mb-md">{err}</div> }.into_any()
                    }
                }}

                <form on:submit=on_submit>
                    <div class="form-group">
                        <label class="form-label" for="email">"Email"</label>
                        <input
                            node_ref=email_ref
                            class="form-input"
                            type="email"
                            id="email"
                            placeholder="you@example.com"
                            required
                        />
                    </div>
                    <div class="form-group">
                        <label class="form-label" for="password">"Password"</label>
                        <input
                            node_ref=password_ref
                            class="form-input"
                            type="password"
                            id="password"
                            placeholder="Enter your password"
                            required
                        />
                    </div>
                    <button
                        class="btn btn-primary btn-lg"
                        style="width: 100%"
                        type="submit"
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Signing in..." } else { "Sign In" }}
                    </button>
                </form>

                <p class="auth-footer">
                    "New to the Forge? "
                    <A href="/register">"Create an account"</A>
                </p>
            </div>
        </div>
    }
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let error = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let navigate = use_navigate();

    let email_ref = NodeRef::<leptos::html::Input>::new();
    let password_ref = NodeRef::<leptos::html::Input>::new();
    let name_ref = NodeRef::<leptos::html::Input>::new();
    let role_ref = NodeRef::<leptos::html::Select>::new();

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if loading.get_untracked() {
            return;
        }

        let email = email_ref.get().map(|el| {
            let el: &HtmlInputElement = &el;
            el.value()
        }).unwrap_or_default();
        let password = password_ref.get().map(|el| {
            let el: &HtmlInputElement = &el;
            el.value()
        }).unwrap_or_default();
        let display_name = name_ref.get().map(|el| {
            let el: &HtmlInputElement = &el;
            el.value()
        }).unwrap_or_default();
        let role = role_ref.get().map(|el| {
            let el: &web_sys::HtmlSelectElement = &el;
            el.value()
        }).unwrap_or_default();

        if email.is_empty() || password.is_empty() || display_name.is_empty() {
            error.set("All fields are required".into());
            return;
        }

        loading.set(true);
        error.set(String::new());

        let role_opt = if role.is_empty() { None } else { Some(role) };

        let navigate = navigate.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api::auth::register(email, password, display_name, role_opt).await {
                Ok(resp) => {
                    auth.set_from_token_response(&resp.user_id);
                    auth.load_user().await;
                    navigate("/", Default::default());
                }
                Err(e) => {
                    error.set(e.message);
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <h1 class="auth-title">"Join the Forge"</h1>
                <p class="auth-subtitle">"Every idea deserves a forge"</p>

                {move || {
                    let err = error.get();
                    if err.is_empty() {
                        view! { <div></div> }.into_any()
                    } else {
                        view! { <div class="form-error mb-md">{err}</div> }.into_any()
                    }
                }}

                <form on:submit=on_submit>
                    <div class="form-group">
                        <label class="form-label" for="name">"Display Name"</label>
                        <input
                            node_ref=name_ref
                            class="form-input"
                            type="text"
                            id="name"
                            placeholder="Your name"
                            required
                        />
                    </div>
                    <div class="form-group">
                        <label class="form-label" for="reg-email">"Email"</label>
                        <input
                            node_ref=email_ref
                            class="form-input"
                            type="email"
                            id="reg-email"
                            placeholder="you@example.com"
                            required
                        />
                    </div>
                    <div class="form-group">
                        <label class="form-label" for="reg-password">"Password"</label>
                        <input
                            node_ref=password_ref
                            class="form-input"
                            type="password"
                            id="reg-password"
                            placeholder="Min 8 chars, uppercase, lowercase, digit"
                            required
                        />
                        <span class="form-help">"Must contain uppercase, lowercase, and a digit"</span>
                    </div>
                    <div class="form-group">
                        <label class="form-label" for="role">"I am a..."</label>
                        <select node_ref=role_ref class="form-select" id="role">
                            <option value="curious">"Curious Explorer"</option>
                            <option value="entrepreneur">"Entrepreneur"</option>
                            <option value="maker">"Maker"</option>
                        </select>
                    </div>
                    <button
                        class="btn btn-primary btn-lg"
                        style="width: 100%"
                        type="submit"
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Creating account..." } else { "Create Account" }}
                    </button>
                </form>

                <p class="auth-footer">
                    "Already have an account? "
                    <A href="/login">"Sign in"</A>
                </p>
            </div>
        </div>
    }
}
