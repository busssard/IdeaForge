use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::api;

/// Circular avatar preview + "Change" button. File picker is hidden; the
/// visible button triggers it. Uploads on change, reports the new URL via
/// `on_uploaded`.
#[component]
pub fn AvatarUploader(
    /// The current avatar URL if the user already has one. Pass `None` when
    /// the user hasn't uploaded one yet.
    initial_url: Option<String>,
    /// The first letter of the display name, used as a fallback when no
    /// avatar is set.
    #[prop(into)]
    initial_letter: String,
    /// Fires with the new URL after a successful upload.
    #[prop(into)]
    on_uploaded: Callback<String>,
) -> impl IntoView {
    let current_url = RwSignal::new(initial_url);
    let uploading = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    let letter = StoredValue::new(initial_letter);
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let on_change = move |ev: web_sys::Event| {
        let Some(target) = ev.target() else { return; };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else { return; };
        let Some(files) = input.files() else { return; };
        let Some(file) = files.get(0) else { return; };

        error.set(String::new());
        uploading.set(true);

        wasm_bindgen_futures::spawn_local(async move {
            match api::users::upload_avatar(&file).await {
                Ok(resp) => {
                    current_url.set(Some(resp.avatar_url.clone()));
                    on_uploaded.run(resp.avatar_url);
                }
                Err(e) => {
                    error.set(e.message);
                }
            }
            uploading.set(false);
        });
    };

    let open_picker = move |_: web_sys::MouseEvent| {
        if let Some(input) = input_ref.get() {
            let _ = input.click();
        }
    };

    view! {
        <div class="avatar-uploader">
            <div class="avatar-uploader-preview">
                {move || {
                    match current_url.get() {
                        Some(url) => view! {
                            <img class="avatar-uploader-img" src=url alt="Avatar" />
                        }.into_any(),
                        None => view! {
                            <div class="avatar-uploader-letter">{letter.get_value()}</div>
                        }.into_any(),
                    }
                }}
            </div>

            <div class="avatar-uploader-controls">
                <input
                    node_ref=input_ref
                    type="file"
                    accept="image/png,image/jpeg,image/webp,image/gif"
                    class="avatar-uploader-input"
                    on:change=on_change
                />
                <button
                    type="button"
                    class="btn btn-ghost btn-sm"
                    on:click=open_picker
                    disabled=move || uploading.get()
                >
                    {move || {
                        if uploading.get() { "Uploading..." }
                        else if current_url.get().is_some() { "Change avatar" }
                        else { "Upload avatar" }
                    }}
                </button>
                <span class="avatar-uploader-hint">"PNG, JPEG, WebP or GIF. Max 5 MB."</span>
                {move || {
                    let msg = error.get();
                    (!msg.is_empty()).then(|| view! {
                        <span class="form-error">{msg}</span>
                    })
                }}
            </div>
        </div>
    }
}
