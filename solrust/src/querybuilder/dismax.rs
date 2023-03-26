//! This module provides definition and implementation of Solr DisMax Query Parser.

use crate::querybuilder::common::SolrCommonQueryBuilder;
use crate::querybuilder::facet::FacetBuilder;
use crate::querybuilder::q::{Operator, SolrQueryExpression};
use crate::querybuilder::sanitizer::SOLR_SPECIAL_CHARACTERS;
use crate::querybuilder::sort::SortOrderBuilder;
use solrust_derive::{SolrCommonQueryParser, SolrDisMaxQueryParser};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;

/// The trait of builder that generates parameter for [Solr Standard Query Parser](https://solr.apache.org/guide/solr/latest/query-guide/dismax-query-parser.html).
pub trait SolrDisMaxQueryBuilder: SolrCommonQueryBuilder {
    /// Add [q parameter](https://solr.apache.org/guide/solr/latest/query-guide/dismax-query-parser.html#q-parameter).
    fn q(self, q: String) -> Self;
    /// Add [qf parameter](https://solr.apache.org/guide/solr/latest/query-guide/dismax-query-parser.html#qf-query-fields-parameter).
    fn qf(self, qf: &str) -> Self;
    /// Add [qs parameter](https://solr.apache.org/guide/solr/latest/query-guide/dismax-query-parser.html#qs-query-phrase-slop-parameter).
    fn qs(self, qs: u32) -> Self;
    /// Add [pf parameter](https://solr.apache.org/guide/solr/latest/query-guide/dismax-query-parser.html#pf-phrase-fields-parameter).
    fn pf(self, pf: &str) -> Self;
    /// Add [ps parameter](https://solr.apache.org/guide/solr/latest/query-guide/dismax-query-parser.html#ps-phrase-slop-parameter).
    fn ps(self, ps: u32) -> Self;
    /// Add [mm parameter](https://solr.apache.org/guide/solr/latest/query-guide/dismax-query-parser.html#mm-minimum-should-match-parameter).
    fn mm(self, mm: &str) -> Self;
    /// Add [q.alt parameter](https://solr.apache.org/guide/solr/latest/query-guide/dismax-query-parser.html#q-alt-parameter).
    fn q_alt(self, q: &impl SolrQueryExpression) -> Self;
    /// Add [tie parameter](https://solr.apache.org/guide/solr/latest/query-guide/dismax-query-parser.html#the-tie-tie-breaker-parameter).
    fn tie(self, tie: f64) -> Self;
    /// Add [bq parameter](https://solr.apache.org/guide/solr/latest/query-guide/dismax-query-parser.html#bq-boost-query-parameter).
    ///
    /// `bq` parameter will be added as many times as this method is called.
    fn bq(self, bq: &impl SolrQueryExpression) -> Self;
    /// Add [bf parameter](https://solr.apache.org/guide/solr/latest/query-guide/dismax-query-parser.html#bf-boost-functions-parameter).
    ///
    /// `bf` parameter will be added as many times as this method is called.
    fn bf(self, bf: &str) -> Self;
}

/// Implementation of DisMax Common Query Parser.
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
    use itertools::{sorted, Itertools};

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
    fn test_sanitized_q() {
        let q = QueryOperand::from("Programming C++");
        let builder = DisMaxQueryBuilder::new().q(q.to_string());

        let expected = sorted(
            vec![
                ("defType".to_string(), "dismax".to_string()),
                ("q".to_string(), "Programming C\\+\\+".to_string()),
            ]
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string())),
        )
        .collect_vec();
        let actual = sorted(builder.build()).collect_vec();
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
