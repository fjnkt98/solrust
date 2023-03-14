//! This module defines structs represent query operand and query expression for Solr Standard Query Parser.

use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::ops;

/// Regex object for sanitizing the [Solr special characters](https://solr.apache.org/guide/solr/latest/query-guide/standard-query-parser.html#escaping-special-characters).
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(\+|\-|&&|\|\||!|\(|\)|\{|\}|\[|\]|\^|"|\~|\*|\?|:|/|AND|OR)"#).unwrap()
});

/// Marker trait of Solr query expression.
pub trait SolrQueryExpression: Display {}

/// Marker trait of Solr query operand model: standard, boost, phrase query, etc.
pub trait SolrQueryOperandModel {}

/// Kind of Solr query expression.
pub enum QueryExpressionKind {
    Operand(QueryOperand),
    Expression(QueryExpression),
}

/// Tuple struct representing a query expression.
///
/// This is responsible for wrapping the search expression. Implement addition and multiplication of the expressions(corresponding OR and AND searches) by implementing Add and Mul traits in this struct.
/// The search expression is taken in the form of a String, so any search expression can ben included, but it cannot be guaranteed that the syntax is correct.
pub struct QueryOperand(pub String);

impl SolrQueryExpression for QueryOperand {}

impl Display for QueryOperand {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

impl From<&str> for QueryOperand {
    fn from(expr: &str) -> Self {
        Self(expr.to_string())
    }
}

/// Implement the addition between QueryOperand.
impl ops::Add<QueryOperand> for QueryOperand {
    type Output = QueryExpression;

    fn add(self, rhs: QueryOperand) -> QueryExpression {
        QueryExpression {
            operator: Operator::OR,
            operands: vec![
                QueryExpressionKind::Operand(self),
                QueryExpressionKind::Operand(rhs),
            ],
        }
    }
}

/// Implement the multiplication between QueryOperand.
impl ops::Mul<QueryOperand> for QueryOperand {
    type Output = QueryExpression;

    fn mul(self, rhs: QueryOperand) -> QueryExpression {
        QueryExpression {
            operator: Operator::AND,
            operands: vec![
                QueryExpressionKind::Operand(self),
                QueryExpressionKind::Operand(rhs),
            ],
        }
    }
}

/// Implement the addition of QueryOperand and QueryExpression.
impl ops::Add<QueryExpression> for QueryOperand {
    type Output = QueryExpression;

    fn add(self, rhs: QueryExpression) -> QueryExpression {
        match rhs.operator {
            Operator::OR => {
                let mut operands = vec![QueryExpressionKind::Operand(self)];
                operands.extend(rhs.operands.into_iter());
                QueryExpression {
                    operator: Operator::OR,
                    operands: operands,
                }
            }
            Operator::AND => QueryExpression {
                operator: Operator::OR,
                operands: vec![
                    QueryExpressionKind::Operand(self),
                    QueryExpressionKind::Expression(rhs),
                ],
            },
        }
    }
}

/// Implement the multiplication of QueryOperand and QueryExpression.
impl ops::Mul<QueryExpression> for QueryOperand {
    type Output = QueryExpression;

