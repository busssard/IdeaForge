//! Transparency badge: cross-checks every SRI `integrity="..."` hash your
//! browser actually loaded against the canonical `HASHES.txt` generated at
//! build time (see `scripts/emit-hashes.sh`).
//!
//! What this proves (and doesn't):
//! - The browser already enforces each asset's SRI hash — if the deployed
//!   asset didn't match the one `index.html` advertises, it wouldn't have
//!   loaded at all. This component shows that the advertised hash also
//!   matches the published manifest.
//! - It doesn't prove the published manifest matches the source repo —
//!   that's a separate cross-check the user does offline by rebuilding
//!   at the same commit. We link to that procedure.

use leptos::prelude::*;
use std::collections::HashSet;
use wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq)]
enum Check {
    Loading,
    Match { loaded: usize, published: usize },
    Mismatch { loaded: Vec<String>, missing: Vec<String> },
    NoManifest,
    Error(String),
}

#[component]
pub fn IntegrityCheck() -> impl IntoView {
    let state = RwSignal::new(Check::Loading);

    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let result = run_check().await;
            state.set(result);
        });
    });

    view! {
        <div class="integrity-card card">
            <header class="integrity-header">
                <h3>"Build integrity"</h3>
                {move || match state.get() {
                    Check::Loading => view! { <span class="integrity-pill integrity-pill-check">"Checking…"</span> }.into_any(),
                    Check::Match { .. } => view! {
                        <span class="integrity-pill integrity-pill-ok">"\u{2713} Verified"</span>
                    }.into_any(),
                    Check::Mismatch { .. } => view! {
                        <span class="integrity-pill integrity-pill-bad">"\u{2717} Mismatch"</span>
                    }.into_any(),
                    Check::NoManifest => view! {
                        <span class="integrity-pill integrity-pill-warn">"No manifest"</span>
                    }.into_any(),
                    Check::Error(_) => view! {
                        <span class="integrity-pill integrity-pill-bad">"Error"</span>
                    }.into_any(),
                }}
            </header>

            <p class="integrity-explainer">
                "Your browser enforces Subresource Integrity hashes on every asset it loads. "
                "This check cross-references the hashes in your live DOM against the canonical "
                <code>"HASHES.txt"</code>
                " generated when the build was published."
            </p>

            {move || match state.get() {
                Check::Match { loaded, published } => view! {
                    <p class="integrity-detail">
                        {format!(
                            "{loaded} of {loaded} loaded assets match the published manifest \
                             ({published} total entries)."
                        )}
                    </p>
                }.into_any(),
                Check::Mismatch { loaded, missing } => {
                    let shown: Vec<String> = loaded.iter().take(4).cloned().collect();
                    let missing_shown: Vec<String> = missing.iter().take(4).cloned().collect();
                    view! {
                        <p class="integrity-detail integrity-bad">
                            "One or more assets have hashes that are not in the published manifest."
                        </p>
                        <details class="integrity-details">
                            <summary>"Show diff"</summary>
                            <p class="integrity-subhead">"Loaded by the browser:"</p>
                            <ul class="integrity-list">
                                {shown.into_iter().map(|l| view! { <li>{l}</li> }).collect::<Vec<_>>()}
                            </ul>
                            <p class="integrity-subhead">"Not present in HASHES.txt:"</p>
                            <ul class="integrity-list">
                                {missing_shown.into_iter().map(|l| view! { <li>{l}</li> }).collect::<Vec<_>>()}
                            </ul>
                        </details>
                    }.into_any()
                }
                Check::NoManifest => view! {
                    <p class="integrity-detail">
                        "This server doesn't publish a "
                        <code>"HASHES.txt"</code>
                        ". The browser is still enforcing SRI — "
                        "you can inspect the loaded hashes below and compare manually."
                    </p>
                    <LoadedHashes />
                }.into_any(),
                Check::Error(msg) => view! {
                    <p class="integrity-detail integrity-bad">{format!("Check failed: {msg}")}</p>
                }.into_any(),
                Check::Loading => view! { <p class="integrity-detail text-muted">"Collecting hashes…"</p> }.into_any(),
            }}

            <p class="integrity-howto">
                "To fully verify a deployment: clone the repo at the release tag, run "
                <code>"trunk build --release"</code>
                ", and diff the produced "
                <code>"dist/HASHES.txt"</code>
                " against the one at "
                <a
                    href="/HASHES.txt"
                    target="_blank"
                    rel="noopener noreferrer"
                >"this server"</a>
                "."
            </p>
        </div>
    }
}

