//! Code generation context.

use crate::{CodegenError, CodegenOptions, MacroInfo, ScriptLang};
use rustc_hash::FxHashSet;
use smol_str::SmolStr;

/// Context for code generation.
#[derive(Debug, Clone)]
pub struct CodegenContext {
    /// Code generation options.
    pub options: CodegenOptions,
    /// The script language.
    pub lang: ScriptLang,
    /// Generic type parameters.
    pub generics: Option<String>,
    /// Macro information.
    pub macros: MacroInfo,
    /// Variables in scope.
    pub scope_vars: Vec<ScopeVar>,
    /// Component imports.
    pub components: FxHashSet<SmolStr>,
    /// Directive imports.
    pub directives: FxHashSet<SmolStr>,
    /// Errors during code generation.
    pub errors: Vec<CodegenError>,
    /// Counter for generating unique names.
    pub counter: u32,
}

/// A variable in the current scope.
#[derive(Debug, Clone)]
pub struct ScopeVar {
    /// Variable name.
    pub name: SmolStr,
    /// Source of the variable (e.g., "props", "v-for", "slot").
    pub source: VarSource,
}

/// Source of a scope variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarSource {
    /// From props.
    Props,
    /// From setup return.
    Setup,
    /// From v-for loop.
    VFor,
    /// From slot props.
    SlotProps,
    /// From import.
    Import,
    /// Built-in.
    Builtin,
}

impl CodegenContext {
    /// Create a new code generation context.
    pub fn new(options: CodegenOptions) -> Self {
        Self {
            options,
            lang: ScriptLang::default(),
            generics: None,
            macros: MacroInfo::default(),
            scope_vars: Vec::new(),
            components: FxHashSet::default(),
            directives: FxHashSet::default(),
            errors: Vec::new(),
            counter: 0,
        }
    }

    /// Generate a unique identifier.
    pub fn unique_id(&mut self, prefix: &str) -> String {
        self.counter += 1;
        format!("{}{}", prefix, self.counter)
    }

    /// Add a scope variable.
    pub fn add_var(&mut self, name: impl Into<SmolStr>, source: VarSource) {
        self.scope_vars.push(ScopeVar {
            name: name.into(),
            source,
        });
    }

    /// Check if a variable is in scope.
    pub fn has_var(&self, name: &str) -> bool {
        self.scope_vars.iter().any(|v| v.name == name)
    }

    /// Get a variable's source.
    pub fn get_var_source(&self, name: &str) -> Option<VarSource> {
        self.scope_vars
            .iter()
            .rev()
            .find(|v| v.name == name)
            .map(|v| v.source)
    }

    /// Enter a new scope, returning a marker.
    pub fn enter_scope(&mut self) -> usize {
        self.scope_vars.len()
    }

    /// Exit a scope, removing variables added since the marker.
    pub fn exit_scope(&mut self, marker: usize) {
        self.scope_vars.truncate(marker);
    }

    /// Record a component usage.
    pub fn use_component(&mut self, name: impl Into<SmolStr>) {
        self.components.insert(name.into());
    }

    /// Record a directive usage.
    pub fn use_directive(&mut self, name: impl Into<SmolStr>) {
        self.directives.insert(name.into());
    }

    /// Add an error.
    pub fn error(&mut self, message: impl Into<String>, span: source_map::Span) {
        self.errors.push(CodegenError {
            message: message.into(),
            span,
        });
    }

    /// Check if using TypeScript.
    pub fn is_typescript(&self) -> bool {
        self.lang.is_typescript()
    }
}
