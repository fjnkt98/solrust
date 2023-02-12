use chrono::{DateTime, FixedOffset, Local, Utc};
use serde::Deserialize;
use serde_with::{DeserializeAs, SerializeAs};

pub struct SolrDateTime;

// ========================== DateTime<FixedOffset>の変換の実装 ============================

/// DateTime<FixedOffset>をシリアライズするための実装
/// UTCタイムゾーンに変換してから末尾の`+00:00`を`Z`に変換してシリアライズする
impl SerializeAs<DateTime<FixedOffset>> for SolrDateTime {
    fn serialize_as<S>(source: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            &source
                .with_timezone(&Utc)
                .to_rfc3339()
                .replace("+00:00", "Z"),
        )
    }
}

/// Solrの日付フォーマットをDateTime<FixedOffset>にデシリアライズする実装
/// Solrの日付フォーマットは末尾にZが付いたUTC時刻なので、末尾のZを`+00:00`に変換してからパースする
impl<'de> DeserializeAs<'de, DateTime<FixedOffset>> for SolrDateTime {
    fn deserialize_as<D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let timestamp = DateTime::parse_from_rfc3339(&value.replace("Z", "+00:00"))
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(timestamp)
    }
}

// =========================================================================================

// ========================== DateTime<Utc>の変換の実装 ============================
impl SerializeAs<DateTime<Utc>> for SolrDateTime {
    fn serialize_as<S>(source: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&source.to_rfc3339().replace("+00:00", "Z"))
    }
}

impl<'de> DeserializeAs<'de, DateTime<Utc>> for SolrDateTime {
    fn deserialize_as<D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let timestamp = DateTime::parse_from_rfc3339(&value.replace("Z", "+00:00"))
            .map_err(|e| serde::de::Error::custom(e.to_string()))?
            .with_timezone(&Utc);

        Ok(timestamp)
    }
}
// =================================================================================

// ========================== DateTime<Local>の変換の実装 ============================
impl SerializeAs<DateTime<Local>> for SolrDateTime {
    fn serialize_as<S>(source: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            &source
                .with_timezone(&Utc)
                .to_rfc3339()
                .replace("+00:00", "Z"),
        )
    }
}

impl<'de> DeserializeAs<'de, DateTime<Local>> for SolrDateTime {
    fn deserialize_as<D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let timestamp = value
            .parse::<DateTime<FixedOffset>>()
            .map_err(|e| serde::de::Error::custom(e.to_string()))?
            .with_timezone(&Local);
        Ok(timestamp)
    }
}
// ===================================================================================

#[cfg(test)]
mod test {
    use super::*;
    use chrono::offset::TimeZone;
    use serde::{Deserialize, Serialize};
    use serde_with::serde_as;

    // ====================== DateTime<FixedOffset>のテスト ===============================
    #[serde_as]
    #[derive(Debug, Serialize, Deserialize)]
    struct DocumentWithFixedDateTimeOffset {
        #[serde_as(as = "SolrDateTime")]
        start_at: DateTime<FixedOffset>,
    }

