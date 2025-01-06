use crate::node::CqlNode;
use crate::node::St;
use crate::error::ParseError;
use crate::lex::Token;
use std::rc::Rc;

pub struct Parser {
    strict: bool,
    look_ch: Option<char>,
    look: Token,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            strict: false,
            look_ch: None,
            look: Token::EOS,
        }
    }

    fn next(self: &mut Self, get: &mut dyn Iterator<Item = char>) {
        self.look_ch = get.next();
    }

    fn lex(self: &mut Self, get: &mut dyn Iterator<Item = char>) -> Result<Token, ParseError> {
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
                    return Ok(Token::And(s));
                }
                if s.eq_ignore_ascii_case("or") {
                    return Ok(Token::Or(s));
                }
                if s.eq_ignore_ascii_case("not") {
                    return Ok(Token::Not(s));
                }
                if s.eq_ignore_ascii_case("prox") {
                    return Ok(Token::Prox(s));
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
                    return Ok(Token::PrefixName(s));
                }
                return Ok(Token::SimpleString(s));
            }
        }
    }

    fn search_term(self: &mut Self) -> Option<String> {
        match &self.look {
            Token::SimpleString(name)
            | Token::PrefixName(name)
            | Token::And(name)
            | Token::Or(name)
            | Token::Not(name)
            | Token::Prox(name)
            | Token::Sortby(name) => {
                return Some(String::from(name));
            }
            _ => return None,
        }
    }

    fn relation_symbol(self: &mut Self) -> Option<String> {
        let lead;
        match &self.look {
            Token::EQ => lead = "=",
            Token::GT => lead = ">",
            Token::LT => lead = "<",
            Token::GE => lead = ">=",
            Token::LE => lead = "<=",
            Token::NE => lead = "<>",
            Token::Exact => lead = "==",
            _ => return None,
        }
        Some(String::from(lead))
    }

    fn relation(self: &mut Self) -> Option<String> {
        if let Some(lead) = self.relation_symbol() {
            return Some(lead)
        }
        if let Token::PrefixName(name) = &self.look {
            return Some(String::from(name));
        }
        None
    }

    fn boolean(self: &mut Self) -> Option<String> {
        match &self.look {
            Token::And(name)
            | Token::Or(name)
            | Token::Not(name)
            | Token::Prox(name) => {
                return Some(String::from(name))
            }
            _ => return None,
        }
    }

    fn modifiers(self: &mut Self, get: &mut dyn Iterator<Item = char>) -> Result<Option<Rc<St>>, ParseError> {
        let mut res: Option<Rc<St>> = None;
        while let Token::Modifier = &self.look {
            self.look = self.lex(get)?;
            if let Some(modifier) = self.search_term() {
                self.look = self.lex(get)?;
                if let Some(relation) = self.relation_symbol() {
                    self.look = self.lex(get)?;
                    if let Some(value) = self.search_term() {
                        res = Some(Rc::new(CqlNode::mk_sc(&modifier, &relation, Some(&value), None)));
                        self.look = self.lex(get)?;
                    } else {
                        return Err(ParseError);
                    }
                } else {
                    res = Some(Rc::new(CqlNode::mk_sc(&modifier, "", None, None)));
                }
            } else {
                return Err(ParseError);
            }
        }
        Ok(res)
    }

    fn search_clause(
        self: &mut Self,
        get: &mut dyn Iterator<Item = char>,
        rel: &St,
    ) -> Result<CqlNode, ParseError> {
        if self.look == Token::LP {
            self.look = self.lex(get)?;
            let res = self.cql_query(get, rel)?;
            if self.look != Token::RP {
                return Err(ParseError);
            }
            self.look = self.lex(get)?;
            return Ok(res);
        }
        let n = self.search_term();
        if let Some(n) = n {
            self.look = self.lex(get)?;
            if let Some(relation) = self.relation() {
                self.look = self.lex(get)?;
                let modifiers = self.modifiers(get)?;
                let rel = CqlNode::mk_sc(&n, &relation, None, modifiers);
                return self.search_clause(get, &rel);
            }
            return Ok(CqlNode::mk_sc_dup(rel, &n));
        }
        // missing search !
        Err(ParseError)
    }

    fn scoped_clause(
        self: &mut Self,
        get: &mut dyn Iterator<Item = char>,
        rel: &St,
    ) -> Result<CqlNode, ParseError> {
        let mut left = self.search_clause(get, rel)?;
        while let Some(op) = self.boolean() {
            self.look = self.lex(get)?;
            let modifiers = self.modifiers(get)?;
            let right = self.search_clause(get, rel)?;
            left = CqlNode::mk_boolean(&op, Box::new(left), Box::new(right), modifiers);
        }
        Ok(left)
    }

    fn cql_query(
        self: &mut Self,
        get: &mut dyn Iterator<Item = char>,
        rel: &St,
    ) -> Result<CqlNode, ParseError> {
        let res = self.scoped_clause(get, rel)?;
        Ok(res)
    }

    pub fn parse(
        self: &mut Self,
        get: &mut dyn Iterator<Item = char>,
    ) -> Result<CqlNode, ParseError> {
        self.next(get);
        self.look = self.lex(get)?;
        let rel = CqlNode::mk_sc("cql.serverChoice", "=", None, None);
        let search = self.cql_query(get, &rel)?;
        let mut sort = Vec::new();
        if let Token::Sortby(_sortby) = &self.look{
            self.look = self.lex(get)?;
            while let Some(index) = &self.search_term() {
                self.look = self.lex(get)?;
                let modifiers = self.modifiers(get)?;
                sort.push(CqlNode::mk_sc(&index, "", None, modifiers));
            }
        }
        if self.look != Token::EOS {
            return Err(ParseError);
        }
        Ok(CqlNode::mk_root(Box::new(search), sort))
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
        assert!(res.is_ok_and(|tok| tok == Token::And(String::from("and"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Or(String::from("OR"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Not(String::from("Not"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Prox(String::from("prox"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::Sortby(String::from("sortby"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("All"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("aNy"))));

        let res = my.lex(it.borrow_mut());
        assert!(res.is_ok_and(|tok| tok == Token::PrefixName(String::from("adJ"))));
    }

    #[test]
    fn empty() {
        let mut my = Parser::new();
        let res = my.parse("  ".chars().borrow_mut());
        assert!(res.is_err());
    }

    #[test]
    fn foo() {
        let mut my = Parser::new();
        let res = my.parse("foo".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti adj computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti = computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti = /a/b=c computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti = /a=b computer".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("(ti = computer)".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("((ti = computer))".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("ti = (computer)".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a and b".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a and/x1=y1 b".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a and/x1=y1/x2=y2 b".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a sortby title".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a sortby title date".chars().borrow_mut());
        assert!(res.is_ok());

        let res = my.parse("a sortby title/a/b date/x=y".chars().borrow_mut());
        assert!(res.is_ok());

    }
}
