//! TypeScript diagnostics parsing and remapping.

use serde::{Deserialize, Serialize};
use source_map::{LineIndex, SourceMap, Span};
use std::collections::HashMap;
use std::path::PathBuf;

/// A collection of TypeScript diagnostics.
#[derive(Debug, Clone, Default)]
pub struct TsDiagnostics {
    /// All diagnostics.
    pub diagnostics: Vec<TsDiagnostic>,
    /// Total error count.
    pub error_count: usize,
    /// Total warning count.
    pub warning_count: usize,
}

impl TsDiagnostics {
    /// Create a new empty diagnostics collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diagnostic.
    pub fn add(&mut self, diagnostic: TsDiagnostic) {
        match diagnostic.severity {
            TsSeverity::Error => self.error_count += 1,
            TsSeverity::Warning => self.warning_count += 1,
            _ => {}
        }
        self.diagnostics.push(diagnostic);
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Get diagnostics for a specific file.
    pub fn for_file(&self, file: &str) -> Vec<&TsDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.file.as_ref().map(|f| f.to_string_lossy()) == Some(file.into()))
            .collect()
    }

    /// Sort diagnostics by file and line.
    pub fn sort(&mut self) {
        self.diagnostics.sort_by(|a, b| {
            match (&a.file, &b.file) {
                (Some(fa), Some(fb)) => {
                    let file_cmp = fa.cmp(fb);
                    if file_cmp != std::cmp::Ordering::Equal {
                        return file_cmp;
                    }
                }
                (Some(_), None) => return std::cmp::Ordering::Less,
                (None, Some(_)) => return std::cmp::Ordering::Greater,
                (None, None) => {}
            }
            a.line.cmp(&b.line).then(a.column.cmp(&b.column))
        });
    }
}

/// A single TypeScript diagnostic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsDiagnostic {
    /// The diagnostic message.
    pub message: String,
    /// The TypeScript error code.
    pub code: u32,
    /// The severity.
    pub severity: TsSeverity,
    /// The file path (if any).
    pub file: Option<PathBuf>,
    /// Line number (1-indexed).
    pub line: Option<u32>,
    /// Column number (1-indexed).
    pub column: Option<u32>,
    /// End line (1-indexed).
    pub end_line: Option<u32>,
    /// End column (1-indexed).
    pub end_column: Option<u32>,
    /// Related information.
    #[serde(default)]
    pub related: Vec<RelatedInfo>,
}

impl TsDiagnostic {
    /// Get the span of the diagnostic.
    pub fn span(&self) -> Option<Span> {
        // This would need the line index to convert line/col to offsets
        None
    }

    /// Format the diagnostic for display.
    pub fn format(&self) -> String {
        let mut result = String::new();

        // Location
        if let Some(file) = &self.file {
            result.push_str(&file.to_string_lossy());
            if let (Some(line), Some(col)) = (self.line, self.column) {
                result.push_str(&format!(":{}:{}", line, col));
            }
            result.push_str(" - ");
        }

        // Severity and code
        result.push_str(&format!("{} TS{}: ", self.severity.as_str(), self.code));

        // Message
        result.push_str(&self.message);

        result
    }
}

/// Related information for a diagnostic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedInfo {
    /// The message.
    pub message: String,
    /// The file path.
    pub file: Option<PathBuf>,
    /// Line number.
    pub line: Option<u32>,
    /// Column number.
    pub column: Option<u32>,
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TsSeverity {
    /// Error.
    Error,
    /// Warning.
    Warning,
    /// Suggestion.
    Suggestion,
    /// Message.
    Message,
}

impl TsSeverity {
    /// Get the severity as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Suggestion => "suggestion",
            Self::Message => "message",
        }
    }
}

impl Default for TsSeverity {
    fn default() -> Self {
        Self::Error
    }
}

/// Remapper for converting virtual file positions to original positions.
pub struct DiagnosticRemapper {
    /// Map from virtual file to original file.
    virtual_to_original: HashMap<PathBuf, PathBuf>,
    /// Source maps for each virtual file.
    source_maps: HashMap<PathBuf, SourceMap>,
    /// Line indices for original files.
    line_indices: HashMap<PathBuf, LineIndex>,
}

impl DiagnosticRemapper {
    /// Create a new remapper.
    pub fn new() -> Self {
        Self {
            virtual_to_original: HashMap::new(),
            source_maps: HashMap::new(),
            line_indices: HashMap::new(),
        }
    }

    /// Register a virtual file mapping.
    pub fn register(
        &mut self,
        virtual_file: PathBuf,
        original_file: PathBuf,
        source_map: SourceMap,
        original_content: &str,
    ) {
        self.virtual_to_original
            .insert(virtual_file.clone(), original_file.clone());
        self.source_maps.insert(virtual_file, source_map);
        self.line_indices
            .insert(original_file, LineIndex::new(original_content));
    }

