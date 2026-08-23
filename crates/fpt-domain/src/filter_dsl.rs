use fpt_core::{AppError, Result};
use serde_json::{Number, Value, json};

#[derive(Debug, Clone, PartialEq)]
enum LogicalOp {
    And,
    Or,
}

impl LogicalOp {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::And => "and",
            Self::Or => "or",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Comparison {
        field: String,
        operator: String,
        value: Value,
    },
    Logical {
        op: LogicalOp,
        conditions: Vec<Self>,
    },
}

pub fn parse_filter_dsl(input: &str) -> Result<Value> {
    let source = input.trim();
    if source.is_empty() {
        return Err(AppError::invalid_input("`filter_dsl` cannot be empty"));
    }

    let mut parser = Parser::new(source);
    let expr = parser.parse_expression()?;
    parser.skip_whitespace();
    if !parser.is_eof() {
        return parser.error("unexpected trailing content");
    }

    Ok(wrap_root(expr))
}

fn wrap_root(expr: Expr) -> Value {
    match expr {
        Expr::Logical { .. } => expr_to_value(expr),
        value => json!({
            "logical_operator": "and",
            "conditions": [expr_to_value(value)],
        }),
    }
}

fn expr_to_value(expr: Expr) -> Value {
    match expr {
        Expr::Comparison {
            field,
            operator,
            value,
        } => json!([field, operator, value]),
        Expr::Logical { op, conditions } => json!({
            "logical_operator": op.as_str(),
            "conditions": conditions.into_iter().map(expr_to_value).collect::<Vec<_>>(),
        }),
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr> {
        self.parse_binary_chain("or", LogicalOp::Or, Self::parse_and)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        self.parse_binary_chain("and", LogicalOp::And, Self::parse_primary)
    }

    /// Parse a left-associative chain of `keyword`-separated sub-expressions.
    ///
    /// If only one sub-expression is found the node is returned as-is;
    /// otherwise they are wrapped in a `Logical` node with the given `op`.
    fn parse_binary_chain(
        &mut self,
        keyword: &str,
        op: LogicalOp,
        mut sub_parser: impl FnMut(&mut Self) -> Result<Expr>,
    ) -> Result<Expr> {
        let mut terms = vec![sub_parser(self)?];

        loop {
            let checkpoint = self.pos;
            if self.consume_keyword(keyword) {
                terms.push(sub_parser(self)?);
            } else {
                self.pos = checkpoint;
                break;
            }
        }

        if terms.len() == 1 {
            Ok(terms.remove(0))
        } else {
            Ok(Expr::Logical {
                op,
                conditions: terms,
            })
        }
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        self.skip_whitespace();
        if self.consume_char('(') {
            let expr = self.parse_expression()?;
            self.skip_whitespace();
            if !self.consume_char(')') {
                return self.error("missing closing parenthesis `)`");
            }
            return Ok(expr);
        }

        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let field = self.parse_field_path()?;
        let operator = self.parse_operator()?;
        let value = self.parse_value()?;

        Ok(Expr::Comparison {
            field,
            operator,
            value,
        })
    }

    fn parse_field_path(&mut self) -> Result<String> {
        let mut path = String::new();
        path.push_str(&self.parse_identifier("field name")?);

        loop {
            self.skip_whitespace();
            if !self.consume_char('.') {
                break;
            }
            path.push('.');
            path.push_str(&self.parse_identifier("field name")?);
        }

        Ok(path)
    }

    fn parse_operator(&mut self) -> Result<String> {
        self.skip_whitespace();

        if self.consume_literal("==") {
            return Ok("is".to_string());
        }
        if self.consume_literal("!=") {
            return Ok("is_not".to_string());
        }
        if self.consume_literal(">=") {
            return Ok("greater_than_or_equal".to_string());
        }
        if self.consume_literal("<=") {
            return Ok("less_than_or_equal".to_string());
        }
        if self.consume_literal(">") {
            return Ok("greater_than".to_string());
        }
        if self.consume_literal("<") {
            return Ok("less_than".to_string());
        }
        if self.consume_literal("~") {
            return Ok("contains".to_string());
        }

        let keyword = self.parse_identifier("operator")?.to_ascii_lowercase();
        if keyword == "not" {
            let checkpoint = self.pos;
            if self.consume_keyword("in") {
                return Ok("not_in".to_string());
            }
            self.pos = checkpoint;
        }

        Ok(keyword)
    }

    fn parse_value(&mut self) -> Result<Value> {
        self.skip_whitespace();
        let Some(ch) = self.peek_char() else {
            return self.error("missing operand");
        };

        match ch {
            '"' | '\'' => self.parse_string().map(Value::String),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => {
                let checkpoint = self.pos;
                let ident = self.parse_identifier("value")?;
                let lower = ident.to_ascii_lowercase();
                match lower.as_str() {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    "null" => Ok(Value::Null),
                    _ => {
                        // Entity-link shorthand: `EntityType:id` → {"type": "EntityType", "id": id}
                        // Example: `Project:123` → {"type": "Project", "id": 123}
                        self.skip_whitespace();
                        if self.consume_char(':') {
                            let id = self.parse_number()?;
                            return Ok(json!({"type": ident, "id": id}));
                        }

                        self.pos = checkpoint;
                        Err(AppError::invalid_input(format!(
                            "filter_dsl syntax error: unsupported value `{ident}` (position {}); expected a string, number, bool, null, JSON object, or entity-link shorthand like `Project:123`",
                            self.pos
                        ))
                        .with_hint(format!(
                            "Wrap string values in quotes: '{ident}'. For entity-links, use `EntityType:id` shorthand (e.g. `Project:123`) or a full JSON object `{{\"type\": \"Project\", \"id\": 123}}`."
                        )))
                    }
                }
            }
        }
    }

    /// Parse a JSON object literal `{...}` for use as an entity-link value.
    ///
    /// Supports the ShotGrid entity-link shorthand:
    ///   `project is {"type": "Project", "id": 123}`
    fn parse_object(&mut self) -> Result<Value> {
        self.expect_char('{', "opening `{` of object")?;
        self.skip_whitespace();

        let mut map = serde_json::Map::new();
        if self.consume_char('}') {
            return Ok(Value::Object(map));
        }

        loop {
            self.skip_whitespace();
            // key must be a quoted string
            let key = match self.peek_char() {
                Some('"') | Some('\'') => self.parse_string()?,
                _ => {
                    return self.error(
                        "JSON object keys must be quoted strings, e.g. {\"type\": \"Project\", \"id\": 123}",
                    );
                }
            };
            self.skip_whitespace();
            if !self.consume_char(':') {
                return self.error("missing `:` between JSON object key and value");
            }
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if self.consume_char('}') {
                break;
            }
            if !self.consume_char(',') {
                return self.error("missing `,` or closing `}` in JSON object");
            }
        }

        Ok(Value::Object(map))
    }

    fn parse_array(&mut self) -> Result<Value> {
        self.expect_char('[', "opening `[` of array")?;
        self.skip_whitespace();

        let mut values = Vec::new();
        if self.consume_char(']') {
            return Ok(Value::Array(values));
        }

        loop {
            values.push(self.parse_value()?);
            self.skip_whitespace();

            if self.consume_char(']') {
                break;
            }
            self.expect_char(',', "array separator `,`")?;
        }

        Ok(Value::Array(values))
    }

    fn parse_number(&mut self) -> Result<Value> {
        self.skip_whitespace();
        let mut literal = String::new();

        if self.consume_char('-') {
            literal.push('-');
        }

        let mut has_integer = false;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                has_integer = true;
                literal.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }

        if !has_integer {
            return self.error("invalid number format");
        }

        if self.consume_char('.') {
            literal.push('.');
            let mut has_fraction = false;
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    has_fraction = true;
                    literal.push(ch);
                    self.pos += 1;
                } else {
                    break;
                }
            }

            if !has_fraction {
                return self.error("invalid float format");
            }

            let value = literal
                .parse::<f64>()
                .map_err(|error| AppError::invalid_input(format!("number parse error: {error}")))?;
            let number = Number::from_f64(value)
                .ok_or_else(|| AppError::invalid_input("NaN and Infinity are not supported"))?;
            return Ok(Value::Number(number));
        }

        let value = literal
            .parse::<i64>()
            .map_err(|error| AppError::invalid_input(format!("number parse error: {error}")))?;
        Ok(Value::Number(Number::from(value)))
    }

    fn parse_string(&mut self) -> Result<String> {
        self.skip_whitespace();
        let quote = self.peek_char().ok_or_else(|| {
            AppError::invalid_input("filter_dsl syntax error: missing opening quote for string")
        })?;
        if !matches!(quote, '"' | '\'') {
            return self.error("string must be wrapped in quotes");
        }
        self.pos += 1;

        let mut output = String::new();
        while let Some(ch) = self.peek_char() {
            self.pos += 1;
            if ch == quote {
                return Ok(output);
            }

            if ch == '\\' {
                let escaped = self.peek_char().ok_or_else(|| {
                    AppError::invalid_input(
                        "filter_dsl syntax error: illegal escape at end of input",
                    )
                })?;
                self.pos += 1;
                let normalized = match escaped {
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    other => {
                        return Err(AppError::invalid_input(format!(
                            "filter_dsl syntax error: unsupported escape character `\\{other}` (position {})",
                            self.pos
                        )));
                    }
                };
                output.push(normalized);
                continue;
            }

            output.push(ch);
        }

        self.error("missing closing quote for string")
    }

    fn parse_identifier(&mut self, label: &str) -> Result<String> {
        self.skip_whitespace();

        let Some(first) = self.peek_char() else {
            return self.error(format!("missing {label}"));
        };

        if !is_identifier_start(first) {
            return self.error(format!("{label} must start with a letter or underscore"));
        }

        let mut output = String::new();
        output.push(first);
        self.pos += 1;

        while let Some(ch) = self.peek_char() {
            if is_identifier_continue(ch) {
                output.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }

        Ok(output)
    }

    fn expect_char(&mut self, expected: char, message: &str) -> Result<()> {
        self.skip_whitespace();
        if self.consume_char(expected) {
            Ok(())
        } else {
            self.error(format!("missing {message}"))
        }
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        self.skip_whitespace();
        let checkpoint = self.pos;

        for expected in keyword.chars() {
            match self.peek_char() {
                Some(ch) if ch.eq_ignore_ascii_case(&expected) => self.pos += 1,
                _ => {
                    self.pos = checkpoint;
                    return false;
                }
            }
        }

        if self.peek_char().is_some_and(is_identifier_continue) {
            self.pos = checkpoint;
            return false;
        }

        true
    }

    fn consume_literal(&mut self, literal: &str) -> bool {
        self.skip_whitespace();
        let checkpoint = self.pos;
        for expected in literal.chars() {
            match self.peek_char() {
                Some(ch) if ch == expected => self.pos += 1,
                _ => {
                    self.pos = checkpoint;
                    return false;
                }
            }
        }
        true
    }

    fn skip_whitespace(&mut self) {
        while self.peek_char().is_some_and(|ch| ch.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.pos += 1;
            return true;
        }
        false
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn error<T>(&self, message: impl Into<String>) -> Result<T> {
        Err(AppError::invalid_input(format!(
            "filter_dsl syntax error: {} (position {})",
            message.into(),
            self.pos
        )))
    }
}

const fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

const fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
