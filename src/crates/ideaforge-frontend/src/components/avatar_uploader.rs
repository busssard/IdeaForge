use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::api;
use crate::components::avatar_cropper::AvatarCropper;

/// Circular avatar preview + "Change" button. File picker is hidden; the
/// visible button triggers it. On pick we open a cropper modal; the cropper's
/// output (a 512x512 JPEG Blob) is what actually gets uploaded.
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

    // When the user picks a file we stash it here and the cropper modal opens.
    // Uses LocalStorage because `web_sys::File` isn't `Send`/`Sync`.
    let pending_file = RwSignal::<Option<web_sys::File>, _>::new_local(None);

    let on_change = move |ev: web_sys::Event| {
        let Some(target) = ev.target() else {
            return;
        };
        let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
            return;
        };
        let Some(files) = input.files() else { return };
        let Some(file) = files.get(0) else { return };
        error.set(String::new());
        pending_file.set(Some(file));
    };

    let open_picker = move |_: web_sys::MouseEvent| {
        if let Some(input) = input_ref.get() {
            // Clear previous selection so picking the same file re-opens the
            // cropper (otherwise the `change` event doesn't fire).
            input.set_value("");
            input.click();
        }
    };

    let upload_blob = move |blob: web_sys::Blob| {
        error.set(String::new());
        uploading.set(true);
        pending_file.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            match api::users::upload_avatar(&blob).await {
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

    let cancel_crop = move |_: ()| {
        pending_file.set(None);
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
                <span class="avatar-uploader-hint">"PNG, JPEG, WebP or GIF. Any size — we'll crop and compress it."</span>
                {move || {
                    let msg = error.get();
                    (!msg.is_empty()).then(|| view! {
                        <span class="form-error">{msg}</span>
                    })
                }}
            </div>

            {move || pending_file.get().map(|file| view! {
                <AvatarCropper
                    file=file
                    on_confirm=Callback::new(move |blob: web_sys::Blob| { upload_blob(blob); })
                    on_cancel=Callback::new(cancel_crop)
                />
            })}
        </div>
    }
}