    #[test]
    fn test_serialize_fixed_offset_datetime() {
        let doc = DocumentWithFixedDateTimeOffset {
            start_at: DateTime::parse_from_rfc3339("2022-10-01T12:30:15+00:00").unwrap(),
        };

        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(json, r#"{"start_at":"2022-10-01T12:30:15Z"}"#);
    }

    #[test]
    fn test_serialize_fixed_offset_datetime_with_offset() {
        let doc = DocumentWithFixedDateTimeOffset {
            start_at: DateTime::parse_from_rfc3339("2022-10-01T12:30:15+09:00").unwrap(),
        };

        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(json, r#"{"start_at":"2022-10-01T03:30:15Z"}"#);
    }

    #[test]
    fn test_deserialize_fixed_offset_datetime() {
        let raw = r#"{"start_at": "2022-10-01T12:30:15Z"}"#;

        let doc: DocumentWithFixedDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert_eq!(
            doc.start_at,
            DateTime::parse_from_rfc3339("2022-10-01T12:30:15+00:00").unwrap()
        );
    }
    // ====================================================================================

    // ====================== Option<DateTime<FixedOffset>>のテスト ===============================
    #[serde_as]
    #[derive(Debug, Serialize, Deserialize)]
    struct DocumentWithOptionalFixedDateTimeOffset {
        #[serde(default)]
        #[serde_as(as = "Option<SolrDateTime>")]
        start_at: Option<DateTime<FixedOffset>>,
    }

    #[test]
    fn test_serialize_optional_fixed_offset_datetime() {
        let doc = DocumentWithOptionalFixedDateTimeOffset {
            start_at: Some(DateTime::parse_from_rfc3339("2022-10-01T12:30:15+09:00").unwrap()),
        };

        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(json, r#"{"start_at":"2022-10-01T03:30:15Z"}"#);
    }

    #[test]
    fn test_serialize_optional_fixed_offset_datetime_with_none() {
        let doc = DocumentWithOptionalFixedDateTimeOffset { start_at: None };

        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(json, r#"{"start_at":null}"#);
    }

    #[test]
    fn deserialize_optional_fixed_offset_datetime() {
        let raw = r#"{"start_at": "2022-10-01T12:30:15Z"}"#;

        let doc: DocumentWithOptionalFixedDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert_eq!(
            doc.start_at,
            Some(DateTime::parse_from_rfc3339("2022-10-01T12:30:15+00:00").unwrap())
        );
    }

    #[test]
    fn deserialize_optional_fixed_offset_datetime_with_null() {
        let raw = r#"{"start_at": null}"#;

        let doc: DocumentWithOptionalFixedDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert!(doc.start_at.is_none());
    }

    #[test]
    fn deserialize_optional_fixed_offset_datetime_without_field() {
        let raw = r#"{}"#;

        let doc: DocumentWithOptionalFixedDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert!(doc.start_at.is_none());
    }
    // ============================================================================================

    // ====================== DateTime<Utc>のテスト ===============================
    #[serde_as]
    #[derive(Debug, Serialize, Deserialize)]
    struct DocumentWithUtcDateTimeOffset {
        #[serde_as(as = "SolrDateTime")]
        start_at: DateTime<Utc>,
    }

    #[test]
    fn test_serialize_utc_datetime() {
        let doc = DocumentWithUtcDateTimeOffset {
            start_at: Utc
                .datetime_from_str("2022-10-01T12:30:15", "%Y-%m-%dT%H:%M:%S")
                .unwrap(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(json, r#"{"start_at":"2022-10-01T12:30:15Z"}"#)
    }

    #[test]
    fn test_deserialize_utc_datetime() {
        let raw = r#"{"start_at": "2022-10-01T12:30:15Z"}"#;
        let doc: DocumentWithUtcDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert_eq!(
            doc.start_at,
            Utc.datetime_from_str("2022-10-01T12:30:15", "%Y-%m-%dT%H:%M:%S")
                .unwrap()
        );
    }
    // ============================================================================

    // ====================== Option<DateTime<Utc>>のテスト ===============================
    #[serde_as]
    #[derive(Debug, Serialize, Deserialize)]
    struct DocumentWithOptionalUtcDateTimeOffset {
        #[serde(default)]
        #[serde_as(as = "Option<SolrDateTime>")]
        start_at: Option<DateTime<Utc>>,
    }

    #[test]
    fn test_serialize_optional_utc_datetime() {
        let doc = DocumentWithOptionalUtcDateTimeOffset {
            start_at: Some(
                Utc.datetime_from_str("2022-10-01T12:30:15", "%Y-%m-%dT%H:%M:%S")
                    .unwrap(),
            ),
        };
        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(json, r#"{"start_at":"2022-10-01T12:30:15Z"}"#)
    }

    #[test]
    fn test_serialize_optional_utc_datetime_with_none() {
        let doc = DocumentWithOptionalUtcDateTimeOffset { start_at: None };
        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(json, r#"{"start_at":null}"#)
    }

    #[test]
    fn test_deserialize_optional_utc_datetime() {
        let raw = r#"{"start_at": "2022-10-01T12:30:15Z"}"#;
        let doc: DocumentWithOptionalUtcDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert_eq!(
            doc.start_at,
            Some(
                Utc.datetime_from_str("2022-10-01T12:30:15", "%Y-%m-%dT%H:%M:%S")
                    .unwrap()
            )
        );
    }

    #[test]
    fn test_deserialize_optional_utc_datetime_with_null() {
        let raw = r#"{"start_at": null}"#;
        let doc: DocumentWithOptionalUtcDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert!(doc.start_at.is_none(),);
    }

    #[test]
    fn test_deserialize_optional_utc_datetime_without_field() {
        let raw = r#"{}"#;
        let doc: DocumentWithOptionalUtcDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert!(doc.start_at.is_none(),);
    }

    // ============================================================================

    // ====================== DateTime<Local>のテスト ===============================
    #[serde_as]
    #[derive(Debug, Serialize, Deserialize)]
    struct DocumentWithLocalDateTimeOffset {
        #[serde_as(as = "SolrDateTime")]
        start_at: DateTime<Local>,
    }

    #[test]
    fn test_serialize_local_datetime() {
        let doc = DocumentWithLocalDateTimeOffset {
            start_at: Local
                .datetime_from_str("2022-10-01T12:30:15", "%Y-%m-%dT%H:%M:%S")
                .unwrap(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(json, r#"{"start_at":"2022-10-01T03:30:15Z"}"#)
    }

    #[test]
    fn test_deserialize_local_datetime() {
        let raw = r#"{"start_at": "2022-10-01T03:30:15Z"}"#;
        let doc: DocumentWithLocalDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert_eq!(
            doc.start_at,
            Local
                .datetime_from_str("2022-10-01T12:30:15", "%Y-%m-%dT%H:%M:%S")
                .unwrap()
        )
    }
    // ==============================================================================

    // ====================== Option<DateTime<Local>>のテスト ===============================
    #[serde_as]
    #[derive(Debug, Serialize, Deserialize)]
    struct DocumentWithOptionalLocalDateTimeOffset {
        #[serde(default)]
        #[serde_as(as = "Option<SolrDateTime>")]
        start_at: Option<DateTime<Local>>,
    }

    #[test]
    fn test_serialize_optional_local_datetime() {
        let doc = DocumentWithOptionalLocalDateTimeOffset {
            start_at: Some(
                Local
                    .datetime_from_str("2022-10-01T12:30:15", "%Y-%m-%dT%H:%M:%S")
                    .unwrap(),
            ),
        };
        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(json, r#"{"start_at":"2022-10-01T03:30:15Z"}"#)
    }

    #[test]
    fn test_serialize_optional_local_datetime_with_none() {
        let doc = DocumentWithOptionalLocalDateTimeOffset { start_at: None };
        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(json, r#"{"start_at":null}"#)
    }

    #[test]
    fn test_deserialize_optional_local_datetime() {
        let raw = r#"{"start_at": "2022-10-01T03:30:15Z"}"#;
        let doc: DocumentWithOptionalLocalDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert_eq!(
            doc.start_at,
            Some(
                Local
                    .datetime_from_str("2022-10-01T12:30:15", "%Y-%m-%dT%H:%M:%S")
                    .unwrap()
            )
        );
    }

    #[test]
    fn test_deserialize_optional_local_datetime_with_null() {
        let raw = r#"{"start_at": null}"#;
        let doc: DocumentWithOptionalLocalDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert!(doc.start_at.is_none());
    }

    #[test]
    fn test_deserialize_optional_local_datetime_without_field() {
        let raw = r#"{}"#;
        let doc: DocumentWithOptionalLocalDateTimeOffset = serde_json::from_str(raw).unwrap();
        assert!(doc.start_at.is_none());
    }

    // ==============================================================================
}