    fn mul(self, rhs: QueryExpression) -> QueryExpression {
        match rhs.operator {
            Operator::AND => {
                let mut operands = vec![QueryExpressionKind::Operand(self)];
                operands.extend(rhs.operands.into_iter());
                QueryExpression {
                    operator: Operator::AND,
                    operands: operands,
                }
            }
            Operator::OR => QueryExpression {
                operator: Operator::AND,
                operands: vec![
                    QueryExpressionKind::Operand(self),
                    QueryExpressionKind::Expression(rhs),
                ],
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Operator {
    AND,
    OR,
}

/// Struct representing query expression, that is multiple query operand or query expression combined with logical operators.
pub struct QueryExpression {
    pub operator: Operator,
    pub operands: Vec<QueryExpressionKind>,
}

/// Trait that implement helper methods that generate QueryExpression from a vector of query operand.
pub trait Aggregation<T: SolrQueryExpression> {
    fn sum(operands: Vec<T>) -> QueryExpression;
    fn prod(operands: Vec<T>) -> QueryExpression;
}

/// Implementation of the method that generates QueryExpression from a vector of QueryOperand.
impl Aggregation<QueryOperand> for QueryExpression {
    fn sum(operands: Vec<QueryOperand>) -> QueryExpression {
        QueryExpression {
            operator: Operator::OR,
            operands: operands
                .into_iter()
                .map(|op| QueryExpressionKind::Operand(op))
                .collect(),
        }
    }

    fn prod(operands: Vec<QueryOperand>) -> QueryExpression {
        QueryExpression {
            operator: Operator::AND,
            operands: operands
                .into_iter()
                .map(|op| QueryExpressionKind::Operand(op))
                .collect(),
        }
    }
}

/// Implementation of the method that generates QueryExpression from a vector of QueryExpression.
impl Aggregation<QueryExpression> for QueryExpression {
    fn sum(operands: Vec<QueryExpression>) -> QueryExpression {
        QueryExpression {
            operator: Operator::OR,
            operands: operands
                .into_iter()
                .map(|op| QueryExpressionKind::Expression(op))
                .collect(),
        }
    }

    fn prod(operands: Vec<QueryExpression>) -> QueryExpression {
        QueryExpression {
            operator: Operator::AND,
            operands: operands
                .into_iter()
                .map(|op| QueryExpressionKind::Expression(op))
                .collect(),
        }
    }
}

impl SolrQueryExpression for QueryExpression {}

impl Display for QueryExpression {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let operator = match self.operator {
            Operator::AND => " AND ",
            Operator::OR => " OR ",
        };

        // Convert elements of a vector recursively into a String.
        let s = self
            .operands
            .iter()
            .map(|expr| match expr {
                QueryExpressionKind::Operand(op) => op.to_string(),
                QueryExpressionKind::Expression(expr) => format!("({})", expr.to_string()),
            })
            .collect::<Vec<String>>()
            .join(operator);
        write!(f, "{}", s)?;

        Ok(())
    }
}

/// Implement the addition between QueryExpression.
impl ops::Add<QueryExpression> for QueryExpression {
    type Output = QueryExpression;

    fn add(self, rhs: QueryExpression) -> QueryExpression {
        if self.operator == Operator::OR && rhs.operator == Operator::OR {
            let operands = Vec::from_iter(itertools::chain(
                self.operands.into_iter(),
                rhs.operands.into_iter(),
            ));
            return QueryExpression {
                operator: Operator::OR,
                operands,
            };
        } else {
            return QueryExpression {
                operator: Operator::OR,
                operands: vec![
                    QueryExpressionKind::Expression(self),
                    QueryExpressionKind::Expression(rhs),
                ],
            };
        }
    }
}

/// Implement the multiplication between QueryExpression.
impl ops::Mul<QueryExpression> for QueryExpression {
    type Output = QueryExpression;

    fn mul(self, rhs: QueryExpression) -> QueryExpression {
        if self.operator == Operator::AND && rhs.operator == Operator::AND {
            let operands = Vec::from_iter(itertools::chain(
                self.operands.into_iter(),
                rhs.operands.into_iter(),
            ));
            return QueryExpression {
                operator: Operator::AND,
                operands,
            };
        } else {
            return QueryExpression {
                operator: Operator::AND,
                operands: vec![
                    QueryExpressionKind::Expression(self),
                    QueryExpressionKind::Expression(rhs),
                ],
            };
        }
    }
}

/// Implement the addition of QueryExpression and QueryOperand.
impl ops::Add<QueryOperand> for QueryExpression {
    type Output = QueryExpression;

    fn add(mut self, rhs: QueryOperand) -> QueryExpression {
        match self.operator {
            Operator::OR => {
                self.operands.push(QueryExpressionKind::Operand(rhs));
                self
            }
            Operator::AND => QueryExpression {
                operator: Operator::OR,
                operands: vec![
                    QueryExpressionKind::Expression(self),
                    QueryExpressionKind::Operand(rhs),
                ],
            },
        }
    }
}

/// Implement the multiplication of QueryExpression and QueryOperand.
impl ops::Mul<QueryOperand> for QueryExpression {
    type Output = QueryExpression;

    fn mul(mut self, rhs: QueryOperand) -> QueryExpression {
        match self.operator {
            Operator::AND => {
                self.operands.push(QueryExpressionKind::Operand(rhs));
                self
            }
            Operator::OR => QueryExpression {
                operator: Operator::AND,
                operands: vec![
                    QueryExpressionKind::Expression(self),
                    QueryExpressionKind::Operand(rhs),
                ],
            },
        }
    }
}

/// Struct to building plain search expression(e.g. text_en:foo)
pub struct StandardQueryOperand {
    field: String,
    word: String,
}

impl SolrQueryOperandModel for StandardQueryOperand {}

impl StandardQueryOperand {
    pub fn new(field: &str, word: &str) -> Self {
        Self {
            field: String::from(field),
            word: String::from(word),
        }
    }
}

impl Display for StandardQueryOperand {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let field = RE.replace_all(&self.field, r"\$0");
        let word = RE.replace_all(&self.word, r"\$0");
        write!(f, "{}:{}", field, word)?;
        Ok(())
    }
}

impl From<StandardQueryOperand> for QueryOperand {
    fn from(op: StandardQueryOperand) -> QueryOperand {
        QueryOperand(op.to_string())
    }
}

/// Struct to building range search expression(e.g. text_en:[* TO *])
pub struct RangeQueryOperand {
    field: String,
    start: Option<String>,
    end: Option<String>,
    left_open: bool,
    right_open: bool,
}

impl SolrQueryOperandModel for RangeQueryOperand {}

impl RangeQueryOperand {
    pub fn new(field: &str) -> Self {
        let field = RE.replace_all(field, r"\$0");
        Self {
            field: String::from(field),
            start: None,
            end: None,
            left_open: false,
            right_open: true,
        }
    }

    pub fn gt(mut self, start: String) -> Self {
        self.start = Some(start);
        self.left_open = true;
        self
    }

    pub fn ge(mut self, start: String) -> Self {
        self.start = Some(start);
        self.left_open = false;
        self
    }

    pub fn lt(mut self, end: String) -> Self {
        self.end = Some(end);
        self.right_open = true;
        self
    }
    pub fn le(mut self, end: String) -> Self {
        self.end = Some(end);
        self.right_open = false;
        self
    }
}

impl Display for RangeQueryOperand {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let left_parenthesis = if self.left_open { '{' } else { '[' };
        let right_parenthesis = if self.right_open { '}' } else { ']' };
        let start = match &self.start {
            Some(start) => String::from(RE.replace_all(start, r"\$0")),
            None => String::from("*"),
        };
        let end = match &self.end {
            Some(end) => String::from(RE.replace_all(end, r"\$0")),
            None => String::from("*"),
        };

        write!(
            f,
            "{}:{}{} TO {}{}",
            self.field, left_parenthesis, start, end, right_parenthesis
        )?;
        Ok(())
    }
}

impl From<RangeQueryOperand> for QueryOperand {
    fn from(op: RangeQueryOperand) -> QueryOperand {
        QueryOperand(op.to_string())
    }
}

/// Struct to building phrase query expression(e.g. text_en:"foo bar")
pub struct PhraseQueryOperand {
    field: String,
    word: String,
}

impl SolrQueryOperandModel for PhraseQueryOperand {}

impl PhraseQueryOperand {
    pub fn new(field: &str, word: &str) -> Self {
        Self {
            field: String::from(field),
            word: String::from(word),
        }
    }
}

impl Display for PhraseQueryOperand {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let field = RE.replace_all(&self.field, r"\$0");
        let word = RE.replace_all(&self.word, r"\$0");
        write!(f, r#"{}:"{}""#, field, word)?;
        Ok(())
    }
}

impl From<PhraseQueryOperand> for QueryOperand {
    fn from(op: PhraseQueryOperand) -> QueryOperand {
        QueryOperand(op.to_string())
    }
}

// Struct to building boost query expression(e.g. text_en:foo^3)
pub struct BoostQueryOperand {
    field: String,
    word: String,
    boost: f64,
}

impl SolrQueryOperandModel for BoostQueryOperand {}

impl BoostQueryOperand {
    pub fn new(field: &str, word: &str, boost: f64) -> Self {
        Self {
            field: String::from(field),
            word: String::from(word),
            boost: boost,
        }
    }
}

impl Display for BoostQueryOperand {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let field = RE.replace_all(&self.field, r"\$0");
        let word = RE.replace_all(&self.word, r"\$0");
        write!(f, "{}:{}^{}", field, word, self.boost)?;
        Ok(())
    }
}

impl From<BoostQueryOperand> for QueryOperand {
    fn from(op: BoostQueryOperand) -> QueryOperand {
        QueryOperand(op.to_string())
    }
}

/// Struct to building fuzzy query expression(e.g. text_en:foo~1)
pub struct FuzzyQueryOperand {
    field: String,
    word: String,
    fuzzy: u32,
}

impl SolrQueryOperandModel for FuzzyQueryOperand {}

impl FuzzyQueryOperand {
    pub fn new(field: &str, word: &str, fuzzy: u32) -> Self {
        Self {
            field: String::from(field),
            word: String::from(word),
            fuzzy: fuzzy,
        }
    }
}

impl Display for FuzzyQueryOperand {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let field = RE.replace_all(&self.field, r"\$0");
        let word = RE.replace_all(&self.word, r"\$0");
        write!(f, "{}:{}~{}", field, word, self.fuzzy)?;
        Ok(())
    }
}

