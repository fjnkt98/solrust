//! This module defines the SolrCore struct.
//!
//! The SolrCore struct is an abstraction of operations on the Solr core.
//!
//! Operations such as obtaining core status, posting and searching documents,
//! and reload core can be performed through this struct.

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

    /// Method to get core status.
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

        // Once the core object has been created,
        // 1. the `status` field must be present in the response JSON
        // 2. the key of the `status` field must contain this core
        //
        // is guaranteed, so `unwrap()` is used.
        let status = core_list.status.unwrap().get(&self.name).unwrap().clone();

        Ok(status)
    }

    /// Method to request the core to reload.
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

    /// Method to send request the core to search the document with some query parameters.
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

    /// TODO: Method to request the core to analyze given word.
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

    /// Method to post the document to the core.
    /// The document to be posted must be a JSON string.
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

    /// Method to send request the core to commit the post.
    ///
    /// When optimize is true, this method request to commit with optimization.
    pub async fn commit(&self, optimize: bool) -> Result<()> {
        if optimize {
            self.post(br#"{"optimize": {}}"#.to_vec()).await?;
        } else {
            self.post(br#"{"commit": {}}"#.to_vec()).await?;
        }

        Ok(())
    }

    /// Method to send request the core to rollback the post.
    pub async fn rollback(&self) -> Result<()> {
        self.post(br#"{"rollback": {}}"#.to_vec()).await?;

        Ok(())
    }

    /// Method to send a request to the core to delete all existing documents.
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

    /// Normal system test to get core status.
    ///
    /// Run this test with the Docker container started with the following command.
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

    /// Normal system test of reload of the core.
    ///
    /// The reload is considered successful if the time elapsed between the start of the reload
    /// and the start of the reloaded core is less than or equal to 1 second.
    ///
    /// Run this test with the Docker container started with the following command.
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

    /// Normal system test of the function to search documents.
    ///
    /// Run this test with the Docker container started with the following command.
    ///
    /// ```ignore
    /// docker run --rm -d -p 8983:8983 solr:9.1.0 solr-precreate example
    /// ```
    #[tokio::test]
    #[ignore]
    async fn test_select_in_normal() {
        let core = SolrCore::new("example", "http://localhost:8983");

        let params = vec![("q".to_string(), "*:*".to_string())];
        let response = core.select::<Document>(&params).await.unwrap();

        assert_eq!(response.header.status, 0);
    }

    /// Anomaly system test of the function to search documents.
    ///
    /// If nonexistent field was specified, select() method will return error.
    #[tokio::test]
    #[ignore]
    async fn test_select_in_non_normal() {
        let core = SolrCore::new("example", "http://localhost:8983");

        let params = vec![("q".to_string(), "text_hoge:*".to_string())];
        let response = core.select::<Document>(&params).await;

        assert!(response.is_err());
    }

    /// Normal system test of the function to analyze the word.
    ///
    /// Run this test with the Docker container started with the following command.
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

    /// Test scenario to test the behavior of a series of process: post documents to core, reload core, search for document, delete documents.
    ///
    /// Run this test with the Docker container started with the following command.
    ///
    /// ```ignore
    /// docker run --rm -d -p 8983:8983 solr:9.1.0 solr-precreate example
    /// ```
    #[tokio::test]
    #[ignore]
    async fn test_scenario() {
        let core = SolrCore::new("example", "http://localhost:8983");

        // Define schema for test with Schema API
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

        // Documents for test
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

        // Reload core (Only for operation check)
        core.reload().await.unwrap();

        // Post the documents to core.
        core.post(documents).await.unwrap();
        core.commit(true).await.unwrap();
        let status = core.status().await.unwrap();

        // Verify that 3 documents are registered.
        assert_eq!(status.index.num_docs, 3);

        // Test to search document
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

        // Delete all documents.
        core.truncate().await.unwrap();
        core.commit(true).await.unwrap();
        let status = core.status().await.unwrap();
        // Verify that no documents in index.
        assert_eq!(status.index.num_docs, 0);
    }
}
