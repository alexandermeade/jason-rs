//this token defintion is borrowed from my nyx language project

use crate::parser::Parser;
use crate::astnode::ASTNode;
use crate::jason_errors::{JasonError};

pub type Args = Vec<ASTNode>;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    ERR(String),
    Unknown(char),
    StringLiteral(String),
    CompositeString(Vec<String>, Vec<ASTNode>),
    IntLiteral(i64),
    BoolLiteral(bool),
    FloatLiteral(f64),
    Import(Args),
    Export(Args),
    FnCall(Args),
    LuaFnCall(Args),
    Map(Args),
    Index(Args),
    // input args, block args
    TemplateDef(Args, Args),
    //StructDef(Box<Token>, Args),
    Block(Args),
    List(Args),
    Tuple(Args),
    Path(String),
    ID,
    Template(Vec<Token>, Args),
    InnerType(String, Vec<Token>),
    Return,
    //special chars
    Bar,
    DoubleColon,
    Colon,
    Dot,
    //whitespace
    NewLine,
    Comma,
    //enclosing symbols
    OpenParen,
    OpenBracket,
    OpenCurly,
    ClosedParen,
    ClosedBracket,
    ClosedCurly,
    // Math Symbols
    Equals,
    Narwhal,
    SpiderWalrus,
    Plus,
    Minus,
    Mult,
    Divide,
    Mod, 
    StartComment,
    EndComment,
    EOF,
    EOT,
    //Logic Symbol
    LessThan,
    GreaterThan,
    LessThanEqualTo,
    GreaterThanEqualTo,
    // keywords
    StringConverion(Args),
    IntConverion(Args),
    FloatConverion(Args),
    UPick,
    Pick,
    From,
    At,
    Repeat,
    Append, 
    Unpack,
    FN,
    Let,
    AS,
    Out,
    Const,
    Type,
    StringType,
    NumberType,
    IntType,
    FloatType,
    BoolType,
    AnyType,
    NullType,
    Embed,
    Use(Args),
    Empty,
    Null,
    DollarSign,
    Merge,
    VarianceOperator, 
    With,
    While,
    Include,
    Info,
    InfoT,
    Auto(String)
}

impl TokenType {

    pub fn find_keyword(content:&str) -> TokenType {
        match content {
            "null" => TokenType::Null,
            "from" => TokenType::From,
            "as" => TokenType::AS,
            "String" => TokenType::StringType,
            "Number" => TokenType::NumberType,
            "Int"    => TokenType::IntType,
            "Float"  => TokenType::FloatType,
            "Bool"   => TokenType::BoolType,
            "Any"    => TokenType::AnyType,
            "Null"   => TokenType::NullType,
            "true" => TokenType::BoolLiteral(true),
            "false" => TokenType::BoolLiteral(false),
            "embed" => TokenType::Embed,
            "return" => TokenType::Return,
            "out" => TokenType::Out,
            "at" => TokenType::At,
            "upick" => TokenType::UPick,
            "pick" => TokenType::Pick,
            "repeat" => TokenType::Repeat,
            "append" => TokenType::Append,
            "with"   => TokenType::With,
            "while"  => TokenType::While,
            "info"   => TokenType::Info,
            "infoT"   => TokenType::InfoT,
            "include" => TokenType::Include,
            _ => TokenType::ID
        }
    }

    pub fn is_keyword(content:&str) -> bool {
        return Self::find_keyword(content) != TokenType::ID;
    }

    pub fn as_open_delim(&self) -> Option<char> {
        match self {
            TokenType::OpenParen => Some('('),
            TokenType::OpenBracket => Some('['),
            TokenType::OpenCurly => Some('{'),
            TokenType::LessThan => Some('<'),
            _ => None,
        }
    }
    
