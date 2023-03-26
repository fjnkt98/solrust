//! This module defines the traits and structs that generates query parameters for facet search.

use std::string::ToString;
/// Build parameters for facet search.
pub trait FacetBuilder {
    fn build(&self) -> Vec<(String, String)>;
}

/// Sort order for field facet.
///
/// https://solr.apache.org/guide/solr/latest/query-guide/faceting.html#field-value-faceting-parameters:~:text=a%20regular%20expression.-,facet.sort,-Optional
pub enum FieldFacetSortOrder {
    Index,
    Count,
}

/// Type of algorithm or method that Solr should use when faceting a field.
///
/// https://solr.apache.org/guide/solr/latest/query-guide/faceting.html#field-value-faceting-parameters:~:text=in%20the%20response.-,facet.method,-Optional
pub enum FieldFacetMethod {
    Enum,
    Fc,
    Fcs,
}

/// Implementation of the builder generates parameters for field facetting.
pub struct FieldFacetBuilder {
    field: String,
    prefix: Option<String>,
    contains: Option<String>,
    ignore_case: Option<bool>,
    sort: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    min_count: Option<u32>,
    missing: Option<bool>,
    method: Option<String>,
    exists: Option<bool>,
    local_params: Vec<(String, String)>,
}

impl FieldFacetBuilder {
    pub fn new(field: &str) -> Self {
        Self {
            field: field.to_string(),
            prefix: None,
            contains: None,
            ignore_case: None,
            sort: None,
            limit: None,
            offset: None,
            min_count: None,
            missing: None,
            method: None,
            exists: None,
            local_params: Vec::new(),
        }
    }

    /// Add `f.<FIELD_NAME>.facet.prefix` parameter.
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    /// Add `f.<FIELD_NAME>.facet.contains` parameter.
    pub fn contains(mut self, contains: &str) -> Self {
        self.contains = Some(contains.to_string());
        self
    }

    /// Add `f.<FIELD_NAME>.facet.ignoreCase` parameter.
    pub fn ignore_case(mut self, ignore_case: bool) -> Self {
        self.ignore_case = Some(ignore_case);
        self
    }

    /// Add `f.<FIELD_NAME>.facet.sort` parameter.
    pub fn sort(mut self, sort: FieldFacetSortOrder) -> Self {
        self.sort = Some(match sort {
            FieldFacetSortOrder::Count => "count".to_string(),
            FieldFacetSortOrder::Index => "index".to_string(),
        });
        self
    }

    /// Add `f.<FIELD_NAME>.facet.limit` parameter.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add `f.<FIELD_NAME>.facet.offset` parameter.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Add `f.<FIELD_NAME>.facet.mincount` parameter.
    pub fn min_count(mut self, min_count: u32) -> Self {
        self.min_count = Some(min_count);
        self
    }

    /// Add `f.<FIELD_NAME>.facet.missing` parameter.
    pub fn missing(mut self, missing: bool) -> Self {
        self.missing = Some(missing);
        self
    }

    /// Add `f.<FIELD_NAME>.facet.method` parameter.
    pub fn method(mut self, method: FieldFacetMethod) -> Self {
        self.method = Some(match method {
            FieldFacetMethod::Enum => "enum".to_string(),
            FieldFacetMethod::Fc => "fc".to_string(),
            FieldFacetMethod::Fcs => "fcs".to_string(),
        });
        self
    }

    /// Add `f.<FIELD_NAME>.facet.exists` parameter.
    pub fn exists(mut self, exists: bool) -> Self {
        self.exists = Some(exists);
        self
    }

    /// Add a local parameter for faceting.
    pub fn local_param(mut self, key: &str, value: &str) -> Self {
        self.local_params.push((key.to_string(), value.to_string()));
        self
    }
}

