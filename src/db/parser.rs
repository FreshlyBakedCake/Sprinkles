use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Literals
    Ident(String),
    Number(i64),

    // Delims
    LParen,
    RParen,
    LBrace,
    RBrace,
    DotDot,
    Comma,

    // Ops
    And,
    Or,
    Not,
    Eq,

    // Functions
    Unread,
    Last,
    Page,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' | '\n' | '\r' => continue,
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '{' => tokens.push(Token::LBrace),
            '}' => tokens.push(Token::RBrace),
            ',' => tokens.push(Token::Comma),
            '=' => tokens.push(Token::Eq),
            '!' => tokens.push(Token::Not),
            '&' => tokens.push(Token::And),
            '|' => tokens.push(Token::Or),
            '.' => {
                if chars.next() == Some('.') {
                    tokens.push(Token::DotDot);
                } else {
                    return Err("Expected '..'".into());
                }
            }
            '0'..='9' => {
                let mut num = c.to_string();
                while let Some(&d @ '0'..='9') = chars.peek() {
                    num.push(d);
                    chars.next();
                }
                tokens.push(Token::Number(num.parse().unwrap()));
            }
            'a'..='z' | 'A'..='Z' | '_' | '-' => {
                let mut ident = c.to_string();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let tok = match ident.as_str() {
                    "and" | "AND" => Token::And,
                    "or" | "OR" => Token::Or,
                    "not" | "NOT" => Token::Not,
                    "unread" => Token::Unread,
                    "last" => Token::Last,
                    "page" => Token::Page,
                    _ => Token::Ident(ident),
                };
                tokens.push(tok);
            }
            _ => return Err(format!("Unexpected character: {}", c)),
        }
    }

    Ok(tokens)
}

#[derive(Debug)]
enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),

    Range(i64, i64),

    AttrFilter(String, String),

    Unread,
    Last(i64),
    Page(i64, i64),
}

struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into(),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn pop(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        let tok = self.pop();
        if tok == Some(expected.clone()) {
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, tok))
        }
    }

    fn parse(&mut self) -> Result<Expr, String> {
        let expr = self.or_expr()?;
        Ok(expr)
    }

    fn or_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.and_expr()?;
        while let Some(Token::Or) | Some(Token::Ident(_)) = self.peek() {
            if let Some(Token::Ident(s)) = self.peek() {
                if s.to_lowercase() != "or" {
                    break;
                }
            }
            self.pop();
            let right = self.and_expr()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn and_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.not_expr()?;
        while let Some(Token::And) | Some(Token::Ident(_)) = self.peek() {
            if let Some(Token::Ident(s)) = self.peek() {
                if s.to_lowercase() != "and" {
                    break;
                }
            }
            self.pop(); // consume AND
            let right = self.not_expr()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn not_expr(&mut self) -> Result<Expr, String> {
        if let Some(Token::Not) = self.peek() {
            self.pop();
            let expr = self.not_expr()?;
            return Ok(Expr::Not(Box::new(expr)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::LBrace) => self.attr_filter(),
            Some(Token::Number(_)) => self.range(),
            Some(Token::Ident(_)) | Some(Token::Unread) | Some(Token::Last) | Some(Token::Page) => {
                self.function_call()
            }
            Some(Token::LParen) => {
                self.pop();
                let expr = self.parse()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            tok => Err(format!("Unexpected token {:?}", tok)),
        }
    }

    fn range(&mut self) -> Result<Expr, String> {
        let start = match self.pop() {
            Some(Token::Number(n)) => n,
            _ => return Err("Expected number for range start".into()),
        };
        self.expect(Token::DotDot)?;
        let end = match self.pop() {
            Some(Token::Number(n)) => n,
            _ => return Err("Expected number for range end".into()),
        };
        Ok(Expr::Range(start, end))
    }

    fn attr_filter(&mut self) -> Result<Expr, String> {
        self.expect(Token::LBrace)?;
        let attr = match self.pop() {
            Some(Token::Ident(s)) => s,
            _ => return Err("Expected attribute name".into()),
        };
        self.expect(Token::Eq)?;
        let value = match self.pop() {
            Some(Token::Ident(s)) => s,
            Some(Token::Number(n)) => n.to_string(),
            _ => return Err("Expected value after '='".into()),
        };
        self.expect(Token::RBrace)?;
        Ok(Expr::AttrFilter(attr, value))
    }

    fn function_call(&mut self) -> Result<Expr, String> {
        match self.pop() {
            Some(Token::Unread) => {
                // self.expect(Token::LParen)?;
                // self.expect(Token::RParen)?;
                return Ok(Expr::Unread);
            }
            Some(Token::Last) => {
                self.expect(Token::LParen)?;
                let (num, defaulted) = match self.pop() {
                    Some(Token::Number(n)) => (n, false),
                    Some(Token::RParen) => (1, true),
                    _ => return Err("Unclosed parenthesis when calling last".into()),
                };
                if !defaulted {
                    self.expect(Token::RParen)?;
                }
                return Ok(Expr::Last(num));
            }
            Some(Token::Page) => {
                self.expect(Token::LParen)?;
                let offset = match self.pop() {
                    Some(Token::Number(n)) => n,
                    _ => 0,
                };
                let per_page = if let Some(Token::Comma) = self.peek() {
                    self.pop();
                    match self.pop() {
                        Some(Token::Number(n)) => n,
                        _ => return Err("Expected per_page in page(offset, per_page)".into()),
                    }
                } else {
                    10
                };
                self.expect(Token::RParen)?;
                return Ok(Expr::Page(offset, per_page));
            }
            _ => return Err("Unknown function".into()),
        };
    }
}

struct SqlEmitter {
    params: Vec<String>,
}

impl SqlEmitter {
    fn new() -> Self {
        SqlEmitter { params: Vec::new() }
    }

    fn push(&mut self, s: String) -> () {
        self.params.push(s);
    }

    fn emit(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::And(l, r) => format!("({} AND {})", self.emit(l), self.emit(r)),
            Expr::Or(l, r) => format!("({} OR {})", self.emit(l), self.emit(r)),
            Expr::Not(e) => format!("(NOT {})", self.emit(e)),
            Expr::Range(a, b) => {
                self.push(a.to_string());
                self.push(b.to_string());
                "id BETWEEN ? AND ?".into()
            }
            Expr::AttrFilter(attr, value) => {
                self.push(value.clone());
                format!(
                    "EXISTS ( \
                        SELECT 1 \
                        FROM dbus_notifications d \
                        WHERE d.id = dbus_notification_id \
                            AND d.{} = ? \
                    )",
                    attr
                )
            }
            Expr::Unread => "closed = 0".into(),
            Expr::Last(n) => {
                self.push(n.to_string());
                "id IN (SELECT id FROM notifications ORDER BY id DESC LIMIT ?)".into()
            }
            Expr::Page(offset, per_page) => {
                self.push(offset.to_string());
                self.push(per_page.to_string());
                "id IN (SELECT id FROM notifications ORDER BY id DESC LIMIT ? OFFSET ?)".into()
            }
        }
    }
}

pub fn parse_query(query: &str) -> Result<(String, Vec<String>), String> {
    let tokens = tokenize(query)?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    let mut emitter = SqlEmitter::new();
    let sql = emitter.emit(&ast);
    Ok((sql, emitter.params))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_and() {
        let (sql, params) = parse_query("{urgent = 1} & unread()").unwrap();
        assert_eq!(sql, "(urgent = ? AND closed = 0)");
        assert_eq!(params, vec!["1"]);
    }

    #[test]
    fn range_and_page() {
        let (sql, params) = parse_query("100..200 & page(20,5)").unwrap();
        assert!(sql.contains("BETWEEN ? AND ?"));
        assert!(sql.contains("LIMIT ? OFFSET ?"));
        assert_eq!(params, vec!["100", "200", "20", "5"]);
    }
}
