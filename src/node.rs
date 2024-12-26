pub struct CqlSt {
    index: String,
    index_uri: Option<String>,
    term: String,
    relation: String,
    relation_uri: Option<String>,
    modifiers: Option<Box<CqlNode>>,
}

pub struct CqlBoolean {
    value: String,
    left: Option<Box<CqlNode>>,
    right: Option<Box<CqlNode>>,
}

pub struct CqlSort {
    index: String,
    next: Option<Box<CqlNode>>,
    modifiers: Option<Box<CqlNode>>,
    search: Option<Box<CqlNode>>,
}

pub enum CqlNode {
    St(CqlSt),
    Boolean(CqlBoolean),
    Sort(CqlSort),
}

impl CqlNode {
    fn mk_sc(index: &str, relation: &str, term: &str) -> CqlNode {
        let st = CqlSt {
            index: String::from(index),
            index_uri: None,
            term: String::from(term),
            relation: String::from(relation),
            relation_uri: None,
            modifiers: None,
        };
        CqlNode::St(st)
    }
    fn mk_boolean(value: &str, left: Option<Box<CqlNode>>, right: Option<Box<CqlNode>>) -> CqlNode {
        let bo = CqlBoolean {
            value: String::from(value),
            left,
            right,
        };
        CqlNode::Boolean(bo)
    }
    fn mk_sort(index: &str, modifiers: Option<Box<CqlNode>>) -> CqlNode {
        let sort = CqlSort {
            index: String::from(index),
            modifiers,
            next: None,
            search: None,
        };
        CqlNode::Sort(sort)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_sc() {
        let my_sc = CqlNode::mk_sc("ti", "=", "value");
        let mut ok = false;
        match my_sc {
            CqlNode::St(n) => {
                assert_eq!(n.index, "ti");
                assert_eq!(n.relation, "=");
                assert_eq!(n.term, "value");
                assert!(n.index_uri.is_none());
                assert!(n.relation_uri.is_none());
                assert!(n.modifiers.is_none());
                assert!(n.modifiers.is_none());
                ok = true;
            }
            _ => {}
        }
        assert!(ok);
    }

    #[test]
    fn create_boolean() {
        let my_bool = CqlNode::mk_boolean("and", None, None);
        let mut ok = false;
        match my_bool {
            CqlNode::Boolean(n) => {
                assert_eq!(n.value, "and");
                assert!(n.left.is_none());
                assert!(n.right.is_none());
                ok = true;
            }
            _ => {}
        }
        assert!(ok);
    }

    #[test]
    fn create_sort() {
        let my_sort = CqlNode::mk_sort("date", None);
        let mut ok = false;
        match my_sort {
            CqlNode::Sort(n) => {
                assert_eq!(n.index, "date");
                assert!(n.modifiers.is_none());
                assert!(n.next.is_none());
                assert!(n.search.is_none());
                ok = true;
            }
            _ => {}
        }
        assert!(ok);
    }

    #[test]
    fn create_tree() {
        let my_sc1 = Box::new(CqlNode::mk_sc("ti", "=", "house"));
        let my_sc2 = Box::new(CqlNode::mk_sc("au", "=", "andersen"));
        let my_bool = CqlNode::mk_boolean("and", Some(my_sc1), Some(my_sc2));
        let mut matches = 0;
        match my_bool {
            CqlNode::Boolean(n) => {
                match n.left.as_deref().unwrap() {
                    CqlNode::St(n1) => {
                        assert_eq!("ti", n1.index);
                        matches += 1;
                    }
                    _ => {}
                };
                match n.right.as_deref().unwrap() {
                    CqlNode::St(n2) => {
                        assert_eq!("au", n2.index);
                        matches += 1;
                    }
                    _ => {}
                };
            }
            _ => {}
        }
        assert_eq!(2, matches);
    }
}
