use std::fmt;
use std::rc::Rc;
use crate::astnode::ASTNode;
use unicode_width::UnicodeWidthChar;

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
    UndefinedVariable(String),
    InvalidOperation,
    MissingNode,
    ConversionError,
    Custom,
    Bundle(Vec<JasonError>),
    ImportError,
    LuaError,
    LuaFnError(String),
    LexerError(String),
}

pub struct JasonError {
    pub kind: JasonErrorKind,
    pub node: Option<Rc<ASTNode>>,
    pub message: String,
    pub context: Vec<String>,
    pub file: Rc<String>,
}


fn highlight_string(text: &str, target: &str) -> String {
    if target.is_empty() || !text.contains(target) {
        return text.to_string();
    }

    let target = if target == "*ALL*" { text } else { target };

    let text_chars: Vec<char> = text.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    let mut highlight_line = String::new();
    let mut i = 0;

    while i < text_chars.len() {
        // Check if we're at the start of the target string
        if i + target_chars.len() <= text_chars.len() {
            let slice = &text_chars[i..i + target_chars.len()];
            if slice == &target_chars[..] && is_standalone(&text_chars, i, target_chars.len()) {
                // Found a standalone match - add carets for each character
                for ch in &target_chars {
                    if *ch == '\t' {
                        highlight_line.push('\t');
                    } else {
                        let width = UnicodeWidthChar::width(*ch).unwrap_or(1);
                        for _ in 0..width {
                            highlight_line.push('^');
                        }
                    }
                }
                i += target_chars.len();
                continue;
            }
        }

        // Not a match - add spacing
        let ch = text_chars[i];
        if ch == '\t' {
            highlight_line.push('\t');
        } else {
            let width = UnicodeWidthChar::width(ch).unwrap_or(1);
            for _ in 0..width {
                highlight_line.push(' ');
            }
        }
        i += 1;
    }

    format!("{}\n{}", text, highlight_line)
}

/// Checks if the match at position `start` with length `len` is standalone.
/// A match is standalone if it's not surrounded by alphanumeric characters.
/// Punctuation, symbols, and whitespace are allowed adjacent to the match.
fn is_standalone(chars: &[char], start: usize, len: usize) -> bool {
    let end = start + len;

    // Check character before the match
    let valid_before = if start == 0 {
        true
    } else {
        !chars[start - 1].is_alphanumeric()
    };

    // Check character after the match
    let valid_after = if end >= chars.len() {
        true
    } else {
        !chars[end].is_alphanumeric()
    };

    valid_before && valid_after
}




pub type JasonResult<T> = Result<T, JasonError>;

impl JasonError {
    pub fn new(kind: JasonErrorKind, file: Rc<String>, node: Option<Rc<ASTNode>>, msg: impl Into<String>) -> Self {
        Self {
            kind,
            node,
            message: msg.into(),
            context: Vec::new(),
            file: file,
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
            JasonErrorKind::UndefinedVariable(_) => "Undefined Variable",
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
            JasonErrorKind::LexerError(_) => "Lexer Error",
            JasonErrorKind::LuaFnError(_) => "Lua Function Error"
        }
    }
}

impl std::error::Error for JasonError {}

impl From<mlua::Error> for JasonError {
    fn from(err: mlua::Error) -> Self {
        JasonError::new(
            JasonErrorKind::LuaError,
            Rc::new("lua src".to_string()),
            None,
            format!("Lua error: {}", err),
        )
    }
}

impl std::fmt::Display for JasonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            JasonErrorKind::Bundle(errors) => {
                for e in errors {
                    writeln!(f, "{}", e)?;
                }
            }
            JasonErrorKind::LexerError(err) => {
                println!("{:>5}", err);
            }
            _ => {
                // Print error kind and message with line number if available
                if let Some(node) = &self.node {
                    writeln!(f, "{} in file {} on line {}: {}", self.kind_str(), self.file, node.token.row, self.message)?;
                } else {
                    writeln!(f, "{}: {}", self.kind_str(), self.message)?;
                }
            }
        }
        
    

        // If we have an AST node, print the reconstructed code line
        if let Some(node) = &self.node {
            let code_line = format!("{:>5} | {}", node.token.row, node.root().plain_sum.clone());
            match &self.kind {
                JasonErrorKind::ImportError => {
                    println!("{:>5}", highlight_string(&code_line, "*ALL*"));
                },

                JasonErrorKind::UndefinedVariable(var) => {
                    println!("{:>5}", highlight_string(&code_line, &var));
                },
                JasonErrorKind::LuaFnError(fn_name) => {
                    println!("{:>5}", highlight_string(&code_line, &fn_name));
                },
                JasonErrorKind::SyntaxError | JasonErrorKind::MissingKey | JasonErrorKind::MissingValue => { 
                    println!("{:>5}", highlight_string(&code_line, "*ALL*"));
                }
                _ => {}
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
