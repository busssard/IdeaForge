//! User-content markdown renderer. Wraps `pulldown-cmark` with two layers
//! of user-content safety:
//!
//! 1. **Raw-HTML events from the parser are dropped** — so if someone pastes
//!    `<script>`, `<iframe>`, `<img onerror=...>`, or similar, it never
//!    reaches the output HTML. Users get markdown syntax and nothing else.
//! 2. **URL schemes are restricted** — links must be `http:`, `https:`,
//!    `mailto:`, or relative. Images must be `http:` or `https:`. Anything
//!    else is replaced with a safe placeholder so `javascript:` /
//!    `data:text/html` XSS vectors can't reach the DOM.
//!
//! Images also pick up `referrerpolicy="no-referrer"` (so the image origin
//! doesn't see the reader's Referer) and `loading="lazy"`.

use leptos::prelude::*;
use pulldown_cmark::{CowStr, Event, LinkType, Options, Parser, Tag};

/// Render user-authored markdown into sanitized HTML suitable for display.
pub fn render(markdown: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(markdown, opts);

    // Filter events: drop raw HTML, sanitize link/image URLs.
    let cleaned: Vec<Event<'_>> = parser
        .filter_map(|ev| match ev {
            Event::Html(_) | Event::InlineHtml(_) => None,
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => Some(Event::Start(Tag::Link {
                link_type,
                dest_url: CowStr::Boxed(sanitize_link_url(&dest_url).into_boxed_str()),
                title,
                id,
            })),
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => Some(Event::Start(Tag::Image {
                link_type: if link_type == LinkType::Autolink {
                    LinkType::Inline
                } else {
                    link_type
                },
                dest_url: CowStr::Boxed(sanitize_image_url(&dest_url).into_boxed_str()),
                title,
                id,
            })),
            other => Some(other),
        })
        .collect();

    let mut html = String::with_capacity(markdown.len() * 2);
    pulldown_cmark::html::push_html(&mut html, cleaned.into_iter());

    // Post-process image tags to harden privacy + perf. Using a dumb string
    // replace is fine because pulldown-cmark always emits `<img ` with a
    // trailing space.
    html = html.replace(
        "<img ",
        "<img loading=\"lazy\" referrerpolicy=\"no-referrer\" ",
    );

    // External links open in a new tab — avoids dropping the user out of
    // the app.
    html = html.replace("<a href=\"http", "<a target=\"_blank\" rel=\"noopener noreferrer\" href=\"http");

    html
}

fn sanitize_link_url(url: &str) -> String {
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with('#')
        || !url.contains(':')
    {
        url.to_string()
    } else {
        "#".to_string()
    }
}

fn sanitize_image_url(url: &str) -> String {
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") || !url.contains(':') {
        url.to_string()
    } else {
        // Blank URL renders a broken-image icon; better than silently letting
        // a `javascript:` payload through.
        String::new()
    }
}

/// Drop-in component for rendering user-authored markdown. Pass the raw
/// source as `content`; the component does all the sanitization +
/// rendering.
#[component]
pub fn Markdown(
    /// Raw markdown source from the user.
    #[prop(into)]
    content: String,
    /// Optional wrapper class (e.g. "idea-detail-body", "chat-msg-text").
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    let rendered = render(&content);
    let cls = if class.is_empty() {
        "markdown-body".to_string()
    } else {
        format!("markdown-body {class}")
    };
    view! { <div class=cls inner_html=rendered></div> }
}
