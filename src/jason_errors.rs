#[derive(Debug)]
pub struct JasonError {
    pub message: String,
    pub context: Vec<String>, // stack of function calls / nodes
}

pub type JasonResult<T> = Result<T, JasonError>;

impl JasonError {
    pub fn new(msg: impl Into<String>) -> Self {
        JasonError { message: msg.into(), context: Vec::new() }
    }

    // Add context to the stack
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }
}