impl From<FuzzyQueryOperand> for QueryOperand {
    fn from(op: FuzzyQueryOperand) -> QueryOperand {
        QueryOperand(op.to_string())
    }
}

/// Struct to building proximity query expression(e.g. text_en:"foo bar"~1)
pub struct ProximityQueryOperand {
    field: String,
    word: String,
    proximity: u32,
}

impl SolrQueryOperandModel for ProximityQueryOperand {}

impl ProximityQueryOperand {
    pub fn new(field: &str, word: &str, proximity: u32) -> Self {
        Self {
            field: String::from(field),
            word: String::from(word),
            proximity: proximity,
        }
    }
}

impl Display for ProximityQueryOperand {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let field = RE.replace_all(&self.field, r"\$0");
        let word = RE.replace_all(&self.word, r"\$0");
        write!(f, r#"{}:"{}"~{}"#, field, word, self.proximity)?;
        Ok(())
    }
}

impl From<ProximityQueryOperand> for QueryOperand {
    fn from(op: ProximityQueryOperand) -> QueryOperand {
        QueryOperand(op.to_string())
    }
}

/// Struct to building constant weight query expression(e.g. text_en:foo^=0)
pub struct ConstantQueryOperand {
    field: String,
    word: String,
    weight: f64,
}

impl SolrQueryOperandModel for ConstantQueryOperand {}

