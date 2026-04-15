//! Modal cropper shown before an avatar upload. The user pans and zooms the
//! source image inside a fixed-size square stage with a circular mask; on
//! confirm the visible square is rendered to a 512×512 offscreen canvas and
//! encoded as JPEG at 0.85 quality — that Blob is what gets uploaded, so the
//! server never sees the original 10MP phone photo.

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Blob, CanvasRenderingContext2d, HtmlCanvasElement};

const STAGE_SIZE: f64 = 320.0;
const OUTPUT_SIZE: u32 = 512;
const JPEG_QUALITY: f64 = 0.85;
const MAX_ZOOM: f64 = 5.0;

#[component]
pub fn AvatarCropper(
    file: web_sys::File,
    #[prop(into)] on_confirm: Callback<web_sys::Blob>,
    #[prop(into)] on_cancel: Callback<()>,
) -> impl IntoView {
    // Object URL points at the user-selected File. Kept in a StoredValue so we
    // can revoke it when the component unmounts.
    let object_url = StoredValue::new(
        web_sys::Url::create_object_url_with_blob(&file).unwrap_or_default(),
    );

    let tx = RwSignal::new(0.0_f64);
    let ty = RwSignal::new(0.0_f64);
    let zoom = RwSignal::new(1.0_f64);
    let min_zoom = RwSignal::new(1.0_f64);
    let nat_w = RwSignal::new(0.0_f64);
    let nat_h = RwSignal::new(0.0_f64);

    let dragging = RwSignal::new(false);
    let drag_from = RwSignal::new((0.0_f64, 0.0_f64));
    let drag_start_offset = RwSignal::new((0.0_f64, 0.0_f64));

    let img_ref = NodeRef::<leptos::html::Img>::new();
    let saving = RwSignal::new(false);
    let error = RwSignal::new(String::new());

    // Revoke the object URL on unmount so long sessions don't leak blobs.
    on_cleanup(move || {
        let url = object_url.get_value();
        if !url.is_empty() {
            let _ = web_sys::Url::revoke_object_url(&url);
        }
    });

    // When the <img> finishes loading we have natural dimensions — pick a zoom
    // that guarantees the image covers the crop box on both axes so the user
    // can't position an empty corner under the circle.
    let on_img_load = move |_ev: web_sys::Event| {
        let Some(img) = img_ref.get() else {
            return;
        };
        let w = img.natural_width() as f64;
        let h = img.natural_height() as f64;
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        nat_w.set(w);
        nat_h.set(h);
        let zmin = STAGE_SIZE / w.min(h);
        min_zoom.set(zmin);
        zoom.set(zmin);
        tx.set(0.0);
        ty.set(0.0);
    };

    let on_pointer_down = move |ev: web_sys::PointerEvent| {
        ev.prevent_default();
        dragging.set(true);
        drag_from.set((ev.client_x() as f64, ev.client_y() as f64));
        drag_start_offset.set((tx.get_untracked(), ty.get_untracked()));
        if let Some(target) = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        {
            let _ = target.set_pointer_capture(ev.pointer_id());
        }
    };

    let on_pointer_move = move |ev: web_sys::PointerEvent| {
        if !dragging.get_untracked() {
            return;
        }
        let (from_x, from_y) = drag_from.get_untracked();
        let (sx, sy) = drag_start_offset.get_untracked();
        let new_tx = sx + (ev.client_x() as f64 - from_x);
        let new_ty = sy + (ev.client_y() as f64 - from_y);
        let (cx, cy) = clamp_pan(new_tx, new_ty, zoom.get_untracked(), nat_w.get_untracked(), nat_h.get_untracked());
        tx.set(cx);
        ty.set(cy);
    };

    let on_pointer_up = move |_ev: web_sys::PointerEvent| {
        dragging.set(false);
    };

    let on_zoom_input = move |ev: web_sys::Event| {
        if let Ok(v) = event_target_value(&ev).parse::<f64>() {
            zoom.set(v);
            // Re-clamp pan: tightening zoom can push the image off the edge.
            let (cx, cy) = clamp_pan(
                tx.get_untracked(),
                ty.get_untracked(),
                v,
                nat_w.get_untracked(),
                nat_h.get_untracked(),
            );
            tx.set(cx);
            ty.set(cy);
        }
    };

    let cancel = move |_: web_sys::MouseEvent| {
        on_cancel.run(());
    };

    let on_confirm_click = move |_: web_sys::MouseEvent| {
        if saving.get_untracked() {
            return;
        }
        let Some(img) = img_ref.get() else {
            return;
        };
        let w = nat_w.get_untracked();
        let h = nat_h.get_untracked();
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let z = zoom.get_untracked();
        let cur_tx = tx.get_untracked();
        let cur_ty = ty.get_untracked();

        let (sx, sy, sw, sh) = compute_source_rect(cur_tx, cur_ty, z, w, h);

        // Offscreen canvas: we don't attach it to the DOM, it exists only for
        // the purpose of producing the JPEG blob.
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
            error.set("No document available.".into());
            return;
        };
        let canvas = match doc
            .create_element("canvas")
            .ok()
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
        {
            Some(c) => c,
            None => {
                error.set("Couldn't create canvas.".into());
                return;
            }
        };
        canvas.set_width(OUTPUT_SIZE);
        canvas.set_height(OUTPUT_SIZE);
        let ctx = match canvas.get_context("2d").ok().flatten().and_then(|c| {
            c.dyn_into::<CanvasRenderingContext2d>().ok()
        }) {
            Some(c) => c,
            None => {
                error.set("Couldn't get 2D context.".into());
                return;
            }
        };

        // JPEG can't encode alpha — fill white first so transparent PNG inputs
        // don't end up with black backgrounds.
        ctx.set_fill_style_str("#ffffff");
        ctx.fill_rect(0.0, 0.0, OUTPUT_SIZE as f64, OUTPUT_SIZE as f64);
        let dest = OUTPUT_SIZE as f64;
        if let Err(e) = ctx
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &img, sx, sy, sw, sh, 0.0, 0.0, dest, dest,
            )
        {
            error.set(format!("Draw failed: {e:?}"));
            return;
        }

        saving.set(true);
        spawn_local(async move {
            match canvas_to_jpeg_blob(&canvas).await {
                Ok(blob) => on_confirm.run(blob),
                Err(e) => {
                    saving.set(false);
                    error.set(format!("Couldn't encode: {e:?}"));
                }
            }
        });
    };

    view! {
        <div class="cropper-backdrop" role="dialog" aria-modal="true">
            <div class="cropper-panel">
                <h3 class="cropper-title">"Position your photo"</h3>
                <p class="cropper-hint">"Drag to reposition, use the slider to zoom. The circle is what everyone will see."</p>

                <div
                    class="cropper-stage"
                    on:pointerdown=on_pointer_down
                    on:pointermove=on_pointer_move
                    on:pointerup=on_pointer_up
                    on:pointercancel=on_pointer_up
                >
                    <img
                        node_ref=img_ref
                        class="cropper-img"
                        src=move || object_url.get_value()
                        on:load=on_img_load
                        draggable="false"
                        alt=""
                        style=move || {
                            let z = zoom.get();
                            let x = tx.get();
                            let y = ty.get();
                            format!("transform: translate(-50%, -50%) translate({x}px, {y}px) scale({z});")
                        }
                    />
                    <div class="cropper-overlay"></div>
                </div>

                <label class="cropper-zoom">
                    <span>"Zoom"</span>
                    <input
                        type="range"
                        min=move || format!("{:.4}", min_zoom.get())
                        max=move || format!("{:.2}", MAX_ZOOM)
                        step="0.01"
                        prop:value=move || format!("{:.4}", zoom.get())
                        on:input=on_zoom_input
                    />
                </label>

                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! { <p class="cropper-error">{e}</p> })
                }}

                <div class="cropper-buttons">
                    <button type="button" class="btn btn-ghost" on:click=cancel>"Cancel"</button>
                    <button
                        type="button"
                        class="btn btn-primary"
                        on:click=on_confirm_click
                        disabled=move || saving.get()
                    >
                        {move || if saving.get() { "Saving…" } else { "Use this photo" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Given the user's pan offsets and zoom, figure out which rectangle of the
/// source image is currently covering the crop square.
fn compute_source_rect(cur_tx: f64, cur_ty: f64, z: f64, w: f64, h: f64) -> (f64, f64, f64, f64) {
    let sw = STAGE_SIZE / z;
    let sh = STAGE_SIZE / z;
    let box_center = STAGE_SIZE / 2.0;
    let disp_w = w * z;
    let disp_h = h * z;
    let img_tl_x = box_center + cur_tx - disp_w / 2.0;
    let img_tl_y = box_center + cur_ty - disp_h / 2.0;
    let mut sx = -img_tl_x / z;
    let mut sy = -img_tl_y / z;
    sx = sx.clamp(0.0, (w - sw).max(0.0));
    sy = sy.clamp(0.0, (h - sh).max(0.0));
    (sx, sy, sw, sh)
}

/// Clamp the pan so the image never exposes a gap inside the crop square.
fn clamp_pan(tx: f64, ty: f64, z: f64, w: f64, h: f64) -> (f64, f64) {
    if w <= 0.0 || h <= 0.0 || z <= 0.0 {
        return (tx, ty);
    }
    let disp_w = w * z;
    let disp_h = h * z;
    let max_x = ((disp_w - STAGE_SIZE) / 2.0).max(0.0);
    let max_y = ((disp_h - STAGE_SIZE) / 2.0).max(0.0);
    (tx.clamp(-max_x, max_x), ty.clamp(-max_y, max_y))
}

async fn canvas_to_jpeg_blob(canvas: &HtmlCanvasElement) -> Result<Blob, JsValue> {
    let canvas = canvas.clone();
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let reject_clone = reject.clone();
        let cb = Closure::once_into_js(move |blob: JsValue| {
            if blob.is_null() || blob.is_undefined() {
                let _ = reject_clone.call1(&JsValue::NULL, &JsValue::from_str("toBlob returned null"));
            } else {
                let _ = resolve.call1(&JsValue::NULL, &blob);
            }
        });
        if let Err(e) = canvas.to_blob_with_type_and_encoder_options(
            cb.unchecked_ref::<js_sys::Function>(),
            "image/jpeg",
            &JsValue::from_f64(JPEG_QUALITY),
        ) {
            let _ = reject.call1(&JsValue::NULL, &e);
        }
    });
    let value = JsFuture::from(promise).await?;
    value
        .dyn_into::<Blob>()
        .map_err(|_| JsValue::from_str("canvas.toBlob returned non-blob"))
}
