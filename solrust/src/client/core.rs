use crate::types::response::*;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

type Result<T> = std::result::Result<T, SolrCoreError>;

#[derive(Debug, Error)]
pub enum SolrCoreError {
    #[error("Failed to request to solr core")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to deserialize JSON data")]
    DeserializeError(#[from] serde_json::Error),
    #[error("Unexpected error")]
    UnexpectedError((u32, String)),
}

#[derive(Clone)]
pub struct SolrCore {
    pub name: String,
    pub base_url: String,
    pub core_url: String,
    client: Client,
}

impl SolrCore {
    pub fn new(name: &str, base_url: &str) -> Self {
        let core_url = format!("{}/solr/{}", base_url, name);

        SolrCore {
            name: String::from(name),
            base_url: String::from(base_url),
            core_url: core_url,
            client: reqwest::Client::new(),
        }
    }
    pub async fn status(&self) -> Result<SolrCoreStatus> {
        let response = self
            .client
            .get(format!("{}/solr/admin/cores", self.base_url))
            .query(&[("action", "status"), ("core", &self.name)])
            .send()
            .await
            .map_err(|e| SolrCoreError::RequestError(e))?;

        let content = response
            .text()
            .await
            .map_err(|e| SolrCoreError::RequestError(e))?;

        let core_list: SolrCoreList =
            serde_json::from_str(&content).map_err(|e| SolrCoreError::DeserializeError(e))?;

        if let Some(error) = core_list.error {
            return Err(SolrCoreError::UnexpectedError((error.code, error.msg)));
        }

        // コアオブジェクトが作成できた時点で
        //
        // 1. レスポンスのJSONに`status`フィールドが存在すること
        // 2. `status`フィールドのキーにこのコアが含まれていること
        //
        // が保証されているので、`unwrap()`を使用している。
        let status = core_list.status.unwrap().get(&self.name).unwrap().clone();

        Ok(status)
    }

    pub async fn reload(&self) -> Result<u32> {
        let response = self
            .client
            .get(format!("{}/solr/admin/cores", self.base_url))
            .query(&[("action", "reload"), ("core", &self.name)])
            .send()
            .await
            .map_err(|e| SolrCoreError::RequestError(e))?;

        let content = response
            .text()
            .await
            .map_err(|e| SolrCoreError::RequestError(e))?;

        let response: SolrSimpleResponse =
            serde_json::from_str(&content).map_err(|e| SolrCoreError::DeserializeError(e))?;

        if let Some(error) = response.error {
            return Err(SolrCoreError::UnexpectedError((error.code, error.msg)));
        }

        Ok(response.header.status)
    }

    pub async fn select<D>(
        &self,
        params: &Vec<(impl Serialize, impl Serialize)>,
    ) -> Result<SolrSelectResponse<D>>
    where
        D: Serialize + DeserializeOwned,
    {
        let response = self
            .client
            .get(format!("{}/select", self.core_url))
            .query(params)
            .send()
            .await
            .map_err(|e| SolrCoreError::RequestError(e))?;

        let content = response
            .text()
            .await
            .map_err(|e| SolrCoreError::RequestError(e))?;

        let selection: SolrSelectResponse<D> =
            serde_json::from_str(&content).map_err(|e| SolrCoreError::DeserializeError(e))?;

        if let Some(error) = selection.error {
            return Err(SolrCoreError::UnexpectedError((error.code, error.msg)));
        }

        Ok(selection)
    }

    // pub async fn analyze(&self, word: &str, field: &str, analyzer: &str) -> Result<Vec<String>> {
    //     todo!();
    // let params = [("analysis.fieldvalue", word), ("analysis.fieldtype", field)];

    // let response = self
    //     .client
    //     .get(format!("{}/analysis/field", self.core_url))
    //     .query(&params)
    //     .send()
    //     .await
    //     .map_err(|e| SolrCoreError::RequestError(e))?
    //     .text()
    //     .await
    //     .map_err(|e| SolrCoreError::RequestError(e))?;

