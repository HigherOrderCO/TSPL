# The Simplest Parser Library (TSPL)

TSPL is the The Simplest Parser Library that works in Rust.

## Concept

In pure functional languages like Haskell, a Parser can be represented as a function:

```
Parser<A> ::= String -> Reply<A, Error>
```

This allows us to implement a Monad instance for `Parser<A>`, letting us use the `do-notation` to
create simple and elegant parsers for our own types. Sadly, Rust doesn't have an equivalent. Yet,
we can easily emulate it by:

1. Using structs and `impl` to manage the cursor state internally.

2. Returning a `Result`, which allows us to use Rust's `?` to emulate monadic blocks.

This library merely exposes some functions to implement parsers that way, and nothing else.

## Example

As an example, let's create a λ-Term parser using TSPL.

1. Implement the type you want to create a parser for.

```rust
enum Term {
  Lam { name: String, body: Box<Term> },
  App { func: Box<Term>, argm: Box<Term> },
  Var { name: String },
}
```

2. Define your grammar. We'll use the following:

```
<term> ::= <lam> | <app> | <var>
<lam>  ::= "λ" <name> " " <term>
<app>  ::= "(" <func> " " <argm> ")"
<var>  ::= alphanumeric_string
```

3. Create a new Parser with the `new_parser()!` macro.

```rust
TSPL::new_parser!(TermParser);
```

4. Create an `impl Parser` for `TermParser`, with your grammar:

```rust
impl<'i> Parser<'i> for TermParser<'i> {
  fn parse(&mut self) -> Result<Term, String> {
    self.skip_trivia();
    match self.peek_one() {
      Some('λ') => {
        self.advance_one();
        let name = self.parse_name()?;
        let body = Box::new(self.parse()?);
        Ok(Term::Lam { name, body })
      }
      Some('(') => {
        self.consume("(")?;
        let func = Box::new(self.parse()?);
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
```

5. Use your parser!

```rust
fn main() {
  let mut parser = TermParser::new("λx(λy(x y) λz z)");
  match parser.parse() {
    Ok(term) => println!("{:?}", term),
    Err(err) => eprintln!("{}", err),
  }
}
```

The complete example is available in [./examples/lambda_term.rs](./examples/lambda_term.rs). Run it with:

```
cargo run --example lambda_term
```

## Credit

This design is based on T6's new parser for
[HVM-Core](https://github.com/HigherOrderCO/HVM-Core), and is much cleaner than
the old [HOPA](https://github.com/HigherOrderCO/HOPA) approach.
