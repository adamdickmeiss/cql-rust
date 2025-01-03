#[derive(Debug, Clone)]
struct ParseError;

struct Parser {
    strict: bool,
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
    Modifier,
    LP,
    RP,
    PrefixName(String),
    SimpleString(String),
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
            look_ch: None,
        }
    }

    fn next(self: &mut Self, get: &mut dyn Iterator<Item = char>) {
        self.look_ch = get.next();
    }

    pub(crate) fn lex(
        self: &mut Self,
        get: &mut dyn Iterator<Item = char>,
    ) -> Result<Token, ParseError> {
        while let Some(ch) = self.look_ch {
            println!("ch = {ch}");
            if ch != ' ' && ch != '\t' && ch != '\r' && ch != '\n' {
                break;
            }
            self.next(get);
        }
        let ch = match self.look_ch {
            Some(ch1) => ch1,
            None => return Ok(Token::EOS),
        };
        match ch {
            '=' => {
                self.next(get);
                if let Some(ch) = self.look_ch {
                    if ch == '=' {
                        self.next(get);
                        return Ok(Token::Exact);
                    }
                }
                return Ok(Token::EQ);
            }
            '>' => {
                self.next(get);
                if let Some(ch) = self.look_ch {
                    if ch == '=' {
                        self.next(get);
                        return Ok(Token::GE);
                    }
                }
                return Ok(Token::GT);
            }
            '<' => {
                self.next(get);
                if let Some(ch) = self.look_ch {
                    if ch == '=' {
                        self.next(get);
                        return Ok(Token::LE);
                    }
                    if ch == '>' {
                        self.next(get);
                        return Ok(Token::NE);
                    }
                }
                return Ok(Token::LT);
            }
            '/' => {
                self.next(get);
                return Ok(Token::Modifier);
            }
            '(' => {
                self.next(get);
                return Ok(Token::LP);
            }
            ')' => {
                self.next(get);
                return Ok(Token::RP);
            }
            '"' => {
                self.next(get);
                let mut s = String::new();
                while let Some(ch) = self.look_ch {
                    if ch == '"' {
                        break;
                    }
                    s.push(ch);
                    self.next(get);
                    if ch == '\\' {
                        if let Some(ch1) = self.look_ch {
                            s.push(ch1);
                            self.next(get);
                        }
                    }
                }
                return Ok(Token::SimpleString(s));
            }
            _ => {
                let mut s = String::new();
                let mut relation_like = self.strict;
                while let Some(ch) = self.look_ch {
                    if " \n()=<>/".find(ch).is_some() {
                        break;
                    }
                    if ch == '.' {
                        relation_like = true;
                    }
                    s.push(ch);
                    self.next(get);
                    if ch == '\\' {
                        if let Some(ch1) = self.look_ch {
                            s.push(ch1);
                            self.next(get);
                        }
                    }
                }
                if s.eq_ignore_ascii_case("and") {
                    return Ok(Token::And);
                }
                if s.eq_ignore_ascii_case("or") {
                    return Ok(Token::Or);
                }
                if s.eq_ignore_ascii_case("not") {
                    return Ok(Token::Not);
                }
                if s.eq_ignore_ascii_case("prox") {
                    return Ok(Token::Prox);
                }
                if s.eq_ignore_ascii_case("sortby") {
                    return Ok(Token::Sortby);
                }
                if s.eq_ignore_ascii_case("all")
                    || s.eq_ignore_ascii_case("any")
                    || s.eq_ignore_ascii_case("adj")
                {
                    relation_like = true;
                }
                if relation_like {
                    return Ok(Token::PrefixName(s));
                }
                return Ok(Token::SimpleString(s));
            }
        }
    }

    pub fn parse(self: &mut Self, get: &mut dyn Iterator<Item = char>) -> Result<(), ParseError> {
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
    use super::*;
    use std::borrow::BorrowMut;

    #[test]
    fn create_parser() {
        let my_sc = Parser::new();
        assert!(!my_sc.strict);
    }

    #[test]
    fn lex_ops() {
        let mut my = Parser::new();
        let mut it = "= == > >= < <= <>/()".chars();
        my.next(it.borrow_mut());
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::EQ));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Exact));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::GT));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::GE));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::LT));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::LE));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::NE));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Modifier));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::LP));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::RP));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::EOS));

        let mut it = "=".chars();
        my.next(it.borrow_mut());
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::EQ));

        let mut it = ">".chars();
        my.next(it.borrow_mut());
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::GT));

        let mut it = "<".chars();
        my.next(it.borrow_mut());
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::LT));
    }

    #[test]
    fn lex_quoted_strings1() {
        let mut my = Parser::new();
        let mut it = " \"abc\\\"d\"".chars();
        my.next(it.borrow_mut());
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::SimpleString(String::from("abc\\\"d"))));
    }

    #[test]
    fn lex_quoted_strings2() {
        let mut my = Parser::new();
        let mut it = " \"abc\\".chars();
        my.next(it.borrow_mut());
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::SimpleString(String::from("abc\\"))));
    }

    #[test]
    fn simple_strings1() {
        let mut my = Parser::new();
        let mut it = " abc\\".chars();
        my.next(it.borrow_mut());
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::SimpleString(String::from("abc\\"))));
    }

    #[test]
    fn simple_strings2() {
        let mut my = Parser::new();
        let mut it = " dc.ti\\x ".chars();
        my.next(it.borrow_mut());

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("dc.ti\\x"))));
    }

    #[test]
    fn keywords() {
        let mut my = Parser::new();
        let mut it = "and OR Not prox sortby All aNy adJ".chars();
        my.next(it.borrow_mut());

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::And));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Or));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Not));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Prox));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Sortby));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("All"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("aNy"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("adJ"))));
    }

    #[test]
    fn empty() {
        let s = String::from("  ");
        let mut my = Parser::new();
        let res = my.parse(s.chars().borrow_mut());
        assert!(res.is_ok());
    }
}