impl ConstantQueryOperand {
    pub fn new(field: &str, word: &str, weight: f64) -> Self {
        Self {
            field: String::from(field),
            word: String::from(word),
            weight: weight,
        }
    }
}

impl Display for ConstantQueryOperand {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let field = RE.replace_all(&self.field, r"\$0");
        let word = RE.replace_all(&self.word, r"\$0");
        write!(f, "{}:{}^={}", field, word, self.weight)?;
        Ok(())
    }
}

impl From<ConstantQueryOperand> for QueryOperand {
    fn from(op: ConstantQueryOperand) -> QueryOperand {
        QueryOperand(op.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query_operand_representation() {
        let q = StandardQueryOperand::new("name", "alice");
        assert_eq!(String::from("name:alice"), q.to_string());
    }

    #[test]
    fn test_special_characters_should_escaped() {
        let q =
            StandardQueryOperand::new("text", r#"+ - && || ! ( ) { } [ ] ^ " ~ * ? : / AND OR"#);
        assert_eq!(
            String::from(
                r#"text:\+ \- \&& \|| \! \( \) \{ \} \[ \] \^ \" \~ \* \? \: \/ \AND \OR"#
            ),
            q.to_string()
        );
    }

    #[test]
    fn test_fuzzy_query_operand() {
        let q = FuzzyQueryOperand::new("name", "alice", 1);
        assert_eq!(String::from("name:alice~1"), q.to_string());
    }

    #[test]
    fn test_proximity_query_operand() {
        let q = ProximityQueryOperand::new("name", "alice wonder", 2);
        assert_eq!(String::from(r#"name:"alice wonder"~2"#), q.to_string());
    }

    #[test]
    fn test_boost_query_operand() {
        let q = BoostQueryOperand::new("name", "alice", 10.0);
        assert_eq!(String::from("name:alice^10"), q.to_string());
    }

    #[test]
    fn test_constant_query_operand() {
        let q = ConstantQueryOperand::new("name", "alice", 0.0);
        assert_eq!(String::from("name:alice^=0"), q.to_string());
    }

    #[test]
    fn test_phrase_query_operand() {
        let q = PhraseQueryOperand::new("name", "alice");
        assert_eq!(String::from(r#"name:"alice""#), q.to_string());
    }

    #[test]
    fn test_range_query_with_default_parameter() {
        let q = RangeQueryOperand::new("age");

        assert_eq!(String::from("age:[* TO *}"), q.to_string())
    }

    #[test]
    fn test_range_query_with_gt_parameter() {
        let q = RangeQueryOperand::new("age").gt(10.to_string());

        assert_eq!(String::from("age:{10 TO *}"), q.to_string())
    }

    #[test]
    fn test_range_query_with_ge_parameter() {
        let q = RangeQueryOperand::new("age").ge(10.to_string());

        assert_eq!(String::from("age:[10 TO *}"), q.to_string())
    }

    #[test]
    fn test_range_query_with_lt_parameter() {
        let q = RangeQueryOperand::new("age").lt(20.to_string());

        assert_eq!(String::from("age:[* TO 20}"), q.to_string())
    }

    #[test]
    fn test_range_query_with_le_parameter() {
        let q = RangeQueryOperand::new("age").le(20.to_string());

        assert_eq!(String::from("age:[* TO 20]"), q.to_string())
    }

    #[test]
    fn test_range_query() {
        let q = RangeQueryOperand::new("age")
            .ge(10.to_string())
            .lt(20.to_string());

        assert_eq!(String::from("age:[10 TO 20}"), q.to_string())
    }

    // #[test]
    // fn test_left_close_right_close_range_query() {
    //     let q = RangeQueryOperand::new("age")
    //         .start("10")
    //         .end("20")
    //         .left_close()
    //         .right_close();

    //     assert_eq!(String::from("age:[10 TO 20]"), q.to_string())
    // }

    #[test]
    fn test_add_operands() {
        let op1 = QueryOperand::from("name:alice");
        let op2 = QueryOperand::from("age:24");

        let q = op1 + op2;

        assert_eq!(String::from("name:alice OR age:24"), q.to_string())
    }

    #[test]
    fn test_mul_operands() {
        let op1 = QueryOperand::from("name:alice");
        let op2 = QueryOperand::from("age:24");

        let q = op1 * op2;

        assert_eq!(String::from("name:alice AND age:24"), q.to_string())
    }

    #[test]
    fn test_add_operand_to_expression() {
        let op1 = QueryOperand::from("name:alice");
        let op2 = QueryOperand::from("name:bob");
        let op3 = QueryOperand::from("age:24");

        let q = (op1 * op2) + op3;

        assert_eq!(
            String::from("(name:alice AND name:bob) OR age:24"),
            q.to_string()
        )
    }

    #[test]
    fn test_add_expression_to_operand() {
        let op1 = QueryOperand::from("name:alice");
        let op2 = QueryOperand::from("name:bob");
        let op3 = QueryOperand::from("age:24");

        let q = op1 * (op2 + op3);

        assert_eq!(
            String::from("name:alice AND (name:bob OR age:24)"),
            q.to_string()
        )
    }

    #[test]
    fn test_add_expression_to_expression() {
        let op1 = QueryOperand::from("name:alice");
        let op2 = QueryOperand::from("age:24");
        let op3 = QueryOperand::from("name:bob");
        let op4 = QueryOperand::from("age:32");

        let q = (op1 * op2) + (op3 * op4);

        assert_eq!(
            String::from("(name:alice AND age:24) OR (name:bob AND age:32)"),
            q.to_string()
        )
    }

    #[test]
    fn test_mul_expression_to_expression() {
        let op1 = QueryOperand::from("name:alice");
        let op2 = QueryOperand::from("name:bob");
        let op3 = QueryOperand::from("age:24");
        let op4 = QueryOperand::from("age:32");

        let q = (op1 + op2) * (op3 + op4);

        assert_eq!(
            String::from("(name:alice OR name:bob) AND (age:24 OR age:32)"),
            q.to_string()
        )
    }

    #[test]
    fn test_extend_expression_with_add() {
        let op1 = QueryOperand::from("name:alice");
        let op2 = QueryOperand::from("name:bob");
        let op3 = QueryOperand::from("name:charles");

        let q = op1 + op2 + op3;

        assert_eq!(
            String::from("name:alice OR name:bob OR name:charles"),
            q.to_string()
        )
    }

    #[test]
    fn test_extend_expression_with_mul() {
        let op1 = QueryOperand::from("name:alice");
        let op2 = QueryOperand::from("name:bob");
        let op3 = QueryOperand::from("name:charles");

        let q = op1 * op2 * op3;

        assert_eq!(
            String::from("name:alice AND name:bob AND name:charles"),
            q.to_string()
        )
    }
}
