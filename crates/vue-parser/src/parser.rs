//! Parser for Vue Single File Components.

use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use crate::lexer::SfcLexer;
use source_map::Span;

/// Parse a Vue SFC from source code.
pub fn parse_sfc(source: &str) -> ParseResult<Sfc> {
    let mut parser = SfcParser::new(source);
    parser.parse()
}

/// Parser for Vue SFC files.
struct SfcParser<'a> {
    lexer: SfcLexer<'a>,
    source: &'a str,
    errors: Vec<ParseError>,
}

impl<'a> SfcParser<'a> {
    /// Create a new parser for the given source.
    fn new(source: &'a str) -> Self {
        Self {
            lexer: SfcLexer::new(source),
            source,
            errors: Vec::new(),
        }
    }

    /// Parse the SFC.
    fn parse(&mut self) -> ParseResult<Sfc> {
        let mut sfc = Sfc::new(self.source.to_string());

        while !self.lexer.is_eof() {
            self.lexer.skip_whitespace();

            if self.lexer.is_eof() {
                break;
            }

            // Try to parse a comment
            if self.lexer.starts_with("<!--") {
                if let Some(comment) = self.parse_comment() {
                    sfc.comments.push(comment);
                }
                continue;
            }

            // Try to parse a block
            if self.lexer.starts_with("<") && !self.lexer.starts_with("</") {
                self.parse_block(&mut sfc)?;
                continue;
            }

            // Skip any other content
            self.lexer.next_char();
        }

        Ok(sfc)
    }

    /// Parse a comment.
    fn parse_comment(&mut self) -> Option<Comment> {
        let start = self.lexer.pos();
        let content = self.lexer.read_comment()?;
        let span = self.lexer.span_from(start);
        Some(Comment {
            content: content.to_string(),
            span,
        })
    }

    /// Parse a block (template, script, style, or custom).
    fn parse_block(&mut self, sfc: &mut Sfc) -> ParseResult<()> {
        let start = self.lexer.pos();

        // Consume <
        if !self.lexer.consume("<") {
            return Ok(());
        }

        self.lexer.skip_whitespace();

        // Read tag name
        let tag_name = match self.lexer.read_tag_name() {
            Some(name) => name.to_lowercase(),
            None => {
                // Not a valid tag, skip
                return Ok(());
            }
        };

        // Parse attributes
        let attrs = self.parse_attributes()?;

        self.lexer.skip_whitespace();

        // Check for self-closing tag
        let is_self_closing = self.lexer.consume("/>");
        if !is_self_closing {
            self.lexer.consume(">");
        }

        let tag_end = self.lexer.pos();

        // Read content (if not self-closing)
        let (content, content_span) = if is_self_closing {
            (String::new(), Span::empty(tag_end as u32))
        } else {
            let content_start = self.lexer.pos();
            let content = self.lexer.read_block_content(&tag_name);
            let content_end = self.lexer.pos();
            (
                content.to_string(),
                Span::new(content_start as u32, content_end as u32),
            )
        };

        // Consume closing tag
        if !is_self_closing {
            self.lexer.skip_whitespace();
            let close_tag = format!("</{}", tag_name);
            if self
                .lexer
                .remaining()
                .to_lowercase()
                .starts_with(&close_tag)
            {
                self.lexer.consume(&format!("</{}", tag_name));
                // Also try with original case
                if !self.lexer.remaining().starts_with(">") {
                    self.lexer.consume_until(">");
                }
                self.lexer.consume(">");
            }
        }

        let end = self.lexer.pos();
        let span = Span::new(start as u32, end as u32);

        let block = SfcBlock {
            span,
            content_span,
            content: content.clone(),
            attrs: attrs.clone(),
        };

        // Create the appropriate block type
        match tag_name.as_str() {
            "template" => {
                if sfc.template.is_some() {
                    self.errors
                        .push(ParseError::duplicate_block("template", span));
                } else {
                    let lang = get_attr_value(&attrs, "lang").map(String::from);
                    let functional = has_attr(&attrs, "functional");
                    let src = get_src_attr(&attrs);

                    sfc.template = Some(TemplateBlock {
                        block,
                        lang,
                        functional,
                        src,
                    });
                }
            }
            "script" => {
                let is_setup = has_attr(&attrs, "setup");
                let lang = get_attr_value(&attrs, "lang").map(String::from);
                let src = get_src_attr(&attrs);

                if is_setup {
                    if sfc.script_setup.is_some() {
                        self.errors
                            .push(ParseError::duplicate_block("script setup", span));
                    } else {
                        let generic = get_attr_value(&attrs, "generic").map(String::from);
                        let generic_span = get_attr_value_span(&attrs, "generic");

                        sfc.script_setup = Some(ScriptSetupBlock {
                            block,
                            lang,
                            generic,
                            generic_span,
                        });
                    }
                } else if sfc.script.is_some() {
                    self.errors
                        .push(ParseError::duplicate_block("script", span));
                } else {
                    sfc.script = Some(ScriptBlock { block, lang, src });
                }
            }
            "style" => {
                let lang = get_attr_value(&attrs, "lang").map(String::from);
                let scoped = has_attr(&attrs, "scoped");
                let module = if has_attr(&attrs, "module") {
                    Some(
                        get_attr_value(&attrs, "module")
                            .map(String::from)
                            .unwrap_or_else(|| "$style".to_string()),
                    )
                } else {
                    None
                };
                let src = get_src_attr(&attrs);

                sfc.styles.push(StyleBlock {
                    block,
                    lang,
                    scoped,
                    module,
                    src,
                });
            }
            _ => {
                // Custom block
                sfc.custom_blocks.push(CustomBlock {
                    block,
                    block_type: tag_name.into(),
                });
            }
        }

        Ok(())
    }

