use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="footer-inner">
                <span class="footer-text">"IdeaForge — Where Ideas Take Shape"</span>
                <div class="footer-links">
                    <A href="/browse">"Forge Floor"</A>
                    <a href="https://github.com/ideaforge" target="_blank" rel="noopener">"GitHub"</a>
                </div>
            </div>
        </footer>
    }
}