/// Walks the live DOM once, extracts every `(url, integrity)` pair, and
/// compares against `/HASHES.txt`.
async fn run_check() -> Check {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return Check::Error("no document".into());
    };
    let nodes = match doc.query_selector_all("[integrity]") {
        Ok(n) => n,
        Err(e) => return Check::Error(format!("querySelectorAll: {e:?}")),
    };
    let mut loaded: Vec<(String, String)> = Vec::new();
    for i in 0..nodes.length() {
        let Some(node) = nodes.item(i) else { continue };
        let Ok(el) = node.dyn_into::<web_sys::Element>() else { continue };
        let Some(integ) = el.get_attribute("integrity") else { continue };
        let href = el
            .get_attribute("href")
            .or_else(|| el.get_attribute("src"))
            .unwrap_or_default();
        loaded.push((integ, filename(&href)));
    }

    if loaded.is_empty() {
        return Check::Error("No integrity-tagged assets found in the DOM.".into());
    }

    // Fetch the canonical manifest. Anything non-200 means we fall back to
    // displaying the loaded hashes without a comparison.
    let resp = match gloo_net::http::Request::get("/HASHES.txt").send().await {
        Ok(r) => r,
        Err(e) => return Check::Error(format!("fetch HASHES.txt: {e}")),
    };
    if resp.status() == 404 {
        return Check::NoManifest;
    }
    if !resp.ok() {
        return Check::Error(format!("HASHES.txt returned {}", resp.status()));
    }
    let body = match resp.text().await {
        Ok(t) => t,
        Err(e) => return Check::Error(format!("read body: {e}")),
    };

    // Parse "integrity  filename" lines, skip comments/blank.
    let mut published: HashSet<String> = HashSet::new();
    let mut published_count = 0usize;
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Split on whitespace, first column is the integrity hash.
        let Some(integ) = line.split_whitespace().next() else { continue };
        published.insert(integ.to_string());
        published_count += 1;
    }

    let mut missing: Vec<String> = Vec::new();
    let loaded_names: Vec<String> = loaded.iter().map(|(_, n)| n.clone()).collect();
    for (integ, name) in &loaded {
        if !published.contains(integ) {
            missing.push(format!("{name}  ({integ})"));
        }
    }
    if missing.is_empty() {
        Check::Match {
            loaded: loaded.len(),
            published: published_count,
        }
    } else {
        Check::Mismatch {
            loaded: loaded_names,
            missing,
        }
    }
}

fn filename(href: &str) -> String {
    href.rsplit('/').next().unwrap_or(href).to_string()
}

/// Secondary view: just list the loaded hashes when there's no manifest to
/// compare against. Built synchronously at render time — the DOM is already
/// populated by the time this component renders (our own bundle finished
/// loading, otherwise nothing would run).
#[component]
fn LoadedHashes() -> impl IntoView {
    let items = collect_loaded_hashes();
    view! {
        <ul class="integrity-list">
            {items.into_iter().map(|(name, hash)| {
                view! { <li><code>{hash}</code>" — "{name}</li> }
            }).collect::<Vec<_>>()}
        </ul>
    }
}

fn collect_loaded_hashes() -> Vec<(String, String)> {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return Vec::new();
    };
    let nodes: web_sys::NodeList = match doc.query_selector_all("[integrity]") {
        Ok(n) => n,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for i in 0..nodes.length() {
        let Some(node) = nodes.item(i) else { continue };
        let Ok(el) = node.dyn_into::<web_sys::Element>() else { continue };
        let integ = el.get_attribute("integrity").unwrap_or_default();
        let href = el
            .get_attribute("href")
            .or_else(|| el.get_attribute("src"))
            .unwrap_or_default();
        out.push((filename(&href), integ));
    }
    out
}