    /// Parse attributes of a tag.
    fn parse_attributes(&mut self) -> ParseResult<Vec<BlockAttr>> {
        let mut attrs = Vec::new();

        loop {
            self.lexer.skip_whitespace();

            // Check for end of attributes
            if self.lexer.starts_with(">") || self.lexer.starts_with("/>") || self.lexer.is_eof() {
                break;
            }

            let attr_start = self.lexer.pos();

            // Read attribute name
            let name = match self.lexer.read_attr_name() {
                Some(n) => n,
                None => {
                    // Skip invalid character
                    self.lexer.next_char();
                    continue;
                }
            };

            self.lexer.skip_whitespace();

            // Check for value
            if self.lexer.consume("=") {
                self.lexer.skip_whitespace();

                // Read value
                let (value, value_span) =
                    if self.lexer.starts_with("\"") || self.lexer.starts_with("'") {
                        let value_start = self.lexer.pos() + 1; // After quote
                        if let Some((v, _quote)) = self.lexer.read_quoted_string() {
                            let value_end = self.lexer.pos() - 1; // Before closing quote
                            (
                                v.to_string(),
                                Span::new(value_start as u32, value_end as u32),
                            )
                        } else {
                            continue;
                        }
                    } else {
                        let value_start = self.lexer.pos();
                        let v = self.lexer.read_unquoted_value();
                        let value_end = self.lexer.pos();
                        (
                            v.to_string(),
                            Span::new(value_start as u32, value_end as u32),
                        )
                    };

                let span = self.lexer.span_from(attr_start);
                attrs.push(BlockAttr::with_value(name, value, span, value_span));
            } else {
                // Boolean attribute
                let span = self.lexer.span_from(attr_start);
                attrs.push(BlockAttr::boolean(name, span));
            }
        }

        Ok(attrs)
    }
}

// Helper functions

fn get_attr_value<'a>(attrs: &'a [BlockAttr], name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(name))
        .and_then(|a| a.value.as_deref())
}

fn get_attr_value_span(attrs: &[BlockAttr], name: &str) -> Option<Span> {
    attrs
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(name))
        .and_then(|a| a.value_span)
}