    pub fn as_close_delim(&self) -> Option<char> {
        match self {
            TokenType::ClosedParen => Some(')'),
            TokenType::ClosedBracket => Some(']'),
            TokenType::ClosedCurly => Some('}'),
            TokenType::GreaterThan => Some('>'),
            // These are "parsed" closing tokens (already collected their contents)
            TokenType::List(_) => Some(']'),
            TokenType::Block(_) => Some('}'),
            TokenType::Template(_, _) => Some('>'),
            _ => None,
        }
    }
    
    pub fn matches_open(&self, open: &TokenType) -> bool {
        match (open, self) {
            (TokenType::OpenParen, TokenType::ClosedParen) => true,
            (TokenType::OpenBracket, TokenType::ClosedBracket) => true,
            (TokenType::OpenCurly, TokenType::ClosedCurly) => true,
            (TokenType::LessThan, TokenType::GreaterThan) => true,
            _ => false,
        }
    }
    
    pub fn same_delim_type(&self, other: &TokenType) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
    
    pub fn delim_str(&self) -> &'static str {
        match self {
            TokenType::OpenParen | TokenType::ClosedParen => "()",
            TokenType::OpenBracket | TokenType::ClosedBracket | TokenType::List(_) => "[]",
            TokenType::OpenCurly | TokenType::ClosedCurly | TokenType::Block(_) => "{}",
            TokenType::LessThan | TokenType::GreaterThan | TokenType::Template(_, _) => "<>",
            _ => "?",
        }
    }

    //rust thinks this function isn't being called but is :/
    #[allow(dead_code)]
    pub fn is_err(&self) -> bool {
        return match self {
            TokenType::ERR(_) => true,
            _ => false,
        }
    }
      
    //rust thinks this function isn't being called but is :/
    #[allow(dead_code)]
    pub fn is_bound(&self) -> bool { 
        return match *self {
            TokenType::EOT |
            TokenType::EOF => true,
            _ => false,
        }
    }
    //rust thinks this function isn't being called but is :/
    #[allow(dead_code)]
    pub fn is_type(&self) -> bool {
        return *self == TokenType::StringType || 
               *self == TokenType::NumberType || 
               *self == TokenType::BoolType   ||
               matches!(*self, TokenType::InnerType(_, _)); 
    }

}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub plain: String,
    pub token_type: TokenType,
    pub row: usize,
    pub colmn: usize,
}

impl Token {
    pub fn new(token_type: TokenType, plain:String, row:usize, colmn:usize) -> Token {
        Token {
            token_type,
            plain,
            row,
            colmn
        }
    }

    pub fn find_fn_keyword(self) -> Self {
        let row = self.row;
        let colmn = self.colmn;
        let name:&str = &(self.plain().clone());
        let token_type = self.token_type.clone();

        if let TokenType::FnCall(args) = token_type {
            return match name {
                "import" => Token::new(TokenType::Import(args), name.to_string(), row, colmn),
                "export" => Token::new(TokenType::Export(args), name.to_string(), row, colmn),
                "str" => Token::new(TokenType::StringConverion(args), name.to_string(), row, colmn),
                "int" => Token::new(TokenType::IntConverion(args), name.to_string(), row, colmn),
                "float" => Token::new(TokenType::FloatConverion(args), name.to_string(), row, colmn),
                "map" => Token::new(TokenType::Map(args), name.to_string(), row, colmn),
                "use" => Token::new(TokenType::Use(args), name.to_string(), row, colmn),
                _ => self,
            } 
        }

        return self;
    }  

    pub fn is_fn_keyword(name: &str) -> bool {
        return match name {
            "import" |
            "export" |
            "str" |
            "int" |
            "float" |
            "map" |
            "use" => true,
            _ => false,
        } 
    }  



    pub fn plain(&self) -> String {
        self.plain.clone()
    }
    
    #[allow(dead_code)]
    pub fn is_err(&self) -> bool {
        self.token_type.is_err()
    }

    #[allow(dead_code)]
    pub fn is_bound(&self) -> bool {
        self.token_type.is_bound()
    }

