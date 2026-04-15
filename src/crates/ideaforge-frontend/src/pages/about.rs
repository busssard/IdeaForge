//! /about — how IdeaForge private messaging works, with a live integrity
//! check that any visitor can run against the repo at any release tag.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::integrity_check::IntegrityCheck;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="about-page">
            <header class="about-hero">
                <h1>"How IdeaForge works"</h1>
                <p class="text-muted">
                    "IdeaForge connects entrepreneurs, makers, and makers-to-be. This page walks
                    through the pieces most worth knowing about — especially around private
                    messaging, which is end-to-end encrypted and auditable."
                </p>
            </header>

            <section class="about-section">
                <h2>"Private messaging, end to end"</h2>
                <p>
                    "Messages between users are encrypted in the browser via "
                    <a
                        href="https://datatracker.ietf.org/doc/rfc9420/"
                        target="_blank"
                        rel="noopener noreferrer"
                    >"MLS (RFC 9420)"</a>
                    ". Your 6-digit PIN derives (via Argon2id) the key that unlocks your local MLS
                    keystore — the server never sees the PIN, the derived key, or your plaintext."
                </p>
                <figure class="about-infographic">
                    <img
                        src="/messaging-infographic.svg"
                        alt="How IdeaForge messages stay private — Alice's browser encrypts, the server routes ciphertext only, Bob's browser decrypts."
                    />
                    <figcaption>
                        "Alice → IdeaForge → Bob. The middle column carries ciphertext only."
                    </figcaption>
                </figure>
            </section>

            <section class="about-section">
                <h2>"Verify this build"</h2>
                <p>
                    "Every asset your browser loaded carries an SRI hash. The list below matches
                    those hashes against the "
                    <code>"HASHES.txt"</code>
                    " manifest published with this release. If any asset differs, the browser
                    refuses to run it — you'll see a mismatch banner."
                </p>
                <IntegrityCheck />
            </section>

            <section class="about-section">
                <h2>"Open source"</h2>
                <p>
                    "The whole stack — Rust backend, Leptos/WASM frontend, database migrations —
                    is open source. That's the ground truth that makes the integrity check above
                    meaningful: you can clone, rebuild from the same commit, and confirm the
                    hashes match."
                </p>
                <p>
                    <A href="/browse" attr:class="btn btn-secondary">"Back to the Forge Floor"</A>
                </p>
            </section>
        </div>
    }
}
