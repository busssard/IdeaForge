use leptos::prelude::*;

#[component]
pub fn MaturityBadge(#[prop(into)] maturity: String) -> impl IntoView {
    let (class, label) = match maturity.as_str() {
        "spark" => ("badge badge-spark", "Spark"),
        "half_baked" => ("badge badge-half-baked", "Half-Baked"),
        "thought_through" => ("badge badge-thought-through", "Thought Through"),
        "serious" => ("badge badge-serious", "Serious Proposal"),
        "in_work" => ("badge badge-in-work", "In Work"),
        "almost_finished" => ("badge badge-almost-finished", "Almost Finished"),
        "completed" => ("badge badge-completed", "Completed"),
        _ => ("badge badge-spark", "Unknown"),
    };

    // Surfaces on hover — the long-form explanation lives in the details panel
    // on the idea page. This keeps the badge itself terse.
    let tooltip = "Maturity advances as the community engages — stokes, comments, \
                   suggestions, team activity. It isn't manually set.";

    view! {
        <span class=class title=tooltip>{label}</span>
    }
}
