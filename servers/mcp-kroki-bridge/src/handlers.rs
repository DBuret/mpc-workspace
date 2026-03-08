use crate::state::AppState;
use crate::error::AgentError;
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::io::Write;
use tracing::{debug};

pub fn generate_url(base_url: &str, kroki_type: &str, source: &str) -> String {
    // 1. Compression Zlib (indispensable pour matcher ton test Python/Bash)
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(source.as_bytes()).unwrap();
    let compressed = encoder.finish().expect("Échec de la compression");

    // 2. Base64 URL Safe
    let b64 = URL_SAFE.encode(&compressed);

    // 3. Construction manuelle
    // On ne passe pas par url.path_segments_mut().push() pour éviter le percent-encoding de '-' et '_'
    let base = base_url.trim_end_matches('/');
    let final_url = format!("{}/{}/svg/{}", base, kroki_type, b64);

    // --- TRACE DEBUG ---
    debug!(
        "[DEBUG] Kroki URL generated for {}: {}",
        kroki_type, final_url
    );

    final_url
}


