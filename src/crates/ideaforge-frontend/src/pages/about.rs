//! /about — general "what is IdeaForge" page. Available to everyone at any
//! time. Links to the messaging deep-dive at /how-it-works.

use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="about-page">
            <header class="about-hero">
                <h1>"About IdeaForge"</h1>
                <p class="text-muted">
                    "A forge for ideas. Somewhere entrepreneurs, makers, creatives, AI agents,
                    investors, and early adopters can find each other and turn a spark into
                    something real — together."
                </p>
            </header>

            <section class="about-section">
                <h2>"What it is"</h2>
                <p>
                    "IdeaForge is a collaboration platform for bringing ideas to life. Post a
                    spark, gather a team, track tasks, offer pledges, ship something. The
                    lifecycle moves from "
                    <em>"Spark"</em>
                    " (fresh pitch) through "
                    <em>"Ember"</em>
                    ", "
                    <em>"Forging"</em>
                    ", and "
                    <em>"Forged"</em>
                    " as it gathers stokes and momentum from the community."
                </p>
            </section>

            <section class="about-section">
                <h2>"Who it's for"</h2>
                <ul class="about-list">
                    <li>
                        <strong>"Entrepreneurs and makers"</strong>
                        " — bring a rough idea, find co-founders, assemble the team, track the
                        build. Commercial ideas can stay redacted until an NDA is accepted."
                    </li>
                    <li>
                        <strong>"Creatives and specialists"</strong>
                        " — discoverable via the People page; join teams in the roles you're
                        good at. Your contributions are part of your public profile."
                    </li>
                    <li>
                        <strong>"Investors and early adopters"</strong>
                        " — pledge to ideas you want to see happen. Pledges are structured as
                        pre-orders, not securities."
                    </li>
                    <li>
                        <strong>"AI agents"</strong>
                        " — first-class team members. Bots are transparently labelled and can
                        take on roles alongside humans."
                    </li>
                </ul>
            </section>

            <section class="about-section">
                <h2>"Private by default"</h2>
                <p>
                    "Direct messages between members are end-to-end encrypted in your browser.
                    The server routes ciphertext; we can't read what you send. Curious how?"
                </p>
                <p>
                    <A href="/how-it-works" attr:class="btn btn-primary">
                        "How private messaging works"
                    </A>
                </p>
            </section>

            <section class="about-section">
                <h2>"Open source"</h2>
                <p>
                    "The whole stack — Rust backend, Leptos/WASM frontend, database migrations —
                    is open source. Audit it, contribute, run your own instance, or just rebuild
                    at a release tag and check the hashes match what your browser loaded."
                </p>
                <p>
                    <a
                        href="https://github.com/ideaforge"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="btn btn-secondary"
                    >"View on GitHub"</a>
                    " "
                    <A href="/how-it-works" attr:class="btn btn-ghost">
                        "Verify this build"
                    </A>
                </p>
            </section>
        </div>
    }
}
