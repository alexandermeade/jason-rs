use std::fmt;
use crate::astnode::ASTNode;

#[derive(Debug)]
pub enum JasonErrorKind {
    ParseError,
    SyntaxError,
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
    pub node: Option<Box<ASTNode>>,
    pub message: String,
    pub context: Vec<String>,
}

pub type JasonResult<T> = Result<T, JasonError>;

impl JasonError {
    pub fn new(kind: JasonErrorKind, node: Option<Box<ASTNode>>, msg: impl Into<String>) -> Self {
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
        }
    }
}
/*
impl fmt::Display for JasonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            JasonErrorKind::Bundle(errors) => {
                writeln!(f, "{}:", self.message)?;
                for e in errors {
                    for line in format!("{}", e).lines() {
                        writeln!(f, "  {}", line)?; // indent each sub-error
                    }
                }
                return Ok(());
            }
            _ => {
                writeln!(f, "{}: {}", self.kind_str(), self.message)?;
            }
        }

        // Print node info if present
        if let Some(node) = &self.node {
            if node.token.token_type != crate::token::TokenType::Empty {
                let code_line = node.to_code();
                writeln!(f, "{}", code_line)?;

                // caret under the token column
                let mut marker = String::new();
                for _ in 0..node.token.colmn {
                    marker.push(' ');
                }
                marker.push('^');
                writeln!(f, "{}", marker)?;
            }
        }

        // Print context if any
        if !self.context.is_empty() {
            writeln!(f, "Context: {}", self.context.join(" -> "))?;
        }

        Ok(())
    }
}*/

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
            _ => writeln!(f, "{}: {}", self.kind_str(), self.message)?,
        }

        // If we have an AST node, print a simple reconstruction line
        if let Some(node) = &self.node {
            let code_line = node.to_code();
            if !code_line.is_empty() {
                writeln!(f, "{}", code_line)?;
                // Place caret under the token
                let col = node.token.colmn.saturating_sub(1); // zero-based
                writeln!(f, "{:>width$}^", "", width = col)?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Debug for JasonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

