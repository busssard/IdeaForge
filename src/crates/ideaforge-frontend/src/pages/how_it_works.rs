//! /how-it-works — the messaging/encryption deep-dive plus a live integrity
//! check that any visitor can run against the repo at any release tag.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::integrity_check::IntegrityCheck;

#[component]
pub fn HowItWorksPage() -> impl IntoView {
    view! {
        <div class="about-page">
            <header class="about-hero">
                <h1>"How private messaging works"</h1>
                <p class="text-muted">
                    "IdeaForge messages are end-to-end encrypted in the browser. The server
                    routes ciphertext; only the sender and recipient can read the plaintext.
                    Here's the short version, plus a live check you can run yourself."
                </p>
            </header>

            <section class="about-section">
                <h2>"End-to-end, in your browser"</h2>
                <p>
                    "Messages are encrypted via "
                    <a
                        href="https://datatracker.ietf.org/doc/rfc9420/"
                        target="_blank"
                        rel="noopener noreferrer"
                    >"MLS (RFC 9420)"</a>
                    ". Your 6-digit PIN derives (via Argon2id) the key that unlocks your local
                    MLS keystore — the server never sees the PIN, the derived key, or your
                    plaintext. Forget the PIN and your history is gone; there is no reset,
                    because there is nothing on our side to reset from."
                </p>
                <figure class="about-infographic">
                    <img
                        src="/static/messaging-infographic.svg"
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
                    "Every asset your browser loaded carries a Subresource Integrity hash.
                    The list below matches those hashes against the "
                    <code>"HASHES.txt"</code>
                    " manifest published with this release. If any asset differs, the browser
                    refuses to run it — you'll see a mismatch banner."
                </p>
                <IntegrityCheck />
            </section>

            <section class="about-section">
                <h2>"Open source, so you can check for yourself"</h2>
                <p>
                    "The whole stack — Rust backend, Leptos/WASM frontend, database migrations —
                    is open source. That's the ground truth that makes the integrity check above
                    meaningful: clone the repo at the release tag, rebuild from the same commit,
                    and confirm the hashes match."
                </p>
                <p>
                    <A href="/about" attr:class="btn btn-secondary">"About IdeaForge"</A>
                    " "
                    <A href="/browse" attr:class="btn btn-ghost">"Back to the Forge Floor"</A>
                </p>
            </section>
        </div>
    }
}