impl FacetBuilder for FieldFacetBuilder {
    fn build(&self) -> Vec<(String, String)> {
        let mut result: Vec<(String, String)> = Vec::new();

        if self.local_params.len() == 0 {
            result.push((String::from("facet.field"), self.field.clone()));
        } else {
            let local_param = format!(
                "{{!{}}}",
                self.local_params
                    .iter()
                    .map(|(key, value)| format!("{}={}", key, value))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            result.push((
                String::from("facet.field"),
                format!("{}{}", local_param, self.field),
            ));
        }

        if let Some(prefix) = &self.prefix {
            result.push((format!("f.{}.facet.prefix", self.field), prefix.to_string()));
        }

        if let Some(contains) = &self.contains {
            result.push((
                format!("f.{}.facet.contains", self.field),
                contains.to_string(),
            ));
        }

        if let Some(ignore_case) = &self.ignore_case {
            result.push((
                format!("f.{}.facet.contains.ignoreCase", self.field),
                ignore_case.to_string(),
            ));
        }

        if let Some(sort) = &self.sort {
            result.push((format!("f.{}.facet.sort", self.field), sort.to_string()));
        }

        if let Some(limit) = &self.limit {
            result.push((format!("f.{}.facet.limit", self.field), limit.to_string()));
        }

        if let Some(offset) = &self.offset {
            result.push((format!("f.{}.facet.offset", self.field), offset.to_string()));
        }

        if let Some(min_count) = &self.min_count {
            result.push((
                format!("f.{}.facet.mincount", self.field),
                min_count.to_string(),
            ));
        }

        if let Some(missing) = &self.missing {
            result.push((
                format!("f.{}.facet.missing", self.field),
                missing.to_string(),
            ));
        }

        if let Some(method) = &self.method {
            result.push((format!("f.{}.facet.method", self.field), method.to_string()));
        }

        if let Some(exists) = &self.exists {
            result.push((format!("f.{}.facet.exists", self.field), exists.to_string()));
        }

        result
    }
}

pub enum RangeFacetOtherOptions {
    Before,
    After,
    Between,
    All,
    None,
}

pub enum RangeFacetIncludeOptions {
    Lower,
    Upper,
    Edge,
    Outer,
    All,
}

/// Implementation of the builder generates parameters for range facetting.
pub struct RangeFacetBuilder {
    field: String,
    start: String,
    end: String,
    gap: String,
    hardend: Option<bool>,
    other: Option<RangeFacetOtherOptions>,
    include: Option<RangeFacetIncludeOptions>,
    local_params: Vec<(String, String)>,
}

impl RangeFacetBuilder {
    pub fn new(field: &str, start: impl ToString, end: impl ToString, gap: impl ToString) -> Self {
        Self {
            field: field.to_string(),
            start: start.to_string(),
            end: end.to_string(),
            gap: gap.to_string(),
            hardend: None,
            other: None,
            include: None,
            local_params: Vec::new(),
        }
    }

    /// Add `f.<FIELD_NAME>.facet.range.hardend` parameter.
    pub fn hardend(mut self, hardend: bool) -> Self {
        self.hardend = Some(hardend);
        self
    }

    /// Add `f.<FIELD_NAME>.facet.range.other` parameter.
    pub fn other(mut self, other: RangeFacetOtherOptions) -> Self {
        self.other = Some(other);
        self
    }

    /// Add `f.<FIELD_NAME>.facet.range.include` parameter.
    pub fn include(mut self, include: RangeFacetIncludeOptions) -> Self {
        self.include = Some(include);
        self
    }