    #[allow(dead_code)]
    pub fn empty() -> Token {
        Token {
            token_type: TokenType::Empty,
            plain: String::from(""),
            row: 0,
            colmn: 0
        }
    }

    pub fn build_composite_string(strings:&Vec<String>, args:&Vec<ASTNode>) -> String {
        let mut result: String = String::new();

        for (i, string) in strings.iter().enumerate() {
            
            let node_str = if i < args.len() {               
                let node = args.get(i).unwrap(); 
                &node.plain_sum
            }else {
                &String::new() 
            };

            result.push_str(string);
            if !node_str.is_empty() {
                result.push_str("{");
                result.push_str(&node_str);
                result.push_str("}");
            }

        }  

        result
    }

     pub fn pretty(&self) -> String {
        match &self.token_type {
            // ===== Literals =====
            TokenType::StringLiteral(s) => format!("{:?}", s), // adds quotes                                                   
            TokenType::CompositeString(s, a) => format!("${:?}", Self::build_composite_string(s, a)), // adds quotes
            TokenType::IntLiteral(_) => self.plain.clone(),
            TokenType::FloatLiteral(_) => self.plain.clone(),
            TokenType::BoolLiteral(b) => b.to_string(),
            TokenType::Null => "null".to_string(),
            
            // ===== Identifiers & Paths =====
            TokenType::ID => self.plain.clone(),
            TokenType::Path(p) => p.clone(),

            // ===== Function calls =====
            TokenType::FnCall(args)
            | TokenType::Map(args)
            | TokenType::Import(args)
            | TokenType::Export(args)
            | TokenType::StringConverion(args)
            | TokenType::IntConverion(args)
            | TokenType::FloatConverion(args)
            | TokenType::Use(args) => {
                let args_str = args.as_string_tuple();
                format!("{}{}", self.plain, args_str)
            }

            TokenType::LuaFnCall(args) => {
                let args_str = args.as_string_tuple();
                format!("{}{}!", self.plain, args_str)
            }

            // ===== Index (x[...]) =====
            TokenType::Index(args) => {
                let inside = args.as_string_list();
                format!("{}{}", self.plain, inside)
            }

            // ===== Template definitions =====
            TokenType::TemplateDef(input_args, block_args) => {
                let inputs = input_args.as_string_tuple();
                let blocks = block_args.as_string_tuple();
                format!("template {} {}", inputs, blocks)
            }

            // ===== Templates =====
            TokenType::Template(tokens, args) => {
                let name = tokens.iter()
                    .map(|t| t.pretty())
                    .collect::<Vec<_>>()
                    .join("");
                let args_str = args.as_string_tuple();
                format!("{}{}", name, args_str)
            }

            // ===== Composite structures =====
            TokenType::Block(args) => args.as_string_list().replace("[", "{ ").replace("]", " }"),
            TokenType::List(args) => args.as_string_list(),
            TokenType::Tuple(args) => args.as_string_tuple(),

            // ===== Inner generic type (Type<...>) =====
            TokenType::InnerType(name, toks) => {
                let inner = toks.iter()
                    .map(|t| t.pretty())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, inner)
            }

            // ===== Simple punctuation =====
            TokenType::Merge         => "&".to_string(),
            TokenType::Bar           => "|".to_string(),
            TokenType::DoubleColon   => "::".to_string(),
            TokenType::Colon         => ":".to_string(),
            TokenType::Dot           => ".".to_string(),
            TokenType::Comma         => ",".to_string(),
            TokenType::OpenParen     => "(".to_string(),
            TokenType::ClosedParen   => ")".to_string(),
            TokenType::OpenBracket   => "[".to_string(),
            TokenType::ClosedBracket => "]".to_string(),
            TokenType::OpenCurly     => "{".to_string(),
            TokenType::ClosedCurly   => "}".to_string(),
            TokenType::VarianceOperator      => "'".to_string(),
            // ===== Operators =====
            TokenType::Equals => "=".to_string(),
            TokenType::Narwhal => ":=".to_string(),
            TokenType::SpiderWalrus => "::=".to_string(),
            TokenType::Plus   => "+".to_string(),
            TokenType::Minus  => "-".to_string(),
            TokenType::Mult   => "*".to_string(),
            TokenType::Divide => "/".to_string(),
            TokenType::Mod    => "%".to_string(),

            // ===== Comparisons =====
            TokenType::LessThan      => "<".to_string(),
            TokenType::GreaterThan   => ">".to_string(),

            TokenType::LessThanEqualTo    => "<=".to_string(),
            TokenType::GreaterThanEqualTo => ">=".to_string(),

            // ===== Misc =====
            TokenType::Return     => "return".to_string(),
            TokenType::From       => "from".to_string(),
            TokenType::Repeat     => "repeat".to_string(),
            TokenType::Pick       => "pick".to_string(),
            TokenType::UPick      => "upick".to_string(),
            TokenType::At         => "at".to_string(),
            TokenType::Append     => "append".to_string(),
            TokenType::Unpack     => "unpack".to_string(),
            TokenType::FN         => "fn".to_string(),
            TokenType::Let        => "let".to_string(),
            TokenType::AS         => "as".to_string(),
            TokenType::Out        => "out".to_string(),
            TokenType::Const      => "const".to_string(),
            TokenType::Type       => "type".to_string(),
            TokenType::StringType => "String".to_string(),
            TokenType::NumberType => "Number".to_string(),
            TokenType::FloatType  => "Float".to_string(),
            TokenType::IntType    => "Int".to_string(),
            TokenType::BoolType   => "Bool".to_string(),
            TokenType::AnyType    => "Any".to_string(),
            TokenType::NullType   => "Null".to_string(),
            TokenType::Embed      => "embed".to_string(),
            TokenType::With       => "with".to_string(),
            TokenType::While      => "while".to_string(),
            TokenType::Info       => "info".to_string(),
            TokenType::InfoT      => "infoT".to_string(),
            TokenType::Include    => "include".to_string(),
            TokenType::Auto(var)  => format!("*{}", var),
            
            TokenType::DollarSign => "$".to_string(),

            // ===== Error / unknown =====
            TokenType::ERR(s) => format!("{}", s),
            TokenType::Unknown(c) => c.to_string(),

            // ===== Bounds / EOF =====
            TokenType::NewLine => "\n".to_string(),
            TokenType::StartComment => "/*".to_string(),
            TokenType::EndComment => "*/".to_string(),
            TokenType::EOF => "<EOF>".to_string(),
            TokenType::EOT => "<EOT>".to_string(),
            TokenType::Empty => "".to_string(),
        }
    }
}

