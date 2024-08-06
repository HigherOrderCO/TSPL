use highlight_error::{*};

#[macro_export]
macro_rules! new_parser {
  ($Parser:ident) => {
    pub struct $Parser<'i> {
      input: &'i str,
      index: usize,
    }

    impl<'i> Parser<'i> for $Parser<'i> {
      fn input(&mut self) -> &'i str {
        &self.input
      }

      fn index(&mut self) -> &mut usize {
        &mut self.index
      }
    }

    impl<'i> $Parser<'i> {
      pub fn new(input: &'i str) -> Self {
        Self { input, index: 0 }
      }
    }
  }
}

#[derive(Debug, Clone, Hash)]
pub struct ParseError {
  /// Byte-indexed span of the parsing error.
  /// Inclusive on the left and exclusive on the right.
  pub span: (usize, usize),
  /// Error message.
  pub message: String
}

impl ParseError {
  pub fn new(span: (usize, usize), message: impl Into<String>) -> Self {
    ParseError {
      span,
      message: message.into()
    }
  }
}

impl<'a> From<ParseError> for String {
  fn from(val: ParseError) -> Self {
    val.message
  }
}

impl<'a> std::fmt::Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.message.fmt(f)
  }
}

pub trait Parser<'i> {

  fn input(&mut self) -> &'i str;
  fn index(&mut self) -> &mut usize;

  /// Generates an error message for parsing failures, including the highlighted context.
  fn expected<T>(&mut self, exp: &str) -> Result<T, ParseError> {
    let span = (*self.index(), *self.index() + 1);
    let ctx = highlight_error(span.0, span.1, self.input());
    let msg = format!("\x1b[1mPARSE_ERROR\n- expected: \x1b[0m{}\x1b[1m\n- detected:\n\x1b[0m{}", exp, ctx);
    Err(ParseError::new(span, msg))
  }

  /// Generates an error message with an additional custom message.
  fn expected_and<T>(&mut self, exp: &str, msg: &str) -> Result<T, ParseError> {
    let span = (*self.index(), *self.index() + 1);
    let ctx = highlight_error(span.0, span.1, self.input());
    let msg = format!("\x1b[1mPARSE_ERROR\n- expected: \x1b[0m{}\x1b[1m\n- detected:\n\x1b[0m{}\x1b[1m\n - info:\n\x1b[0m{}", exp, ctx, msg);
    Err(ParseError::new(span, msg))
  }

  /// Inspects the next character in the text without consuming it.
  fn peek_one(&mut self) -> Option<char> {
    self.input().get(*self.index()..)?.chars().next()
  }

  /// Inspects the next `count` characters in the text without consuming them.
  fn peek_many(&mut self, count: usize) -> Option<&'i str> {
    let chars = self.input().get(*self.index()..)?.chars();
    let mut end_index = *self.index();
    for (i, c) in chars.enumerate().take(count) {
      if i + 1 == count {
        end_index += c.len_utf8();
        break;
      }
      end_index += c.len_utf8();
    }
    Some(&self.input()[*self.index()..end_index])
  }

  /// Consumes the next character in the text.
  fn advance_one(&mut self) -> Option<char> {
    let chr = self.peek_one()?;
    *self.index() += chr.len_utf8();
    Some(chr)
  }

  /// Advances the parser by `count` characters, consuming them.
  fn advance_many(&mut self, count: usize) -> Option<&'i str> {
    let result = self.peek_many(count)?;
    *self.index() += result.len();
    Some(result)
  }

  /// Skips spaces in the text.
  fn skip_spaces(&mut self) {
    while let Some(c) = self.peek_one() {
      if c.is_ascii_whitespace() {
        self.advance_one();
      } else {
        break;
      }
    }
  }

  /// Skips whitespace & comments in the text.
  fn skip_trivia(&mut self) {
    while let Some(c) = self.peek_one() {
      if c.is_ascii_whitespace() {
        self.advance_one();
        continue;
      }
      if c == '/' && self.input().get(*self.index()..).unwrap_or_default().starts_with("//") {
        while let Some(c) = self.peek_one() {
          if c != '\n' {
            self.advance_one();
          } else {
            break;
          }
        }
        self.advance_one(); // Skip the newline character as well
        continue;
      }
      break;
    }
  }

  /// Checks if the parser has reached the end of the input.
  fn is_eof(&mut self) -> bool {
    *self.index() >= self.input().len()
  }

  /// Consumes an instance of the given string, erroring if it is not found.
  fn consume(&mut self, text: &str) -> Result<(), ParseError> {
    self.skip_trivia();
    if self.input().get(*self.index()..).unwrap_or_default().starts_with(text) {
      *self.index() += text.len();
      Ok(())
    } else {
      self.expected(text)
    }
  }

  /// Checks if the next characters in the input start with the given string.
  fn starts_with(&mut self, text: &str) -> bool {
    self.peek_many(text.chars().count()).map_or(false, |s| s == text)
  }

  /// Consumes all contiguous characters matching a given predicate.
  fn take_while(&mut self, mut f: impl FnMut(char) -> bool) -> &'i str {
    let start = *self.index();
    while let Some(c) = self.peek_one() {
      if f(c) {
        self.advance_one();
      } else {
        break;
      }
    }
    let end = *self.index();
    &self.input()[start..end]
  }

  /// Parses a name from the input, supporting alphanumeric characters, underscores, periods, and hyphens.
  fn parse_name(&mut self) -> Result<String, ParseError> {
    self.skip_trivia();
    let name = self.take_while(|c| c.is_ascii_alphanumeric() || "_.-/$".contains(c));
    if name.is_empty() {
      self.expected("name")
    } else {
      Ok(name.to_owned())
    }
  }

  /// Parses a u64 from the input, supporting dec, hex (0xNUM), and bin (0bNUM).
  fn parse_u64(&mut self) -> Result<u64, ParseError> {
    self.skip_trivia();
    let radix = match self.peek_many(2) {
      Some("0x") => { self.advance_many(2); 16 },
      Some("0b") => { self.advance_many(2); 2 },
      _ => { 10 },
    };
    let num_str = self.take_while(move |c| c.is_digit(radix) || c == '_');
    let num_str = num_str.chars().filter(|c| *c != '_').collect::<String>();
    if num_str.is_empty() {
      self.expected("numeric digit")
    } else {
      u64::from_str_radix(&num_str, radix)
        .map_err(|e| self.expected_and::<u64>("integer", &e.to_string()).unwrap_err())
    }
  }

  /// Parses a single unicode character, supporting scape sequences.
  fn parse_char(&mut self) -> Result<char, ParseError> {
    match self.advance_one() {
      Some('\\') => match self.advance_one() {
        Some('u') => {
          self.consume("{")?;
          let codepoint_str = self.take_while(|c| c.is_digit(16));
          self.consume("}")?;
          u32::from_str_radix(codepoint_str, 16)
            .ok().and_then(std::char::from_u32)
            .ok_or_else(|| self.expected::<char>("unicode-codepoint").unwrap_err())
        }
        Some('0') => Ok('\0'),
        Some('n') => Ok('\n'),
        Some('r') => Ok('\r'),
        Some('t') => Ok('\t'),
        Some('\'') => Ok('\''),
        Some('\"') => Ok('\"'),
        Some('\\') => Ok('\\'),
        Some(chr) => self.expected(&format!("\\{}", chr)),
        None => self.expected("escaped-char"),
      },
      Some(other) => Ok(other),
      None => self.expected("char"),
    }
  }

  /// Parses a quoted character, like 'x'.
  fn parse_quoted_char(&mut self) -> Result<char, String> {
    self.skip_trivia();
    self.consume("'")?;
    let chr = self.parse_char()?;
    self.consume("'")?;
    Ok(chr)
  }

  /// Parses a quoted string, like "foobar".
  fn parse_quoted_string(&mut self) -> Result<String, ParseError> {
    self.skip_trivia();
    self.consume("\"")?;
    let mut result = String::new();
    while let Some(chr) = self.peek_one() {
      if chr == '"' {
        break;
      } else {
        result.push(self.parse_char()?);
      }
    }
    self.consume("\"")?;
    Ok(result)
  }

}
