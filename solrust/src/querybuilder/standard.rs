//! This module provides definition and implementation of Solr Standard Query Parser.

use crate::querybuilder::common::SolrCommonQueryBuilder;
use crate::querybuilder::facet::FacetBuilder;
use crate::querybuilder::q::{Operator, SolrQueryExpression};
use crate::querybuilder::sanitizer::SOLR_SPECIAL_CHARACTERS;
use crate::querybuilder::sort::SortOrderBuilder;
use solrust_derive::{SolrCommonQueryParser, SolrStandardQueryParser};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;

/// The trait of builder that generates parameter for [Solr Standard Query Parser](https://solr.apache.org/guide/solr/latest/query-guide/standard-query-parser.html).
pub trait SolrStandardQueryBuilder: SolrCommonQueryBuilder {
    /// Add `q` parameter.
    fn q(self, q: &impl SolrQueryExpression) -> Self;
    /// Add `df` parameter.
    fn df(self, df: &str) -> Self;
    /// Add `sow` parameter.
    fn sow(self, sow: bool) -> Self;
}

/// Implementation of Solr Standard Query Parser.
#[derive(SolrCommonQueryParser, SolrStandardQueryParser)]
pub struct StandardQueryBuilder {
    params: HashMap<String, String>,
    multi_params: HashMap<String, Vec<String>>,
}

impl StandardQueryBuilder {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
            multi_params: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::querybuilder::facet::{FieldFacetBuilder, RangeFacetBuilder};
    use crate::querybuilder::q::{QueryOperand, StandardQueryOperand};

    #[test]
    fn test_with_q() {
        let q = QueryOperand::from("text_ja:hoge");
        let builder = StandardQueryBuilder::new().q(&q);

        assert_eq!(
            vec![("q".to_string(), "text_ja:hoge".to_string())],
            builder.build()
        );
    }

    #[test]
    fn test_sample_query() {
        let q = QueryOperand::from(StandardQueryOperand::new("text_ja", "高橋?"));
        let sort = SortOrderBuilder::new().desc("score").desc("difficulty");
        let facet1 = FieldFacetBuilder::new("category");
        let facet2 = RangeFacetBuilder::new(
            "difficulty",
            0.to_string(),
            2000.to_string(),
            400.to_string(),
        );
        let builder = StandardQueryBuilder::new()
            .q(&q)
            .op(Operator::AND)
            .sow(true)
            .df("text_ja")
            .sort(&sort)
            .facet(&facet1)
            .facet(&facet2);

        let mut expected = vec![
            ("q".to_string(), r#"text_ja:高橋\?"#.to_string()),
            ("df".to_string(), "text_ja".to_string()),
            ("q.op".to_string(), "AND".to_string()),
            ("sow".to_string(), "true".to_string()),
            ("sort".to_string(), "score desc,difficulty desc".to_string()),
            ("facet".to_string(), "true".to_string()),
            ("facet.field".to_string(), "category".to_string()),
            ("facet.range".to_string(), "difficulty".to_string()),
            (
                "f.difficulty.facet.range.start".to_string(),
                "0".to_string(),
            ),
            (
                "f.difficulty.facet.range.end".to_string(),
                "2000".to_string(),
            ),
            (
                "f.difficulty.facet.range.gap".to_string(),
                "400".to_string(),
            ),
        ];
        expected.sort();
        let mut actual = builder.build();
        actual.sort();

        assert_eq!(actual, expected);
    }
}
