use crate::error::ParseError;

#[derive(PartialEq)]
pub(crate) enum Token {
    EOS,
    Relop(String),
    And(String),
    Or(String),
    Not(String),
    Prox(String),
    PrefixName(String),
    SimpleString(String),
    Sortby(String),
    Modifier,
    LP,
    RP,
}

pub(crate) struct Lexer {
    strict: bool,
    first: bool,
    look_ch: Option<char>,
}

impl Lexer {
    pub(crate) fn new() -> Self {
        Lexer {
            strict: false,
            first: true,
            look_ch: None,
        }
    }

    pub(crate) fn strict(self: &mut Self, strict: bool) {
        self.strict = strict;
    }

    pub(crate) fn next(self: &mut Self, get: &mut dyn Iterator<Item = char>) {
        self.look_ch = get.next();
        self.first = false;
    }

    pub(crate) fn lex(
        self: &mut Self,
        get: &mut dyn Iterator<Item = char>,
    ) -> Result<Token, ParseError> {
        if self.first {
            self.next(get);
        }
        while let Some(ch) = self.look_ch {
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
                        self.next(get);
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
                println!("relation_like = {}", relation_like);
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
                    return Ok(Token::Boolop(s));
                }
                if s.eq_ignore_ascii_case("or") {
                    return Ok(Token::Boolop(s));
                }
                if s.eq_ignore_ascii_case("not") {
                    return Ok(Token::Boolop(s));
                }
                if s.eq_ignore_ascii_case("prox") {
                    return Ok(Token::Boolop(s));
                }
                if s.eq_ignore_ascii_case("sortby") {
                    return Ok(Token::Sortby(s));
                }
                if s.eq_ignore_ascii_case("all")
                    || s.eq_ignore_ascii_case("any")
                    || s.eq_ignore_ascii_case("adj")
                {
                    relation_like = true;
                }
                if relation_like {
                    println!("returning prefixname");
                    return Ok(Token::PrefixName(s));
                }
                return Ok(Token::SimpleString(s));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::BorrowMut;

    #[test]
    fn create() {
        let my_sc = Lexer::new();
        assert!(!my_sc.strict);
    }

    #[test]
    fn lex_ops() {
        let mut it = "= == > >= < <= <>/()".chars();
        let mut my = Lexer::new();

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
        let mut it = " \"abc\\\"d\"\"\"".chars();
        let mut my = Lexer::new();
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::SimpleString(String::from("abc\\\"d"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::SimpleString(String::from(""))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::EOS));
    }

    #[test]
    fn lex_quoted_strings2() {
        let mut it = " \"abc\\".chars();
        let mut my = Lexer::new();
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::SimpleString(String::from("abc\\"))));
    }

    #[test]
    fn simple_strings1() {
        let mut it = " abc\\".chars();
        let mut my = Lexer::new();
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::SimpleString(String::from("abc\\"))));
    }

    #[test]
    fn strict1() {
        let mut it = " abc\\".chars();
        let mut my = Lexer::new();
        my.strict(true);
        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("abc\\"))));
    }

    #[test]
    fn simple_strings2() {
        let mut it = " dc.ti\\x ".chars();
        let mut my = Lexer::new();

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("dc.ti\\x"))));
    }

    #[test]
    fn keywords() {
        let mut it = "and OR Not prox sortby All aNy adJ".chars();
        let mut my = Lexer::new();

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Boolop(String::from("and"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Boolop(String::from("OR"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Boolop(String::from("Not"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Boolop(String::from("prox"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Sortby(String::from("sortby"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("All"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("aNy"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("adJ"))));
    }
}
