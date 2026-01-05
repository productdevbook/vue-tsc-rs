//! Lexer for Vue Single File Components.

use source_map::Span;

/// A lexer for Vue SFC files.
pub struct SfcLexer<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> SfcLexer<'a> {
    /// Create a new lexer for the given source.
    pub fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    /// Get the current position.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Get the remaining source.
    pub fn remaining(&self) -> &'a str {
        &self.source[self.pos..]
    }

    /// Peek at the next character.
    pub fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    /// Consume and return the next character.
    pub fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Skip whitespace and return the number of bytes skipped.
    pub fn skip_whitespace(&mut self) -> usize {
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
        self.pos - start
    }

    /// Check if the remaining source starts with the given string.
    pub fn starts_with(&self, s: &str) -> bool {
        self.remaining().starts_with(s)
    }

    /// Consume a string if the remaining source starts with it.
    pub fn consume(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    /// Consume characters while the predicate is true.
    pub fn consume_while<F>(&mut self, pred: F) -> &'a str
    where
        F: Fn(char) -> bool,
    {
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if pred(c) {
                self.next_char();
            } else {
                break;
            }
        }
        &self.source[start..self.pos]
    }

    /// Consume until the given string is found.
    pub fn consume_until(&mut self, s: &str) -> &'a str {
        let start = self.pos;
        while !self.remaining().is_empty() && !self.starts_with(s) {
            self.next_char();
        }
        &self.source[start..self.pos]
    }

    /// Consume until one of the given strings is found.
    pub fn consume_until_any(&mut self, patterns: &[&str]) -> &'a str {
        let start = self.pos;
        while !self.remaining().is_empty() {
            if patterns.iter().any(|p| self.starts_with(p)) {
                break;
            }
            self.next_char();
        }
        &self.source[start..self.pos]
    }

    /// Read a tag name (identifier).
    pub fn read_tag_name(&mut self) -> Option<&'a str> {
        let start = self.pos;
        // Tag name must start with letter or underscore
        match self.peek_char() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                self.next_char();
            }
            _ => return None,
        }
        // Consume remaining valid characters
        self.consume_while(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':');
        Some(&self.source[start..self.pos])
    }

    /// Read an attribute name.
    pub fn read_attr_name(&mut self) -> Option<&'a str> {
        let start = self.pos;
        // Attribute name can include special Vue prefixes
        match self.peek_char() {
            Some(c)
                if c.is_ascii_alphabetic()
                    || c == '_'
                    || c == ':'
                    || c == '@'
                    || c == '#'
                    || c == 'v' =>
            {
                self.next_char();
            }
            _ => return None,
        }
        // Consume remaining valid characters
        self.consume_while(|c| {
            c.is_ascii_alphanumeric()
                || c == '-'
                || c == '_'
                || c == ':'
                || c == '.'
                || c == '['
                || c == ']'
        });
        Some(&self.source[start..self.pos])
    }

    /// Read a quoted string value.
    pub fn read_quoted_string(&mut self) -> Option<(&'a str, char)> {
        let quote = self.peek_char()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        self.next_char(); // Consume opening quote

        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c == quote {
                let value = &self.source[start..self.pos];
                self.next_char(); // Consume closing quote
                return Some((value, quote));
            }
            // Handle escape sequences
            if c == '\\' {
                self.next_char();
                self.next_char(); // Skip escaped char
            } else {
                self.next_char();
            }
        }
        // Unterminated string - return what we have
        Some((&self.source[start..self.pos], quote))
    }

    /// Read an unquoted attribute value.
    pub fn read_unquoted_value(&mut self) -> &'a str {
        self.consume_while(|c| !c.is_whitespace() && c != '>' && c != '/' && c != '=')
    }

    /// Read a comment.
    pub fn read_comment(&mut self) -> Option<&'a str> {
        if !self.consume("<!--") {
            return None;
        }
        let content = self.consume_until("-->");
        self.consume("-->");
        Some(content)
    }

    /// Read block content until the closing tag.
    pub fn read_block_content(&mut self, closing_tag: &str) -> &'a str {
        let start = self.pos;
        let pattern = format!("</{}", closing_tag);

        while !self.remaining().is_empty() {
            // Check for closing tag (case-insensitive)
            if self.remaining().len() >= pattern.len() {
                let potential = &self.remaining()[..pattern.len()];
                if potential.eq_ignore_ascii_case(&pattern) {
                    // Check if followed by > or whitespace
                    let after = self.remaining().chars().nth(pattern.len());
                    if matches!(
                        after,
                        Some('>') | Some(' ') | Some('\t') | Some('\n') | Some('\r') | None
                    ) {
                        break;
                    }
                }
            }
            self.next_char();
        }
        &self.source[start..self.pos]
    }

    /// Check if at end of input.
    pub fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    /// Get a span from start to current position.
    pub fn span_from(&self, start: usize) -> Span {
        Span::new(start as u32, self.pos as u32)
    }

    /// Get the source slice for a span.
    pub fn slice(&self, span: Span) -> &'a str {
        &self.source[span.start as usize..span.end as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_tag_name() {
        let mut lexer = SfcLexer::new("template>");
        assert_eq!(lexer.read_tag_name(), Some("template"));
    }

    #[test]
    fn test_read_quoted_string() {
        let mut lexer = SfcLexer::new("\"hello world\"");
        assert_eq!(lexer.read_quoted_string(), Some(("hello world", '"')));
    }

    #[test]
    fn test_read_comment() {
        let mut lexer = SfcLexer::new("<!-- this is a comment -->");
        assert_eq!(lexer.read_comment(), Some(" this is a comment "));
    }

    #[test]
    fn test_read_block_content() {
        let mut lexer = SfcLexer::new("<div>Hello</div></template>");
        let content = lexer.read_block_content("template");
        assert_eq!(content, "<div>Hello</div>");
    }
}
