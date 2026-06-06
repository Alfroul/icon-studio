use std::path::{Path, PathBuf};

/// Validate a file path for safe filesystem access.
/// Returns the canonicalized path or an error if path traversal is detected.
pub fn validate_file_path(input: &str) -> Result<PathBuf, String> {
    let path = Path::new(input);

    // Reject obviously malicious patterns
    if input.contains("..") {
        return Err(format!(
            "Invalid path: path traversal detected in '{}'",
            input
        ));
    }

    // Canonicalize if the file/directory exists (for reads), or validate parent dir (for writes)
    if path.exists() {
        path.canonicalize()
            .map_err(|e| format!("Failed to canonicalize path '{}': {}", input, e))
    } else {
        // For write paths, ensure parent exists
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(format!(
                    "Parent directory does not exist for path '{}'",
                    input
                ));
            }
        }
        Ok(path.to_path_buf())
    }
}

/// Escape special characters for use in XML attribute values.
pub fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape special characters for use in XML text content.
pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
