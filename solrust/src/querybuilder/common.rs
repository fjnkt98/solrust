use crate::querybuilder::facet::FacetBuilder;
use crate::querybuilder::q::{Operator, SolrQueryExpression};
use crate::querybuilder::sort::SortOrderBuilder;
use solrust_derive::SolrCommonQueryParser;
use std::collections::HashMap;

pub trait SolrCommonQueryBuilder {
    fn sort(self, sort: &SortOrderBuilder) -> Self;
    fn start(self, start: u32) -> Self;
    fn rows(self, rows: u32) -> Self;
    fn fq(self, fq: &impl SolrQueryExpression) -> Self;
    fn fl(self, fl: String) -> Self;
    fn debug(self) -> Self;
    fn wt(self, wt: &str) -> Self;
    fn facet(self, facet: &impl FacetBuilder) -> Self;
    fn op(self, op: Operator) -> Self;
    fn build(self) -> Vec<(String, String)>;
}

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
