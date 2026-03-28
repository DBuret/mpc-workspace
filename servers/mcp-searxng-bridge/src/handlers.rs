use crate::error::AgentError;
use crate::state::AppState;

use html2md::parse_html;
use readability::extractor;
use tracing::{debug, error, info, instrument, warn};

#[instrument(skip(state), fields(query = %query))]
pub async fn call_searxng(state: &AppState, query: &str) -> Result<String, AgentError> {
if query.is_empty() {
warn!("Requête de recherche vide reçue");
return Ok("Query is empty".into());
}

```
debug!("Envoi de la requête à SearXNG...");

let params = [("q", query), ("format", "json"), ("language", "en-US")];

let resp = state
    .client
    .get(&format!("{}/search", state.url))
    .query(&params)
    .send()
    .await
    .map_err(|e| {
        error!(error = %e, "Échec de la connexion à SearXNG");
        e
    })?;

if !resp.status().is_success() {
    let status = resp.status();
    error!(status = %status, "SearXNG a retourné une erreur");
    return Err(AgentError::Api(format!("SearXNG error: HTTP {}", status)));
}

let json: serde_json::Value = resp.json().await?;
let mut out = String::new();

if let Some(results) = json.get("results").and_then(|r| r.as_array()) {
    for res in results.iter().take(5) {
        let title = res.get("title").and_then(|t| t.as_str()).unwrap_or("");
        let content = res.get("content").and_then(|c| c.as_str()).unwrap_or("");
        let url = res.get("url").and_then(|u| u.as_str()).unwrap_or("");
        out.push_str(&format!("### {}\n{}\nSource: {}\n\n", title, content, url));
    }
}

Ok(if out.is_empty() {
    "No results found".into()
} else {
    out
})
```

}

#[instrument(skip(state), fields(url = %url))]
pub async fn fetch_url(state: &AppState, url: &str) -> Result<String, AgentError> {
if url.is_empty() {
return Ok("URL is empty".into());
}

```
// ── 1. Fetch ──────────────────────────────────────────────────────────────
let resp = state.client.get(url).send().await?;

if !resp.status().is_success() {
    return Ok(format!("HTTP error {}: cannot fetch page.", resp.status()));
}

// ── 2. Guard: refuse non-HTML content types ───────────────────────────────
let content_type = resp
    .headers()
    .get(reqwest::header::CONTENT_TYPE)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("")
    .to_ascii_lowercase();

if !content_type.contains("text/html") && !content_type.contains("text/plain") {
    info!(content_type, "Skipping non-HTML resource");
    return Ok(format!(
        "Skipped: unsupported content type `{content_type}`."
    ));
}

// ── 3. Read body ──────────────────────────────────────────────────────────
let html = resp.text().await?;
debug!(bytes = html.len(), "HTML body received");

// ── 4. Extract main content via Readability ───────────────────────────────
let parsed_url = url
    .parse()
    .map_err(|_| AgentError::Api(format!("Invalid URL: {url}")))?;

let product = extractor::extract(&mut html.as_bytes(), &parsed_url)
    .map_err(|e| AgentError::Api(format!("Readability error: {e}")))?;

if product.content.is_empty() {
    return Ok("Page loaded but no text content could be extracted.".into());
}

// ── 5. Convert cleaned HTML → Markdown ───────────────────────────────────
let markdown = parse_html(&product.content);
let markdown = markdown.trim();

Ok(truncate_at_word(markdown, 12_000))
```

}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Truncate at the last word boundary before `max_chars`, appending a notice.
fn truncate_at_word(text: &str, max_chars: usize) -> String {
if text.len() <= max_chars {
return text.to_string();
}

```
let boundary = text[..max_chars]
    .rfind(|c: char| c.is_whitespace())
    .unwrap_or(max_chars);

format!(
    "{}\n\n*(Content truncated at {} chars)*",
    text[..boundary].trim(),
    max_chars
)
```

}