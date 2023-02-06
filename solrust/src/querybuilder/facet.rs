/// ファセット検索のためのクエリパラメータを生成するビルダのトレイト
pub trait FacetBuilder {
    fn build(&self) -> Vec<(String, String)>;
}

/// フィールドファセットのソート順
///
/// - Index: ファセット結果を辞書順で返す
/// - Count: ファセット結果をカウント数順で返す
///
/// https://solr.apache.org/guide/solr/latest/query-guide/faceting.html#field-value-faceting-parameters:~:text=a%20regular%20expression.-,facet.sort,-Optional
pub enum FieldFacetSortOrder {
    Index,
    Count,
}

/// フィールドファセット時にSolrが使用するアルゴリズム
///
/// https://solr.apache.org/guide/solr/latest/query-guide/faceting.html#field-value-faceting-parameters:~:text=in%20the%20response.-,facet.method,-Optional
pub enum FieldFacetMethod {
    Enum,
    Fc,
    Fcs,
}

/// フィールドファセットのためのパラメータを生成するビルダ
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
        }
    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    pub fn contains(mut self, contains: &str) -> Self {
        self.contains = Some(contains.to_string());
        self
    }

    pub fn ignore_case(mut self, ignore_case: bool) -> Self {
        self.ignore_case = Some(ignore_case);
        self
    }

    pub fn sort(mut self, sort: FieldFacetSortOrder) -> Self {
        self.sort = Some(match sort {
            FieldFacetSortOrder::Count => "count".to_string(),
            FieldFacetSortOrder::Index => "index".to_string(),
        });
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn min_count(mut self, min_count: u32) -> Self {
        self.min_count = Some(min_count);
        self
    }

    pub fn missing(mut self, missing: bool) -> Self {
        self.missing = Some(missing);
        self
    }

    pub fn method(mut self, method: FieldFacetMethod) -> Self {
        self.method = Some(match method {
            FieldFacetMethod::Enum => "enum".to_string(),
            FieldFacetMethod::Fc => "fc".to_string(),
            FieldFacetMethod::Fcs => "fcs".to_string(),
        });
        self
    }

    pub fn exists(mut self, exists: bool) -> Self {
        self.exists = Some(exists);
        self
    }
}

impl FacetBuilder for FieldFacetBuilder {
    fn build(&self) -> Vec<(String, String)> {
        let mut result: Vec<(String, String)> = Vec::new();

        result.push((String::from("facet.field"), self.field.clone()));

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

/// レンジの境界値の扱いを制御するパラメータ
///
/// - Lower: レンジを右半開区間で扱う
/// - Upper: レンジを左半開区間で扱う
/// - Edge: 一番最初の区間の下限境界値と、一番最後の区間の上限境界値を含む
/// - Outer: 境界値はどちらの区間にも含まれる？
/// - All: すべてのオプションを適用する
pub enum RangeFacetIncludeOptions {
    Lower,
    Upper,
    Edge,
    Outer,
    All,
}

/// レンジファセットのためのパラメータを生成するビルダ
pub struct RangeFacetBuilder {
    field: String,
    start: String,
    end: String,
    gap: String,
    hardend: Option<bool>,
    other: Option<RangeFacetOtherOptions>,
    include: Option<RangeFacetIncludeOptions>,
}

impl RangeFacetBuilder {
    pub fn new(field: &str, start: String, end: String, gap: String) -> Self {
        Self {
            field: field.to_string(),
            start: start,
            end: end,
            gap: gap,
            hardend: None,
            other: None,
            include: None,
        }
    }

    pub fn hardend(mut self, hardend: bool) -> Self {
        self.hardend = Some(hardend);
        self
    }

    pub fn other(mut self, other: RangeFacetOtherOptions) -> Self {
        self.other = Some(other);
        self
    }

    pub fn include(mut self, include: RangeFacetIncludeOptions) -> Self {
        self.include = Some(include);
        self
    }
}

impl FacetBuilder for RangeFacetBuilder {
    fn build(&self) -> Vec<(String, String)> {
        let mut result = Vec::new();

        result.push((String::from("facet.range"), self.field.clone()));
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

    #[test]
    fn test_simple_field_facet() {
        let builder = FieldFacetBuilder::new("category");

        assert_eq!(
            vec![(String::from("facet.field"), String::from("category"))],
            builder.build()
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

        assert_eq!(
            vec![
                (String::from("facet.field"), String::from("category")),
                (String::from("f.category.facet.prefix"), String::from("A")),
                (
                    String::from("f.category.facet.contains"),
                    String::from("like")
                ),
                (
                    String::from("f.category.facet.contains.ignoreCase"),
                    String::from("true")
                ),
                (String::from("f.category.facet.sort"), String::from("count")),
                (String::from("f.category.facet.limit"), String::from("100")),
                (String::from("f.category.facet.offset"), String::from("0")),
                (String::from("f.category.facet.mincount"), String::from("1")),
                (
                    String::from("f.category.facet.missing"),
                    String::from("false")
                ),
                (String::from("f.category.facet.method"), String::from("fc")),
                (
                    String::from("f.category.facet.exists"),
                    String::from("false")
                ),
            ],
            builder.build()
        )
    }

    #[test]
    fn test_range_facet() {
        let builder = RangeFacetBuilder::new(
            "difficulty",
            0.to_string(),
            2000.to_string(),
            400.to_string(),
        )
        .include(RangeFacetIncludeOptions::Lower)
        .other(RangeFacetOtherOptions::All);

        assert_eq!(
            vec![
                (String::from("facet.range"), String::from("difficulty")),
                (
                    String::from("f.difficulty.facet.range.start"),
                    String::from("0")
                ),
                (
                    String::from("f.difficulty.facet.range.end"),
                    String::from("2000")
                ),
                (
                    String::from("f.difficulty.facet.range.gap"),
                    String::from("400")
                ),
                (
                    String::from("f.difficulty.facet.range.other"),
                    String::from("all")
                ),
                (
                    String::from("f.difficulty.facet.range.include"),
                    String::from("lower")
                ),
            ],
            builder.build()
        )
    }
}