pub trait ArgsToNode {
    fn as_string_list(&self) -> String;
    fn as_string_tuple(&self) -> String;
}

pub trait TokensToNode {
    fn to_nodes(&self) -> Result<Vec<ASTNode>, JasonError>;
}

impl TokensToNode for Vec<Vec<Token>> {
    fn to_nodes(&self) -> Result<Vec<ASTNode>, JasonError> {
        let mut nodes = Vec::new();

        for tokens in self.iter() {
            let filtered: Vec<Token> = tokens
                .iter()
                .filter(|token| token.token_type != TokenType::NewLine)
                .cloned()
                .collect();

            // Propagate error if parsing fails
            let mut parsed_nodes = Parser::start("".to_string().into(), filtered)?;
            nodes.append(&mut parsed_nodes);
        }

        Ok(nodes)
    }
}

impl ArgsToNode for Vec<ASTNode> {    
    fn as_string_list(&self) -> String {
        "[".to_string()
            + &self.iter()
                .map(|node| node.plain_sum.clone())
                .collect::<Vec<_>>()
                .join(", ")
            + "]"
    }
    
    fn as_string_tuple(&self) -> String {
        "(".to_string()
            + &self.iter()
                .map(|node| node.plain_sum.clone())
                .collect::<Vec<_>>()
                .join(", ")
            + ")"
    }
}