    /// Remap a diagnostic from virtual to original positions.
    pub fn remap(&self, diagnostic: &mut TsDiagnostic) {
        let file = match &diagnostic.file {
            Some(f) => f,
            None => return,
        };

        // Check if this is a virtual file
        let original_file = match self.virtual_to_original.get(file) {
            Some(f) => f,
            None => return,
        };

        let source_map = match self.source_maps.get(file) {
            Some(sm) => sm,
            None => return,
        };

        // Update file path
        diagnostic.file = Some(original_file.clone());

        // Remap position
        if let (Some(_line), Some(col)) = (diagnostic.line, diagnostic.column) {
            // Convert line/col to offset in virtual file
            // Then use source map to get original offset
            // Then convert back to line/col

            // This is a simplified version - a full implementation would need
            // the line index for the virtual file as well
            if let Some(mapping) = source_map.find_source(col) {
                if let Some(line_index) = self.line_indices.get(original_file) {
                    let orig_line_col = line_index.line_col(mapping.source_offset);
                    diagnostic.line = Some(orig_line_col.line + 1);
                    diagnostic.column = Some(orig_line_col.col + 1);
                }
            }
        }
    }

    /// Remap all diagnostics.
    pub fn remap_all(&self, diagnostics: &mut TsDiagnostics) {
        for diagnostic in &mut diagnostics.diagnostics {
            self.remap(diagnostic);
        }
    }
}

impl Default for DiagnosticRemapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse TypeScript JSON output.
pub fn parse_ts_output(output: &str) -> Vec<TsDiagnostic> {
    // TypeScript can output diagnostics in various formats
    // This handles the JSON format from tsc --build or custom JSON output

    let mut diagnostics = Vec::new();

    // Try to parse as JSON array
    if let Ok(parsed) = serde_json::from_str::<Vec<TsDiagnosticJson>>(output) {
        for item in parsed {
            diagnostics.push(item.into());
        }
        return diagnostics;
    }

    // Try to parse line by line (standard tsc output)
    for line in output.lines() {
        if let Some(diag) = parse_tsc_line(line) {
            diagnostics.push(diag);
        }
    }

    diagnostics
}

/// JSON format for TypeScript diagnostics.
#[derive(Debug, Deserialize)]
struct TsDiagnosticJson {
    #[serde(rename = "messageText")]
    message_text: StringOrNested,
    code: u32,
    category: u32,
    #[serde(rename = "fileName")]
    file_name: Option<String>,
    start: Option<Location>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StringOrNested {
    String(String),
    Nested { message_text: String },
}

impl StringOrNested {
    fn as_str(&self) -> &str {
        match self {
            Self::String(s) => s,
            Self::Nested { message_text } => message_text,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Location {
    line: u32,
    character: u32,
}

impl From<TsDiagnosticJson> for TsDiagnostic {
    fn from(json: TsDiagnosticJson) -> Self {
        Self {
            message: json.message_text.as_str().to_string(),
            code: json.code,
            severity: match json.category {
                1 => TsSeverity::Error,
                2 => TsSeverity::Warning,
                3 => TsSeverity::Suggestion,
                _ => TsSeverity::Message,
            },
            file: json.file_name.map(PathBuf::from),
            line: json.start.as_ref().map(|l| l.line + 1),
            column: json.start.as_ref().map(|l| l.character + 1),
            end_line: None,
            end_column: None,
            related: Vec::new(),
        }
    }
}

/// Parse a single line of tsc output.
fn parse_tsc_line(line: &str) -> Option<TsDiagnostic> {
    // Format: file(line,col): severity TScode: message
    // Example: src/main.ts(10,5): error TS2322: Type 'string' is not assignable to type 'number'.

    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Find the position info
    let paren_start = line.find('(')?;
    let paren_end = line.find(')')?;
    let colon_after_paren = line[paren_end..].find(':')? + paren_end;

    let file = &line[..paren_start];
    let position = &line[paren_start + 1..paren_end];
    let rest = &line[colon_after_paren + 1..].trim();

    // Parse line,col
    let mut pos_parts = position.split(',');
    let line_num: u32 = pos_parts.next()?.parse().ok()?;
    let col_num: u32 = pos_parts.next()?.parse().ok()?;

    // Parse severity and code
    let severity_end = rest.find(' ')?;
    let severity_str = &rest[..severity_end];

    let ts_start = rest.find("TS")?;
    let code_end = rest[ts_start + 2..].find(':')? + ts_start + 2;
    let code: u32 = rest[ts_start + 2..code_end].parse().ok()?;

    let message = rest[code_end + 1..].trim().to_string();

    let severity = match severity_str {
        "error" => TsSeverity::Error,
        "warning" => TsSeverity::Warning,
        _ => TsSeverity::Message,
    };

    Some(TsDiagnostic {
        message,
        code,
        severity,
        file: Some(PathBuf::from(file)),
        line: Some(line_num),
        column: Some(col_num),
        end_line: None,
        end_column: None,
        related: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tsc_line() {
        let line = "src/main.ts(10,5): error TS2322: Type 'string' is not assignable to type 'number'.";
        let diag = parse_tsc_line(line).unwrap();
        assert_eq!(diag.file, Some(PathBuf::from("src/main.ts")));
        assert_eq!(diag.line, Some(10));
        assert_eq!(diag.column, Some(5));
        assert_eq!(diag.code, 2322);
        assert_eq!(diag.severity, TsSeverity::Error);
    }

    #[test]
    fn test_ts_diagnostics() {
        let mut diags = TsDiagnostics::new();
        diags.add(TsDiagnostic {
            message: "Test error".to_string(),
            code: 1000,
            severity: TsSeverity::Error,
            file: None,
            line: None,
            column: None,
            end_line: None,
            end_column: None,
            related: Vec::new(),
        });
        assert!(diags.has_errors());
        assert_eq!(diags.error_count, 1);
    }
}
