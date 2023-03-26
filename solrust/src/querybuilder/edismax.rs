//! This module provides definition and implementation of Solr eDisMax Query Parser.

use crate::querybuilder::common::SolrCommonQueryBuilder;
use crate::querybuilder::dismax::SolrDisMaxQueryBuilder;
use crate::querybuilder::facet::FacetBuilder;
use crate::querybuilder::q::{Operator, SolrQueryExpression};
use crate::querybuilder::sanitizer::SOLR_SPECIAL_CHARACTERS;
use crate::querybuilder::sort::SortOrderBuilder;
use solrust_derive::{SolrCommonQueryParser, SolrDisMaxQueryParser, SolrEDisMaxQueryParser};
use std::borrow::Cow;
use std::collections::HashMap;

/// The trait of builder that generates parameter for [Solr eDisMax Query Parser](https://solr.apache.org/guide/solr/latest/query-guide/edismax-query-parser.html).
pub trait SolrEDisMaxQueryBuilder: SolrDisMaxQueryBuilder {
    /// Add `sow` parameter.
    fn sow(self, sow: bool) -> Self;
    /// Add `boost` parameter.
    fn boost(self, boost: &str) -> Self;
    /// Add `lowercaseOperators` parameter.
    fn lowercase_operators(self, flag: bool) -> Self;
    /// Add `pf2` parameter.
    fn pf2(self, pf: &str) -> Self;
    /// Add `ps2` parameter.
    fn ps2(self, ps: u32) -> Self;
    /// Add `pf3` parameter.
    fn pf3(self, pf: &str) -> Self;
    /// Add `ps3` parameter.
    fn ps3(self, ps: u32) -> Self;
    /// Add `stopwords` parameter.
    fn stopwords(self, flag: bool) -> Self;
    /// Add `uf` parameter.
    fn uf(self, uf: &str) -> Self;
}

/// Implementation of Solr eDisMax Query Parser.
#[derive(SolrCommonQueryParser, SolrDisMaxQueryParser, SolrEDisMaxQueryParser)]
pub struct EDisMaxQueryBuilder {
    params: HashMap<String, String>,
    multi_params: HashMap<String, Vec<String>>,
}

impl EDisMaxQueryBuilder {
    pub fn new() -> Self {
        let mut params = HashMap::new();
        params.insert("defType".to_string(), "edismax".to_string());

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
        let builder = EDisMaxQueryBuilder::new().q(q.to_string());

        let mut expected = vec![
            ("defType".to_string(), "edismax".to_string()),
            ("q".to_string(), "プログラミング Rust".to_string()),
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
        let builder = EDisMaxQueryBuilder::new()
            .q("すぬけ 耳".to_string())
            .qf("text_ja text_en")
            .op(Operator::AND)
            .wt("json")
            .sow(true)
            .boost("boost")
            .debug()
            .q_alt(&q)
            .sort(&sort)
            .fl("problem_title".to_string());

        let mut expected = vec![
            ("defType".to_string(), "edismax".to_string()),
            ("q".to_string(), "すぬけ 耳".to_string()),
            ("qf".to_string(), "text_ja text_en".to_string()),
            ("sow".to_string(), "true".to_string()),
            ("boost".to_string(), "boost".to_string()),
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
