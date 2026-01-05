//! Vue template compiler.
//!
//! This crate provides template parsing and compilation for Vue components.
//! It parses Vue template syntax into an AST that can be used for type checking
//! and code generation.

pub mod ast;
pub mod error;
pub mod parser;
pub mod transforms;

pub use ast::*;
pub use error::{CompileError, CompileResult};
pub use parser::parse_template;

/// Compile a Vue template to AST.
pub fn compile(source: &str) -> CompileResult<TemplateAst> {
    parse_template(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_template() {
        let source = "<div>Hello {{ name }}</div>";
        let ast = compile(source).unwrap();
        assert_eq!(ast.children.len(), 1);
    }

    #[test]
    fn test_compile_with_directives() {
        let source = r#"<div v-if="show" v-for="item in items" :class="{ active: isActive }">
            {{ item.name }}
        </div>"#;
        let ast = compile(source).unwrap();
        assert_eq!(ast.children.len(), 1);
    }

    #[test]
    fn test_compile_slots() {
        let source = r#"<MyComponent>
            <template #default="{ item }">
                {{ item.name }}
            </template>
        </MyComponent>"#;
        let ast = compile(source).unwrap();
        assert_eq!(ast.children.len(), 1);
    }
}
