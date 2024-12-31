
#[derive(Debug, Clone)]
struct ParseError;

struct Parser {
    strict: bool,
    last_error: usize,
    look_ch: Option<char>,
}

#[derive(PartialEq)]
enum Token {
    EOS,
    EQ,
    NE,
    LT,
    GT,
    LE,
    GE,
    Exact,
    PrefixName,
    SimpleString,
    And,
    Or,
    Not,
    Prox,
    Sortby,
}

impl Parser {
    fn new() -> Parser {
        Parser {
            strict: false,
            last_error: 0,
            look_ch: None
        }
    }

    fn next(self: &mut Self, get: &mut dyn Iterator<Item = char>) {
        self.look_ch = get.next();
    }

    fn lex(self: &mut Self, get: &mut dyn Iterator<Item = char>) -> Result<Token, ParseError> {
        while let Some(ch) = self.look_ch {
            println!("ch = {ch}");
            if ch != ' ' && ch != '\t' && ch != '\r' && ch != '\n' {
                break
            }
            self.next(get);
        }
        if let Some(ch) = self.look_ch {
            if ch == '=' {
                self.look_ch = get.next();
                if let Some(ch) = self.look_ch {
                    if ch == '=' {
                        self.next(get);
                        return Ok(Token::Exact);
                    }
                }
                return Ok(Token::EQ);
            }
            return Err(ParseError);
        }
        Ok(Token::EOS)
    }

    fn parse(self: &mut Self, get: &mut dyn Iterator<Item = char>) -> Result<(), ParseError> {
        self.next(get);
        let tok = self.lex(get)?;
        if tok != Token::EOS {
            return Err(ParseError);
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use std::borrow::BorrowMut;

    use super::*;

    #[test]
    fn create_parser() {
        let my_sc = Parser::new();
        assert!(!my_sc.strict);
        assert_eq!(0, my_sc.last_error);
    }

    #[test]
    fn lex() {
        let mut my = Parser::new();
        let mut it = "= ==".chars();
        my.next(it.borrow_mut());
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::EQ));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Exact));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::EOS));

    }

    #[test]
    fn empty() {
        let s = String::from("  ");
        let mut my = Parser::new();
        let res = my.parse(s.chars().borrow_mut());
        assert!(res.is_ok());
    }
}
