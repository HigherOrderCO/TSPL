use TSPL::Parser;
use std::fmt;

enum Term {
  Lam { name: String, body: Box<Term> },
  App { func: Box<Term>, argm: Box<Term> },
  Var { name: String },
}

TSPL::new_parser!(TermParser);

impl<'i> TermParser<'i> {
  fn parse(&mut self) -> Result<Term, String> {
    self.skip_trivia();
    match self.peek_one() {
      Some('λ') => {
        self.advance_one();
        let name = self.parse_name()?;
        self.skip_trivia();
        let body = Box::new(self.parse()?);
        Ok(Term::Lam { name, body })
      }
      Some('(') => {
        self.consume("(")?;
        let func = Box::new(self.parse()?);
        self.skip_trivia();
        let argm = Box::new(self.parse()?);
        self.consume(")")?;
        Ok(Term::App { func, argm })
      }
      _ => {
        let name = self.parse_name()?;
        Ok(Term::Var { name })
      }
    }
  }
}

impl fmt::Debug for Term {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Term::Lam { name, body } => write!(f, "λ{} {:?}", name, body),
      Term::App { func, argm } => write!(f, "({:?} {:?})", func, argm),
      Term::Var { name } => write!(f, "{}", name),
    }
  }
}

fn main() {
  let mut parser = TermParser::new("λx(λy(x y) λz z)");
  match parser.parse() {
    Ok(term) => println!("Parsed: {:?}", term),
    Err(err) => eprintln!("{}", err),
  }
}