    /// Add a local parameter for faceting.
    pub fn local_param(mut self, key: &str, value: &str) -> Self {
        self.local_params.push((key.to_string(), value.to_string()));
        self
    }
}

impl FacetBuilder for RangeFacetBuilder {
    fn build(&self) -> Vec<(String, String)> {
        let mut result = Vec::new();

        if self.local_params.len() == 0 {
            result.push((String::from("facet.range"), self.field.clone()));
        } else {
            let local_param = format!(
                "{{!{}}}",
                self.local_params
                    .iter()
                    .map(|(key, value)| format!("{}={}", key, value))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            result.push((
                String::from("facet.range"),
                format!("{}{}", local_param, self.field),
            ));
        }

        result.push((
            format!("f.{}.facet.range.start", self.field),
            self.start.clone(),
        ));
        result.push((
            format!("f.{}.facet.range.end", self.field),
            self.end.clone(),
        ));
        result.push((
            format!("f.{}.facet.range.gap", self.field),
            self.gap.clone(),
        ));

        if let Some(hardend) = self.hardend {
            result.push((
                format!("f.{}.facet.hardend", self.field),
                hardend.to_string(),
            ))
        }

        if let Some(other) = &self.other {
            result.push((
                format!("f.{}.facet.range.other", self.field),
                match other {
                    RangeFacetOtherOptions::None => String::from("none"),
                    RangeFacetOtherOptions::Before => String::from("before"),
                    RangeFacetOtherOptions::After => String::from("after"),
                    RangeFacetOtherOptions::Between => String::from("between"),
                    RangeFacetOtherOptions::All => String::from("all"),
                },
            ));
        }

        if let Some(include) = &self.include {
            result.push((
                format!("f.{}.facet.range.include", self.field),
                match include {
                    RangeFacetIncludeOptions::Lower => String::from("lower"),
                    RangeFacetIncludeOptions::Upper => String::from("upper"),
                    RangeFacetIncludeOptions::Edge => String::from("edge"),
                    RangeFacetIncludeOptions::Outer => String::from("outer"),
                    RangeFacetIncludeOptions::All => String::from("all"),
                },
            ));
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use itertools::{sorted, Itertools};

    #[test]
    fn test_simple_field_facet() {
        let builder = FieldFacetBuilder::new("category");

        assert_eq!(
            builder.build(),
            vec![(String::from("facet.field"), String::from("category"))],
        );
    }

    #[test]
    fn test_field_facet_with_all_params() {
        let builder = FieldFacetBuilder::new("category")
            .prefix("A")
            .contains("like")
            .ignore_case(true)
            .sort(FieldFacetSortOrder::Count)
            .limit(100)
            .offset(0)
            .min_count(1)
            .missing(false)
            .method(FieldFacetMethod::Fc)
            .exists(false);
        let actual = sorted(builder.build()).collect_vec();
        let expected = sorted(
            vec![
                ("facet.field", "category"),
                ("f.category.facet.prefix", "A"),
                ("f.category.facet.contains", "like"),
                ("f.category.facet.contains.ignoreCase", "true"),
                ("f.category.facet.sort", "count"),
                ("f.category.facet.limit", "100"),
                ("f.category.facet.offset", "0"),
                ("f.category.facet.mincount", "1"),
                ("f.category.facet.missing", "false"),
                ("f.category.facet.method", "fc"),
                ("f.category.facet.exists", "false"),
            ]
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string())),
        )
        .collect_vec();

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_field_facet_with_local_params() {
        let builder = FieldFacetBuilder::new("category")
            .min_count(0)
            .local_param("ex", "dt");
        let expected = vec![
            ("facet.field", "{!ex=dt}category"),
            ("f.category.facet.mincount", "0"),
        ]
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect_vec();
        assert_eq!(builder.build(), expected);
    }

    #[test]
    fn test_range_facet() {
        let builder = RangeFacetBuilder::new("difficulty", 0, 2000, 400)
            .include(RangeFacetIncludeOptions::Lower)
            .other(RangeFacetOtherOptions::All);
        let actual = sorted(builder.build()).collect_vec();
        let expected = sorted(
            vec![
                ("facet.range", "difficulty"),
                ("f.difficulty.facet.range.start", "0"),
                ("f.difficulty.facet.range.end", "2000"),
                ("f.difficulty.facet.range.gap", "400"),
                ("f.difficulty.facet.range.other", "all"),
                ("f.difficulty.facet.range.include", "lower"),
            ]
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string())),
        )
        .collect_vec();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_range_facet_with_local_params() {
        let builder = RangeFacetBuilder::new("difficulty", 0, 2000, 400).local_param("ex", "dt");
        let actual = sorted(builder.build()).collect_vec();
        let expected = sorted(
            vec![
                ("facet.range", "{!ex=dt}difficulty"),
                ("f.difficulty.facet.range.start", "0"),
                ("f.difficulty.facet.range.end", "2000"),
                ("f.difficulty.facet.range.gap", "400"),
            ]
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string())),
        )
        .collect_vec();

        assert_eq!(actual, expected);
    }
}
