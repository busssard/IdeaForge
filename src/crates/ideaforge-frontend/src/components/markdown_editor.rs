//! A textarea with a small formatting toolbar that inserts markdown syntax
//! at the current cursor position (or wraps the current selection).
//!
//! The result is the raw markdown source. Callers typically render it via
//! the `<Markdown/>` component on display.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Textarea + toolbar. `value` is a two-way bound RwSignal so the parent
/// owns the text and can read it at submit time.
#[component]
pub fn MarkdownEditor(
    /// Backing store for the text. The component reads and writes it.
    value: RwSignal<String>,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional)] rows: Option<i32>,
    #[prop(optional, into)] input_class: String,
) -> impl IntoView {
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();
    let placeholder_owned = placeholder;
    let input_cls = if input_class.is_empty() {
        "form-input md-editor-textarea".to_string()
    } else {
        format!("form-input md-editor-textarea {input_class}")
    };

    // Apply a formatting wrap (prefix/suffix) or insertion to the current
    // selection. If there is no selection, inserts a placeholder so the user
    // sees the pattern and can type over it.
    let apply = move |before: &'static str, after: &'static str, placeholder_text: &'static str| {
        let Some(ta) = textarea_ref.get() else {
            return;
        };
        let el: web_sys::HtmlTextAreaElement = ta.unchecked_into();
        let start = el.selection_start().unwrap_or(None).unwrap_or(0) as usize;
        let end = el.selection_end().unwrap_or(None).unwrap_or(0) as usize;
        let current = el.value();

        // Byte indices are tricky when the selection is in the middle of a
        // multi-byte UTF-8 glyph. `selection_start/end` report code units
        // (UTF-16 in the DOM). For our purposes — ASCII markdown chars —
        // the common case is fine; fall back to appending if the indices
        // look off.
        let (left, mid, right) = if start <= end && end <= current.len() {
            let (l, rest) = current.split_at(start);
            let (m, r) = rest.split_at(end - start);
            (l.to_string(), m.to_string(), r.to_string())
        } else {
            (current.clone(), String::new(), String::new())
        };

        let selected = if mid.is_empty() {
            placeholder_text
        } else {
            mid.as_str()
        };
        let new_val = format!("{left}{before}{selected}{after}{right}");
        value.set(new_val.clone());
        el.set_value(&new_val);

        // Re-select the inserted text so the user can keep typing to
        // replace it.
        let new_start = (left.len() + before.len()) as u32;
        let new_end = new_start + selected.len() as u32;
        let _ = el.set_selection_range(new_start, new_end);
        let _ = el.focus();
    };

    view! {
        <div class="md-editor">
            <div class="md-editor-toolbar" role="toolbar">
                <button
                    type="button"
                    class="md-toolbar-btn"
                    title="Bold (Ctrl+B)"
                    on:click=move |_| apply("**", "**", "bold")
                >"B"</button>
                <button
                    type="button"
                    class="md-toolbar-btn md-toolbar-italic"
                    title="Italic (Ctrl+I)"
                    on:click=move |_| apply("_", "_", "italic")
                >"I"</button>
                <button
                    type="button"
                    class="md-toolbar-btn"
                    title="Inline code"
                    on:click=move |_| apply("`", "`", "code")
                >"<>"</button>
                <button
                    type="button"
                    class="md-toolbar-btn"
                    title="Link"
                    on:click=move |_| apply("[", "](https://)", "link text")
                >"\u{1F517}"</button>
                <button
                    type="button"
                    class="md-toolbar-btn"
                    title="Image"
                    on:click=move |_| apply("![", "](https://)", "alt text")
                >"\u{1F5BC}"</button>
                <button
                    type="button"
                    class="md-toolbar-btn"
                    title="Bulleted list"
                    on:click=move |_| apply("\n- ", "", "item")
                >"\u{2022}"</button>
                <button
                    type="button"
                    class="md-toolbar-btn"
                    title="Code block"
                    on:click=move |_| apply("\n```\n", "\n```\n", "code")
                >"\u{007B}\u{007D}"</button>
                <button
                    type="button"
                    class="md-toolbar-btn"
                    title="Quote"
                    on:click=move |_| apply("\n> ", "", "quote")
                >"\u{201C}"</button>
                <span class="md-editor-hint">"Markdown supported"</span>
            </div>
            <textarea
                node_ref=textarea_ref
                class=input_cls
                rows=rows.unwrap_or(4)
                placeholder=placeholder_owned
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
            ></textarea>
        </div>
    }
}
