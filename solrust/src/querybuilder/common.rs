//! This module provides definition and implementation of Solr Common Query Parser.

use crate::querybuilder::facet::FacetBuilder;
use crate::querybuilder::q::{Operator, SolrQueryExpression};
use crate::querybuilder::sanitizer::SOLR_SPECIAL_CHARACTERS;
use crate::querybuilder::sort::SortOrderBuilder;
use solrust_derive::SolrCommonQueryParser;
use std::borrow::Cow;
use std::collections::HashMap;

/// The trait of builder that generates parameter for [Solr Common Query Parser](https://solr.apache.org/guide/solr/latest/query-guide/common-query-parameters.html).
pub trait SolrCommonQueryBuilder {
    /// Add [sort parameter](https://solr.apache.org/guide/solr/latest/query-guide/common-query-parameters.html#sort-parameter)
    fn sort(self, sort: &SortOrderBuilder) -> Self;
    /// Add [start parameter](https://solr.apache.org/guide/solr/latest/query-guide/common-query-parameters.html#start-parameter)
    fn start(self, start: u32) -> Self;
    /// Add [rows parameter](https://solr.apache.org/guide/solr/latest/query-guide/common-query-parameters.html#rows-parameter)
    fn rows(self, rows: u32) -> Self;
    /// Add [fq parameter](https://solr.apache.org/guide/solr/latest/query-guide/common-query-parameters.html#fq-filter-query-parameter)
    ///
    /// `fq` parameter will be added as many times as this method is called.
    fn fq(self, fq: &impl SolrQueryExpression) -> Self;
    /// Add [fl parameter](https://solr.apache.org/guide/solr/latest/query-guide/common-query-parameters.html#fl-field-list-parameter)
    fn fl(self, fl: String) -> Self;
    /// Add parameters for [debug](https://solr.apache.org/guide/solr/latest/query-guide/common-query-parameters.html#debug-parameter).
    ///
    /// Calling this method will add the parameters `debug=all` and `debug.explain.structured=true`.
    fn debug(self) -> Self;
    /// Add [wt parameter](https://solr.apache.org/guide/solr/latest/query-guide/common-query-parameters.html#wt-parameter)
    fn wt(self, wt: &str) -> Self;
    /// Add [facet parameters](https://solr.apache.org/guide/solr/latest/query-guide/faceting.html).
    ///
    /// facet parameters will be added as many times as this method is called.
    fn facet(self, facet: &impl FacetBuilder) -> Self;
    /// Add `q.op` parameter.
    ///
    /// This parameter is not a Solr Common Query Parser parameter, but is defined here because it is used by all other query parsers.
    fn op(self, op: Operator) -> Self;
    /// Build the parameters.
    fn build(self) -> Vec<(String, String)>;
    /// Escape [Solr special characters](https://solr.apache.org/guide/solr/latest/query-guide/standard-query-parser.html#escaping-special-characters).
    fn sanitize<'a>(&self, s: &'a str) -> Cow<'a, str>;
}

/// Implementation of Solr Common Query Parser.
#[derive(SolrCommonQueryParser)]
pub struct CommonQueryBuilder {
    params: HashMap<String, String>,
    multi_params: HashMap<String, Vec<String>>,
}

impl CommonQueryBuilder {
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
    use crate::querybuilder::facet::{FieldFacetBuilder, FieldFacetSortOrder};
    use crate::querybuilder::q::QueryOperand;

    #[test]
    fn test_with_no_params() {
        let builder = CommonQueryBuilder::new();
        assert!(builder.build().is_empty());
    }

    #[test]
    fn test_w() {
        let sort = SortOrderBuilder::new().desc("score").asc("age");
        let builder = CommonQueryBuilder::new().sort(&sort);

        assert_eq!(
            builder.build(),
            vec![("sort".to_string(), "score desc,age asc".to_string())],
        );
    }

    #[test]
    fn test_with_start() {
        let builder = CommonQueryBuilder::new().start(10);
        assert_eq!(builder.build(), vec![("start".to_string(), 10.to_string())],);
    }

    #[test]
    fn test_with_rows() {
        let builder = CommonQueryBuilder::new().rows(50);
        assert_eq!(builder.build(), vec![("rows".to_string(), 50.to_string())]);
    }

    #[test]
    fn test_with_fq() {
        let op = QueryOperand::from("name:alice");
        let builder = CommonQueryBuilder::new().fq(&op);

        assert_eq!(
            builder.build(),
            vec![("fq".to_string(), "name:alice".to_string())],
        );
    }

    #[test]
    fn test_with_multiple_fq() {
        let builder = CommonQueryBuilder::new()
            .fq(&QueryOperand::from("name:alice"))
            .fq(&QueryOperand::from("age:24"));

        assert_eq!(
            builder.build(),
            vec![
                (String::from("fq"), String::from("name:alice")),
                (String::from("fq"), String::from("age:24"))
            ],
        );
    }

    #[test]
    fn test_with_fl() {
        let builder = CommonQueryBuilder::new().fl(String::from("id,name"));

        assert_eq!(
            builder.build(),
            vec![(String::from("fl"), String::from("id,name")),],
        );
    }

    #[test]
    fn test_q_op() {
        let builder = CommonQueryBuilder::new().op(Operator::AND);

        assert_eq!(
            builder.build(),
            vec![(String::from("q.op"), String::from("AND")),],
        )
    }

    #[test]
    fn test_facet() {
        let facet = FieldFacetBuilder::new("gender").sort(FieldFacetSortOrder::Count);
        let builder = CommonQueryBuilder::new().facet(&facet);

        let mut expected = vec![
            (String::from("facet"), String::from("true")),
            (String::from("facet.field"), String::from("gender")),
            (String::from("f.gender.facet.sort"), String::from("count")),
        ];
        let mut actual = builder.build();
        expected.sort();
        actual.sort();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_multiple_field_facet() {
        let facet1 = FieldFacetBuilder::new("gender").sort(FieldFacetSortOrder::Count);
        let facet2 = FieldFacetBuilder::new("prefecture").min_count(1);
        let builder = CommonQueryBuilder::new().facet(&facet1).facet(&facet2);

        let mut expected = vec![
            (String::from("facet"), String::from("true")),
            (String::from("facet.field"), String::from("gender")),
            (String::from("f.gender.facet.sort"), String::from("count")),
            (String::from("facet.field"), String::from("prefecture")),
            (
                String::from("f.prefecture.facet.mincount"),
                String::from("1"),
            ),
        ];
        let mut actual = builder.build();
        expected.sort();
        actual.sort();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_debug() {
        let builder = CommonQueryBuilder::new().wt("json");
        assert_eq!(
            builder.build(),
            vec![("wt".to_string(), "json".to_string())]
        )
    }
}