    // let result: SolrAnalysisResponse =
    //     serde_json::from_str(&response).map_err(|e| SolrCoreError::DeserializeError(e))?;

    // let result = result.analysis.field_types.get(field).unwrap();
    // let result = match analyzer {
    //     "index" => result.index.as_ref().unwrap(),
    //     "query" => result.query.as_ref().unwrap(),
    //     _ => return Err(SolrCoreError::InvalidValueError),
    // };
    // let result = result.last().unwrap().clone();

    // let result = match result {
    //     Value::Array(array) => array
    //         .iter()
    //         .map(|e| e["text"].to_string().trim_matches('"').to_string())
    //         .collect::<Vec<String>>(),
    //     _ => Vec::new(),
    // };

    // Ok(result)
    // }

    pub async fn post(&self, body: Vec<u8>) -> Result<SolrSimpleResponse> {
        let response = self
            .client
            .post(format!("{}/update", self.core_url))
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| SolrCoreError::RequestError(e))?;

        let content = response
            .text()
            .await
            .map_err(|e| SolrCoreError::RequestError(e))?;

        let post_result: SolrSimpleResponse =
            serde_json::from_str(&content).map_err(|e| SolrCoreError::DeserializeError(e))?;

        Ok(post_result)
    }

    pub async fn commit(&self, optimize: bool) -> Result<()> {
        if optimize {
            self.post(br#"{"optimize": {}}"#.to_vec()).await?;
        } else {
            self.post(br#"{"commit": {}}"#.to_vec()).await?;
        }

        Ok(())
    }

    pub async fn rollback(&self) -> Result<()> {
        self.post(br#"{"rollback": {}}"#.to_vec()).await?;

        Ok(())
    }

    pub async fn truncate(&self) -> Result<()> {
        self.post(br#"{"delete":{"query": "*:*"}}"#.to_vec())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{DateTime, Utc};
    use serde::Deserialize;
    use serde_json::{self, Value};

    /// コアのステータス取得メソッドの正常系テスト
    ///
    /// 以下のコマンドでDockerコンテナを起動してからテストを実行すること。
    ///
    /// ```ignore
    /// docker run --rm -d -p 8983:8983 solr:9.1.0 solr-precreate example
    /// ```
    #[tokio::test]
    #[ignore]
    async fn test_get_status() {
        let core = SolrCore::new("example", "http://localhost:8983");
        let status = core.status().await.unwrap();

        assert_eq!(status.name, String::from("example"));
    }

    /// コアのリロードメソッドの正常系テスト
    ///
    /// コアのリロード実行時の時刻と、リロード後のコアのスタートタイムの差が1秒以内なら
    /// リロードが実行されたと判断する。
    ///
    /// 以下のコマンドでDockerコンテナを起動してからテストを実行すること。
    ///
    /// ```ignore
    /// docker run --rm -d -p 8983:8983 solr:9.1.0 solr-precreate example
    /// ```
    #[tokio::test]
    #[ignore]
    async fn test_reload() {
        let core = SolrCore::new("example", "http://localhost:8983");

        let before = Utc::now();

        core.reload().await.unwrap();

        let status = core.status().await.unwrap();
        let after = status.start_time.replace("Z", "+00:00");
        let after = DateTime::parse_from_rfc3339(&after)
            .unwrap()
            .with_timezone(&Utc);

        assert!(before < after);

        let duration = (after - before).num_milliseconds();
        assert!(duration.abs() < 1000);
    }

    #[derive(Serialize, Deserialize)]
    struct Document {
        id: i64,
    }

    #[tokio::test]
    #[ignore]
    async fn test_select_in_normal() {
        let core = SolrCore::new("example", "http://localhost:8983");

        let params = vec![("q".to_string(), "*:*".to_string())];
        let response = core.select::<Document>(&params).await.unwrap();

        assert_eq!(response.header.status, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_select_in_non_normal() {
        let core = SolrCore::new("example", "http://localhost:8983");

        let params = vec![("q".to_string(), "text_hoge:*".to_string())];
        let response = core.select::<Document>(&params).await;

        assert!(response.is_err());
    }

    /// 単語の解析メソッドの正常系テスト
    ///
    /// とりあえずエラーが出ないことを確認する。
    ///
    /// 以下のコマンドでDockerコンテナを起動してからテストを実行すること。
    ///
    /// ```ignore
    /// docker run --rm -d -p 8983:8983 solr:9.1.0 solr-precreate example
    /// ```
    // #[tokio::test]
    // #[ignore]
    // async fn test_analyze() {
    //     let core = SolrCore::new("example", "http://localhost:8983");

    //     let word = "solr-client";
    //     let expected = vec![String::from("solr"), String::from("client")];

    //     let actual = core.analyze(word, "text_en", "index").await.unwrap();

    //     assert_eq!(expected, actual);
    // }

    /// コアへのドキュメントのポスト・コアのリロード・ドキュメントの検索・ドキュメントの削除の一連の処理の動作をテストするテストシナリオ
    ///
    /// 以下のコマンドでDockerコンテナを起動してからテストを実行すること。
    ///
    /// ```ignore
    /// docker run --rm -d -p 8983:8983 solr:9.1.0 solr-precreate example
    /// ```
    #[tokio::test]
    #[ignore]
    async fn test_scenario() {
        let core = SolrCore::new("example", "http://localhost:8983");

        // Schema APIを使ってテスト用のスキーマを定義
        let client = reqwest::Client::new();
        client
            .post(format!("{}/schema", core.core_url))
            .body(
                serde_json::json!(
                    {
                        "add-field": {
                            "name": "name",
                            "type": "string",
                            "indexed": true,
                            "stored": true,
                            "multiValued": false
                        },
                    }
                )
                .to_string(),
            )
            .send()
            .await
            .unwrap();
        client
            .post(format!("{}/schema", core.core_url))
            .body(
                serde_json::json!(
                    {
                        "add-field": {
                            "name": "gender",
                            "type": "string",
                            "indexed": true,
                            "stored": true,
                            "multiValued": false
                        }
                    }
                )
                .to_string(),
            )
            .send()
            .await
            .unwrap();

        // テスト用のドキュメント
        let documents = serde_json::json!(
            [
                {
                    "id": "001",
                    "name": "alice",
                    "gender": "female"
                },
                {
                    "id": "002",
                    "name": "bob",
                    "gender": "male"
                },
                {
                    "id": "003",
                    "name": "charles",
                    "gender": "male"
                }
            ]
        )
        .to_string()
        .as_bytes()
        .to_vec();

        // リロード(Schema APIを使っているので不要だけど動作テストなので)
        core.reload().await.unwrap();

        // ドキュメントをポスト
        core.post(documents).await.unwrap();
        // コミット
        core.commit(true).await.unwrap();
        let status = core.status().await.unwrap();

        // 3件のドキュメントが登録されていることを確認
        assert_eq!(status.index.num_docs, 3);

        // 検索のテスト
        let params = vec![
            ("q".to_string(), "name:alice".to_string()),
            ("fl".to_string(), "id,name,gender".to_string()),
        ];
        let result = core.select::<Value>(&params).await.unwrap();
        assert_eq!(result.response.num_found, 1);
        assert_eq!(
            result.response.docs,
            vec![serde_json::json!({"id": "001", "name": "alice", "gender": "female"})]
        );

        // ドキュメントをすべて削除
        core.truncate().await.unwrap();
        core.commit(true).await.unwrap();
        let status = core.status().await.unwrap();
        assert_eq!(status.index.num_docs, 0);
    }
}
