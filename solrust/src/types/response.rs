use crate::types::datetime::SolrDateTime;
/// 命名規則
///
/// - Solrから返ってくるレスポンス本体のモデル -> SolrXXXXResponse
/// - レスポンスの一部 -> SolrXXXX(Header|Body|Info|e.t.c)
///
use chrono::{DateTime, FixedOffset};
use itertools::Itertools;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrResponseHeader {
    pub status: u32,
    #[serde(alias = "QTime")]
    pub qtime: u32,
    pub params: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrErrorInfo {
    pub metadata: Vec<String>,
    pub msg: String,
    pub code: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LuceneInfo {
    #[serde(alias = "solr-spec-version")]
    pub solr_spec_version: String,
    #[serde(alias = "solr-impl-version")]
    pub solr_impl_version: String,
    #[serde(alias = "lucene-spec-version")]
    pub lucene_spec_version: String,
    #[serde(alias = "lucene-impl-version")]
    pub lucene_impl_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrSystemInfo {
    #[serde(alias = "responseHeader")]
    pub header: SolrResponseHeader,
    pub mode: String,
    pub solr_home: String,
    pub core_root: String,
    pub lucene: LuceneInfo,
    pub jvm: Value,
    pub security: Value,
    pub system: Value,
    pub error: Option<SolrErrorInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolrIndexInfo {
    #[serde(alias = "numDocs")]
    pub num_docs: u64,
    #[serde(alias = "maxDoc")]
    pub max_doc: u64,
    #[serde(alias = "deletedDocs")]
    pub deleted_docs: u64,
    pub version: u64,
    #[serde(alias = "segmentCount")]
    pub segment_count: u64,
    pub current: bool,
    #[serde(alias = "hasDeletions")]
    pub has_deletions: bool,
    pub directory: String,
    #[serde(alias = "segmentsFile")]
    pub segments_file: String,
    #[serde(alias = "segmentsFileSizeInBytes")]
    pub segments_file_size_in_bytes: u64,
    #[serde(alias = "userData")]
    pub user_data: Value,
    #[serde(alias = "sizeInBytes")]
    pub size_in_bytes: u64,
    pub size: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolrCoreStatus {
    pub name: String,
    #[serde(alias = "instanceDir")]
    pub instance_dir: String,
    #[serde(alias = "dataDir")]
    pub data_dir: String,
    pub config: String,
    pub schema: String,
    #[serde(alias = "startTime")]
    pub start_time: String,
    pub uptime: u64,
    pub index: SolrIndexInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrCoreList {
    #[serde(alias = "responseHeader")]
    pub header: SolrResponseHeader,
    #[serde(alias = "initFailures")]
    pub init_failures: Value,
    pub status: Option<HashMap<String, SolrCoreStatus>>,
    pub error: Option<SolrErrorInfo>,
}

impl SolrCoreList {
    pub fn as_vec(&self) -> Option<Vec<String>> {
        if let Some(cores) = &self.status {
            Some(cores.keys().cloned().collect())
        } else {
            return None;
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrSimpleResponse {
    #[serde(alias = "responseHeader")]
    pub header: SolrResponseHeader,
    pub error: Option<SolrErrorInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrSelectResponse<T> {
    #[serde(alias = "responseHeader")]
    pub header: SolrResponseHeader,
    pub response: SolrSelectBody<T>,
    pub facet_counts: Option<SolrFacetBody>,
    pub error: Option<SolrErrorInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrSelectBody<T> {
    #[serde(alias = "numFound")]
    pub num_found: u32,
    pub start: u32,
    #[serde(alias = "numFoundExact")]
    pub num_found_exact: bool,
    // TODO: ジェネリクス化
    pub docs: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrFacetBody {
    pub facet_queries: Value,
    #[serde(deserialize_with = "deserialize_facet_fields")]
    pub facet_fields: HashMap<String, Vec<(String, u32)>>,
    #[serde(deserialize_with = "deserialize_facet_ranges")]
    pub facet_ranges: HashMap<String, SolrRangeFacetKind>,
    pub facet_intervals: Value,
    pub facet_heatmaps: Value,
}

fn deserialize_facet_fields<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Vec<(String, u32)>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: HashMap<String, Vec<Value>> = Deserialize::deserialize(deserializer)?;
    let value: HashMap<String, Vec<(String, u32)>> = value
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                v.iter()
                    .tuples()
                    .map(|(v1, v2)| {
                        (
                            v1.as_str().unwrap_or("").to_string(),
                            v2.as_u64().unwrap_or(0) as u32,
                        )
                    })
                    .collect::<Vec<(String, u32)>>(),
            )
        })
        .collect();

    Ok(value)
}

/// レンジファセットの結果をデシリアライズする関数
///
/// レンジファセットの結果の型はフィールドの型依存なのでアドホックに場合分けする必要があった
fn deserialize_facet_ranges<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, SolrRangeFacetKind>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: HashMap<String, Value> = Deserialize::deserialize(deserializer)?;
    let mut result: HashMap<String, SolrRangeFacetKind> = HashMap::new();
    for (field, value) in value.iter() {
        match &value["start"] {
            Value::Number(start) => {
                if start.is_i64() {
                    let value: SolrIntegerRangeFacet = serde_json::from_value(value.clone())
                        .map_err(|e| {
                            D::Error::custom(format!(
                                "Failed to parse integer range facet result. [{}]",
                                e.to_string()
                            ))
                        })?;
                    result.insert(field.to_string(), SolrRangeFacetKind::Integer(value));
                } else {
                    let value: SolrFloatRangeFacet = serde_json::from_value(value.clone())
                        .map_err(|e| {
                            D::Error::custom(format!(
                                "Failed to parse float range facet result. [{}]",
                                e.to_string()
                            ))
                        })?;
                    result.insert(field.to_string(), SolrRangeFacetKind::Float(value));
                }
            }
            Value::String(start) => {
                if DateTime::parse_from_rfc3339(&start.replace("Z", "+00:00")).is_ok() {
                    let value: SolrDateTimeRangeFacet = serde_json::from_value(value.clone())
                        .map_err(|e| {
                            D::Error::custom(format!(
                                "Failed to parse datetime range facet result. [{}]",
                                e.to_string()
                            ))
                        })?;
                    result.insert(field.to_string(), SolrRangeFacetKind::DateTime(value));
                } else {
                    // TODO; 数値、日付型以外のレンジファセットがあったら処理を追加する
                    return Err(D::Error::custom("Unexpected range facet value type."));
                }
            }
            _ => {
                return Err(D::Error::custom("Mismatched range facet value type."));
            }
        }
    }
    Ok(result)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SolrRangeFacetKind {
    Integer(SolrIntegerRangeFacet),
    Float(SolrFloatRangeFacet),
    DateTime(SolrDateTimeRangeFacet),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrIntegerRangeFacet {
    #[serde(deserialize_with = "deserialize_range_facet_counts")]
    pub counts: Vec<(String, u32)>,
    pub gap: i64,
    pub start: i64,
    pub end: i64,
    pub before: Option<i64>,
    pub after: Option<i64>,
    pub between: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrFloatRangeFacet {
    #[serde(deserialize_with = "deserialize_range_facet_counts")]
    pub counts: Vec<(String, u32)>,
    pub gap: f64,
    pub start: f64,
    pub end: f64,
    pub before: Option<f64>,
    pub after: Option<f64>,
    pub between: Option<f64>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct SolrDateTimeRangeFacet {
    #[serde(deserialize_with = "deserialize_range_facet_counts")]
    pub counts: Vec<(String, u32)>,
    pub gap: String,
    #[serde_as(as = "SolrDateTime")]
    pub start: DateTime<FixedOffset>,
    #[serde_as(as = "SolrDateTime")]
    pub end: DateTime<FixedOffset>,
    #[serde(default)]
    #[serde_as(as = "Option<SolrDateTime>")]
    pub before: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    #[serde_as(as = "Option<SolrDateTime>")]
    pub after: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    #[serde_as(as = "Option<SolrDateTime>")]
    pub between: Option<DateTime<FixedOffset>>,
}

/// ファセットの結果の配列をRustが扱える配列にデシリアライズする関数
///
/// Solrのファセットの結果は「文字列、数値」が交互に格納された配列で返ってくる。Rustは型が混じった配列を扱えないので、タプルのリストに変換する。
fn deserialize_range_facet_counts<'de, D>(deserializer: D) -> Result<Vec<(String, u32)>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Vec<Value> = Deserialize::deserialize(deserializer)?;
    let value: Vec<(String, u32)> = value
        .iter()
        .tuples()
        .map(|(v1, v2)| {
            (
                v1.as_str().unwrap_or("").to_string(),
                v2.as_u64().unwrap_or(0) as u32,
            )
        })
        .collect();

    Ok(value)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrAnalysisBody {
    pub field_types: HashMap<String, SolrAnalysisField>,
    pub field_names: HashMap<String, SolrAnalysisField>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrAnalysisField {
    pub index: Option<Vec<Value>>,
    pub query: Option<Vec<Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolrAnalysisResponse {
    #[serde(alias = "responseHeader")]
    pub header: SolrResponseHeader,
    pub analysis: SolrAnalysisBody,
    pub error: Option<SolrErrorInfo>,
}

#[cfg(test)]
mod test {
    use super::*;
    // TODO: テスト書く

    #[test]
    fn test_deserialize_response_header() {
        let raw = r#"
        {
            "status": 400,
            "QTime": 7,
            "params": {
                "facet.range": "difficulty",
                "q": "text_ja:高橋",
                "facet.field": "category",
                "f.difficulty.facet.end": "2000",
                "f.category.facet.mincount": "1",
                "f.difficulty.facet.start": "0",
                "facet": "true",
                "f.difficulty.gap": "800"
            }
        }
        "#;
        let header: SolrResponseHeader = serde_json::from_str(raw).unwrap();
        assert_eq!(header.status, 400);
        assert_eq!(header.qtime, 7);
    }

    #[test]
    fn test_deserialize_error_info() {
        let raw = r#"
        {
            "metadata": [
                "error-class",
                "org.apache.solr.common.SolrException",
                "root-error-class",
                "org.apache.solr.common.SolrException"
            ],
            "msg": "Missing required parameter: f.difficulty.facet.range.start (or default: facet.range.start)",
            "code": 400
        }
        "#;

        let error: SolrErrorInfo = serde_json::from_str(raw).unwrap();
        assert_eq!(error.msg, "Missing required parameter: f.difficulty.facet.range.start (or default: facet.range.start)".to_string());
        assert_eq!(error.code, 400);
    }

    #[test]
    fn test_deserialize_lucene_info() {
        let raw = r#"
        {
            "solr-spec-version": "9.1.0",
            "solr-impl-version": "9.1.0 aa4f3d98ab19c201e7f3c74cd14c99174148616d - ishan - 2022-11-11 13:00:47",
            "lucene-spec-version": "9.3.0",
            "lucene-impl-version": "9.3.0 d25cebcef7a80369f4dfb9285ca7360a810b75dc - ivera - 2022-07-25 12:30:23"
        }
        "#;

        let info: LuceneInfo = serde_json::from_str(raw).unwrap();
        assert_eq!(info.solr_spec_version, "9.1.0".to_string());
    }

    #[test]
    fn test_deserialize_solr_system_info() {
        let raw = r#"
        {
            "responseHeader": {
                "status": 0,
                "QTime": 17
            },
            "mode": "std",
            "solr_home": "/var/solr/data",
            "core_root": "/var/solr/data",
            "lucene": {
                "solr-spec-version": "9.1.0",
                "solr-impl-version": "9.1.0 aa4f3d98ab19c201e7f3c74cd14c99174148616d - ishan - 2022-11-11 13:00:47",
                "lucene-spec-version": "9.3.0",
                "lucene-impl-version": "9.3.0 d25cebcef7a80369f4dfb9285ca7360a810b75dc - ivera - 2022-07-25 12:30:23"
            },
            "jvm": {
                "version": "17.0.5 17.0.5+8",
                "name": "Eclipse Adoptium OpenJDK 64-Bit Server VM",
                "spec": {
                "vendor": "Oracle Corporation",
                "name": "Java Platform API Specification",
                "version": "17"
                },
                "jre": {
                "vendor": "Eclipse Adoptium",
                "version": "17.0.5"
                },
                "vm": {
                "vendor": "Eclipse Adoptium",
                "name": "OpenJDK 64-Bit Server VM",
                "version": "17.0.5+8"
                },
                "processors": 16,
                "memory": {
                "free": "410.9 MB",
                "total": "512 MB",
                "max": "512 MB",
                "used": "101.1 MB (%19.7)",
                "raw": {
                    "free": 430868656,
                    "total": 536870912,
                    "max": 536870912,
                    "used": 106002256,
                    "used%": 19.74445879459381
                }
                },
                "jmx": {
                "classpath": "start.jar",
                "commandLineArgs": [
                    "-Xms512m",
                    "-Xmx512m",
                    "-XX:+UseG1GC",
                    "-XX:+PerfDisableSharedMem",
                    "-XX:+ParallelRefProcEnabled",
                    "-XX:MaxGCPauseMillis=250",
                    "-XX:+UseLargePages",
                    "-XX:+AlwaysPreTouch",
                    "-XX:+ExplicitGCInvokesConcurrent",
                    "-Xlog:gc*:file=/var/solr/logs/solr_gc.log:time,uptime:filecount=9,filesize=20M",
                    "-Dsolr.jetty.inetaccess.includes=",
                    "-Dsolr.jetty.inetaccess.excludes=",
                    "-Dsolr.log.dir=/var/solr/logs",
                    "-Djetty.port=8983",
                    "-DSTOP.PORT=7983",
                    "-DSTOP.KEY=solrrocks",
                    "-Duser.timezone=UTC",
                    "-XX:-OmitStackTraceInFastThrow",
                    "-XX:OnOutOfMemoryError=/opt/solr/bin/oom_solr.sh 8983 /var/solr/logs",
                    "-Djetty.home=/opt/solr/server",
                    "-Dsolr.solr.home=/var/solr/data",
                    "-Dsolr.data.home=",
                    "-Dsolr.install.dir=/opt/solr",
                    "-Dsolr.default.confdir=/opt/solr/server/solr/configsets/_default/conf",
                    "-Dlog4j.configurationFile=/var/solr/log4j2.xml",
                    "-Dsolr.jetty.host=0.0.0.0",
                    "-Xss256k",
                    "-XX:CompileCommand=exclude,com.github.benmanes.caffeine.cache.BoundedLocalCache::put",
                    "-Djava.security.manager",
                    "-Djava.security.policy=/opt/solr/server/etc/security.policy",
                    "-Djava.security.properties=/opt/solr/server/etc/security.properties",
                    "-Dsolr.internal.network.permission=*",
                    "-DdisableAdminUI=false"
                ],
                "startTime": "2023-01-26T14:06:26.026Z",
                "upTimeMS": 47574
                }
            },
            "security": {},
            "system": {
                "name": "Linux",
                "arch": "amd64",
                "availableProcessors": 16,
                "systemLoadAverage": 1.88,
                "version": "5.15.0-58-generic",
                "committedVirtualMemorySize": 6041583616,
                "cpuLoad": 0.0625,
                "freeMemorySize": 153268224,
                "freePhysicalMemorySize": 153268224,
                "freeSwapSpaceSize": 8422940672,
                "processCpuLoad": 0.5,
                "processCpuTime": 11970000000,
                "systemCpuLoad": 0,
                "totalMemorySize": 7512129536,
                "totalPhysicalMemorySize": 7512129536,
                "totalSwapSpaceSize": 10737410048,
                "maxFileDescriptorCount": 1048576,
                "openFileDescriptorCount": 156
            }
            }
        "#;

        let info: SolrSystemInfo = serde_json::from_str(raw).unwrap();
        assert_eq!(info.header.qtime, 17);
    }

    #[test]
    fn test_deserialize_index_info() {
        let raw = r#"
        {
            "numDocs": 0,
            "maxDoc": 0,
            "deletedDocs": 0,
            "version": 2,
            "segmentCount": 0,
            "current": true,
            "hasDeletions": false,
            "directory": "org.apache.lucene.store.NRTCachingDirectory:NRTCachingDirectory(MMapDirectory@/var/solr/data/atcoder/data/index lockFactory=org.apache.lucene.store.NativeFSLockFactory@404f935c; maxCacheMB=48.0 maxMergeSizeMB=4.0)",
            "segmentsFile": "segments_1",
            "segmentsFileSizeInBytes": 69,
            "userData": {},
            "sizeInBytes": 69,
            "size": "69 bytes"
        }
        "#;
        let info: SolrIndexInfo = serde_json::from_str(raw).unwrap();
        assert_eq!(info.num_docs, 0);
    }

    #[test]
    fn test_deserialize_core_list() {
        let raw = r#"
        {
            "responseHeader": {
                "status": 0,
                "QTime": 1
            },
            "initFailures": {},
            "status": {
                "atcoder": {
                "name": "atcoder",
                "instanceDir": "/var/solr/data/atcoder",
                "dataDir": "/var/solr/data/atcoder/data/",
                "config": "solrconfig.xml",
                "schema": "schema.xml",
                "startTime": "2023-01-26T14:06:28.956Z",
                "uptime": 321775,
                "index": {
                    "numDocs": 0,
                    "maxDoc": 0,
                    "deletedDocs": 0,
                    "version": 2,
                    "segmentCount": 0,
                    "current": true,
                    "hasDeletions": false,
                    "directory": "org.apache.lucene.store.NRTCachingDirectory:NRTCachingDirectory(MMapDirectory@/var/solr/data/atcoder/data/index lockFactory=org.apache.lucene.store.NativeFSLockFactory@404f935c; maxCacheMB=48.0 maxMergeSizeMB=4.0)",
                    "segmentsFile": "segments_1",
                    "segmentsFileSizeInBytes": 69,
                    "userData": {},
                    "sizeInBytes": 69,
                    "size": "69 bytes"
                }
                }
            }
        }
        "#;
        let info: SolrCoreList = serde_json::from_str(raw).unwrap();

        assert_eq!(info.as_vec().unwrap(), vec![String::from("atcoder")]);
    }

    #[test]
    fn test_deserialize_simple_response() {
        let raw = r#"
        {
            "responseHeader": {
                "status": 0,
                "QTime": 181
            }
        }
        "#;

        let response: SolrSimpleResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(response.header.qtime, 181);
    }

    #[test]
    fn test_deserialize_simple_response_with_error() {
        let raw = r#"
        {
            "responseHeader": {
                "status": 400,
                "QTime": 0
            },
            "error": {
                "metadata": [
                "error-class",
                "org.apache.solr.common.SolrException",
                "root-error-class",
                "org.apache.solr.common.SolrException"
                ],
                "msg": "No such core: hoge",
                "code": 400
            }
        }
        "#;

        let response: SolrSimpleResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(response.error.unwrap().code, 400);
    }

    #[allow(dead_code)]
    #[serde_as]
    #[derive(Deserialize)]
    struct Document {
        problem_id: String,
        problem_title: String,
        problem_url: String,
        contest_id: String,
        contest_title: String,
        contest_url: String,
        difficulty: i64,
        #[serde_as(as = "SolrDateTime")]
        start_at: DateTime<FixedOffset>,
        duration: i64,
        rate_change: String,
        category: String,
    }

    #[test]
    fn test_deserialize_select_body() {
        let raw = r#"
        {
            "numFound": 5650,
            "start": 0,
            "numFoundExact": true,
            "docs": [
                {
                    "problem_id": "APG4b_a",
                    "problem_title": "A. 1.00.はじめに",
                    "problem_url": "https://atcoder.jp/contests/APG4b/tasks/APG4b_a",
                    "contest_id": "APG4b",
                    "contest_title": "C++入門 AtCoder Programming Guide for beginners (APG4b)",
                    "contest_url": "https://atcoder.jp/contests/APG4b",
                    "difficulty": 0,
                    "start_at": "1970-01-01T00:00:00Z",
                    "duration": -1141367296,
                    "rate_change": "-",
                    "category": "Other Contests",
                    "_version_": 1756245857733181400
                }
            ]
        }
        "#;

        let body: SolrSelectBody<Document> = serde_json::from_str(raw).unwrap();
        assert_eq!(body.num_found, 5650);
    }

    #[test]
    fn test_deserialize_facet_counts() {
        let raw = r#"
        {
            "facet_queries": {},
            "facet_fields": {
                "category": [
                    "ABC",
                    400,
                    "ARC",
                    123,
                    "Other Sponsored",
                    95,
                    "AGC",
                    67,
                    "Other Contests",
                    41,
                    "ABC-Like",
                    38,
                    "PAST",
                    25,
                    "AGC-Like",
                    19,
                    "AHC",
                    12,
                    "Marathon",
                    5,
                    "ARC-Like",
                    1
                ]
            },
            "facet_ranges": {
                "difficulty": {
                    "counts": [
                        "0",
                        210,
                        "400",
                        69,
                        "800",
                        56,
                        "1200",
                        77,
                        "1600",
                        74
                    ],
                    "gap": 400,
                    "before": 140,
                    "after": 200,
                    "between": 486,
                    "start": 0,
                    "end": 2000
                },
                "start_at": {
                    "counts": [
                        "2021-10-10T00:00:00Z",
                        19,
                        "2021-12-10T00:00:00Z",
                        26,
                        "2022-02-10T00:00:00Z",
                        30,
                        "2022-04-10T00:00:00Z",
                        15,
                        "2022-06-10T00:00:00Z",
                        23,
                        "2022-08-10T00:00:00Z",
                        18
                    ],
                    "gap": "+2MONTHS",
                    "start": "2021-10-10T00:00:00Z",
                    "end": "2022-10-10T00:00:00Z"
                }
            },
            "facet_intervals": {},
            "facet_heatmaps": {}
        }
        "#;

        let facet: SolrFacetBody = serde_json::from_str(raw).unwrap();
        assert!(facet.facet_fields.contains_key("category"));
    }

    #[test]
    fn test_deserialize_select_response() {
        let raw = r#"
        {
            "responseHeader": {
                "status": 0,
                "QTime": 27,
                "params": {}
            },
            "response": {
                "numFound": 0,
                "start": 0,
                "numFoundExact": true,
                "docs": []
            }
        }
        "#;
        let select: SolrSelectResponse<Document> = serde_json::from_str(raw).unwrap();
        assert_eq!(select.response.num_found, 0);
    }
}
