use leptos::prelude::*;

/// Displays a badge indicating the idea's visibility type.
#[component]
pub fn VisibilityBadge(openness: String) -> impl IntoView {
    let (class, label, icon) = match openness.as_str() {
        "open" => ("badge badge-open", "Open Source".to_string(), "\u{1F513}"),
        "collaborative" => ("badge badge-collaborative", "Collaborative".to_string(), "\u{1F91D}"),
        "commercial" => ("badge badge-commercial", "Commercial".to_string(), "\u{1F4BC}"),
        "private" => ("badge badge-private", "Private".to_string(), "\u{1F512}"),
        other => ("badge", other.to_string(), ""),
    };

    let label_title = label.clone();

    view! {
        <span class=class title=label_title>
            <span class="badge-icon">{icon}</span>
            " "
            {label}
        </span>
    }
}