fn has_attr(attrs: &[BlockAttr], name: &str) -> bool {
    attrs.iter().any(|a| a.name.eq_ignore_ascii_case(name))
}

fn get_src_attr(attrs: &[BlockAttr]) -> Option<SrcAttr> {
    attrs
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("src"))
        .and_then(|a| {
            a.value.as_ref().map(|v| SrcAttr {
                value: v.clone(),
                span: a.span,
                value_span: a.value_span.unwrap_or(a.span),
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let sfc = parse_sfc("").unwrap();
        assert!(sfc.template.is_none());
        assert!(sfc.script.is_none());
        assert!(sfc.script_setup.is_none());
        assert!(sfc.styles.is_empty());
    }

    #[test]
    fn test_parse_template_only() {
        let source = "<template><div>Hello</div></template>";
        let sfc = parse_sfc(source).unwrap();
        assert!(sfc.template.is_some());
        let template = sfc.template.unwrap();
        assert_eq!(template.content.trim(), "<div>Hello</div>");
    }

    #[test]
    fn test_parse_script_setup() {
        let source = r#"<script setup lang="ts">
const msg = 'Hello'
</script>"#;
        let sfc = parse_sfc(source).unwrap();
        assert!(sfc.script_setup.is_some());
        let script = sfc.script_setup.unwrap();
        assert_eq!(script.lang.as_deref(), Some("ts"));
    }

    #[test]
    fn test_parse_script_with_generic() {
        let source = r#"<script setup lang="ts" generic="T extends string, U">
defineProps<{ value: T; other: U }>()
</script>"#;
        let sfc = parse_sfc(source).unwrap();
        let script = sfc.script_setup.unwrap();
        assert_eq!(script.generic.as_deref(), Some("T extends string, U"));
    }

    #[test]
    fn test_parse_multiple_styles() {
        let source = r#"<style scoped>
.foo { color: red; }
</style>
<style lang="scss" module>
.bar { color: blue; }
</style>"#;
        let sfc = parse_sfc(source).unwrap();
        assert_eq!(sfc.styles.len(), 2);
        assert!(sfc.styles[0].scoped);
        assert!(!sfc.styles[1].scoped);
        assert_eq!(sfc.styles[1].lang.as_deref(), Some("scss"));
        assert_eq!(sfc.styles[1].module.as_deref(), Some("$style"));
    }

    #[test]
    fn test_parse_custom_block() {
        let source = r#"<i18n lang="json">
{
  "en": { "hello": "Hello" }
}
</i18n>"#;
        let sfc = parse_sfc(source).unwrap();
        assert_eq!(sfc.custom_blocks.len(), 1);
        assert_eq!(sfc.custom_blocks[0].block_type.as_str(), "i18n");
    }

    #[test]
    fn test_parse_with_comments() {
        let source = r#"<!-- This is a comment -->
<template>
  <div>Hello</div>
</template>"#;
        let sfc = parse_sfc(source).unwrap();
        assert_eq!(sfc.comments.len(), 1);
        assert!(sfc.comments[0].content.contains("This is a comment"));
    }

    #[test]
    fn test_parse_script_and_script_setup() {
        let source = r#"<script lang="ts">
export interface Props {
  msg: string
}
</script>

<script setup lang="ts">
const props = defineProps<Props>()
</script>"#;
        let sfc = parse_sfc(source).unwrap();
        assert!(sfc.script.is_some());
        assert!(sfc.script_setup.is_some());
    }

    #[test]
    fn test_parse_style_module_named() {
        let source = r#"<style module="classes">
.foo { color: red; }
</style>"#;
        let sfc = parse_sfc(source).unwrap();
        assert_eq!(sfc.styles[0].module.as_deref(), Some("classes"));
    }

    #[test]
    fn test_parse_external_src() {
        let source = r#"<script src="./external.ts" lang="ts"></script>"#;
        let sfc = parse_sfc(source).unwrap();
        let script = sfc.script.unwrap();
        assert!(script.src.is_some());
        assert_eq!(script.src.unwrap().value, "./external.ts");
    }
}
