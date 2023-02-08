use crate::querybuilder::common::SolrCommonQueryBuilder;
use crate::querybuilder::facet::FacetBuilder;
use crate::querybuilder::q::{Operator, SolrQueryExpression};
use crate::querybuilder::sort::SortOrderBuilder;
use solrust_derive::{SolrCommonQueryParser, SolrDisMaxQueryParser};
use std::collections::HashMap;

pub trait SolrDisMaxQueryBuilder: SolrCommonQueryBuilder {
    fn q(self, q: String) -> Self;
    fn qf(self, qf: &str) -> Self;
    fn qs(self, qs: u32) -> Self;
    fn pf(self, pf: &str) -> Self;
    fn ps(self, ps: u32) -> Self;
    fn mm(self, mm: &str) -> Self;
    fn q_alt(self, q: &impl SolrQueryExpression) -> Self;
    fn tie(self, tie: f64) -> Self;
    fn bq(self, bq: &impl SolrQueryExpression) -> Self;
    fn bf(self, bf: &str) -> Self;
}

#[derive(SolrCommonQueryParser, SolrDisMaxQueryParser)]
pub struct DisMaxQueryBuilder {
    params: HashMap<String, String>,
    multi_params: HashMap<String, Vec<String>>,
}

impl DisMaxQueryBuilder {
    pub fn new() -> Self {
        let mut params = HashMap::new();
        params.insert("defType".to_string(), "dismax".to_string());

        Self {
            params: params,
            multi_params: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::querybuilder::q::QueryOperand;

    #[test]
    fn test_q() {
        let q = QueryOperand::from("プログラミング Rust");
        let builder = DisMaxQueryBuilder::new().q(q.to_string());

        let mut expected = vec![
            ("defType".to_string(), "dismax".to_string()),
            ("q".to_string(), "プログラミング Rust".to_string()),
        ];
        let mut actual = builder.build();
        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_qf() {
        let q = QueryOperand::from("プログラミング Rust");
        let builder = DisMaxQueryBuilder::new().q(q.to_string()).qf("title text");

        let mut expected = vec![
            ("defType".to_string(), "dismax".to_string()),
            ("q".to_string(), "プログラミング Rust".to_string()),
            ("qf".to_string(), "title text".to_string()),
        ];
        let mut actual = builder.build();
        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_sample_query() {
        let q = QueryOperand::from("*:*");
        let sort = SortOrderBuilder::new().desc("score").asc("start_at");
        let builder = DisMaxQueryBuilder::new()
            .q("すぬけ 耳".to_string())
            .qf("text_ja")
            .op(Operator::AND)
            .wt("json")
            .debug()
            .q_alt(&q)
            .sort(&sort)
            .fl("problem_title".to_string());

        let mut expected = vec![
            ("defType".to_string(), "dismax".to_string()),
            ("q".to_string(), "すぬけ 耳".to_string()),
            ("qf".to_string(), "text_ja".to_string()),
            ("q.op".to_string(), "AND".to_string()),
            ("wt".to_string(), "json".to_string()),
            ("debug".to_string(), "all".to_string()),
            ("debug.explain.structured".to_string(), "true".to_string()),
            ("q.alt".to_string(), "*:*".to_string()),
            ("sort".to_string(), "score desc,start_at asc".to_string()),
            ("fl".to_string(), "problem_title".to_string()),
        ];
        let mut actual = builder.build();
        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
    }
}
