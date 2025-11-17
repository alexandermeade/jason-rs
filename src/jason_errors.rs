use std::fmt;
use std::rc::Rc;
use crate::astnode::ASTNode;

#[derive(Debug)]
pub enum JasonErrorKind {
    ParseError,
    SyntaxError,
    MissingValue,
    MissingKey,
    ValueError,
    TypeError,
    FileError,
    ContextError,
    UndefinedVariable,
    InvalidOperation,
    MissingNode,
    ConversionError,
    Custom,
    Bundle(Vec<JasonError>),
    ImportError,
    LuaError,
}

pub struct JasonError {
    pub kind: JasonErrorKind,
    pub node: Option<Rc<ASTNode>>,
    pub message: String,
    pub context: Vec<String>,
}

pub type JasonResult<T> = Result<T, JasonError>;

impl JasonError {
    pub fn new(kind: JasonErrorKind, node: Option<Rc<ASTNode>>, msg: impl Into<String>) -> Self {
        Self {
            kind,
            node,
            message: msg.into(),
            context: Vec::new(),
        }
    }
    
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }
    
    fn kind_str(&self) -> &str {
        match &self.kind {
            JasonErrorKind::ParseError => "Parse Error",
            JasonErrorKind::SyntaxError => "Syntax Error",
            JasonErrorKind::ValueError => "Value Error",
            JasonErrorKind::TypeError => "Type Error",
            JasonErrorKind::UndefinedVariable => "Undefined Variable",
            JasonErrorKind::InvalidOperation => "Invalid Operation",
            JasonErrorKind::MissingNode => "Missing Node",
            JasonErrorKind::ConversionError => "Conversion Error",
            JasonErrorKind::Custom => "Custom Error",
            JasonErrorKind::ImportError => "Import Error",
            JasonErrorKind::LuaError => "Lua Error",
            JasonErrorKind::Bundle(_) => "Multiple Errors",
            JasonErrorKind::FileError => "File Error",
            JasonErrorKind::ContextError => "Context Error",
            JasonErrorKind::MissingValue => "Missing Value",
            JasonErrorKind::MissingKey => "Missing Key",
        }
    }
}

impl std::error::Error for JasonError {}

impl From<mlua::Error> for JasonError {
    fn from(err: mlua::Error) -> Self {
        JasonError::new(
            JasonErrorKind::LuaError,
            None,
            format!("Lua error: {}", err),
        )
    }
}

impl std::fmt::Display for JasonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            JasonErrorKind::Bundle(errors) => {
                writeln!(f, "{}:", self.message)?;
                for e in errors {
                    writeln!(f, " - {}", e)?;
                }
                return Ok(());
            }
            _ => {
                // Print error kind and message with line number if available
                if let Some(node) = &self.node {
                    writeln!(f, "{} on line {}: {}", self.kind_str(), node.token.row, self.message)?;
                } else {
                    writeln!(f, "{}: {}", self.kind_str(), self.message)?;
                }
            }
        }
        
        // If we have an AST node, print the reconstructed code line
        if let Some(node) = &self.node {
            let code_line = node.root().plain_sum.clone();
            if !code_line.is_empty() {
                // Print line number with code
                writeln!(f, "{:>5} | {}", node.token.row, code_line)?;
            }
        }
        
        // Print context if any
        if !self.context.is_empty() {
            writeln!(f, "Context: {}", self.context.join(" -> "))?;
        }
        
        Ok(())
    }
}

impl std::fmt::Debug for JasonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
