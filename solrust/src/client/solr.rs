//! This module defines the SolrClient struct.
//!
//! SolrClient struct is responsible for connecting to a running Solr instance
//! and creating a SolrCore struct, which represents a single Solr core.

use crate::client::core::SolrCore;
use crate::types::response::*;
use reqwest::Client;
use thiserror::Error;
use url::Url;

type Result<T> = std::result::Result<T, SolrClientError>;

#[derive(Debug, Error)]
pub enum SolrClientError {
    #[error("Failed to request to solr")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to parse given URL")]
    UrlParseError(#[from] url::ParseError),
    #[error("Given URL host is invalid")]
    InvalidHostError,
    #[error("Specified core name does not exist")]
    SpecifiedCoreNotFoundError,
    #[error("Failed to deserialize JSON data")]
    DeserializeError(#[from] serde_json::Error),
    #[error("Unexpected error")]
    UnexpectedError((u32, String)),
}

#[derive(Debug)]
pub struct SolrClient {
    /// Host URL of the Solr instance. e.g.) http://localhost:8983
    url: String,
    /// reqwest HTTP client
    client: Client,
}

impl SolrClient {
    /// Of the URL given as argument, only the schema and hostname are extracted and used.
    /// For example, if http://localhost:8983/solr is given, all port numbers and paths are ignored.
    pub fn new(url: &str, port: u32) -> Result<Self> {
        let url = Url::parse(url).map_err(|e| SolrClientError::UrlParseError(e))?;

        let scheme = url.scheme();
        let host = url
            .host_str()
            .ok_or_else(|| SolrClientError::InvalidHostError)?;

        Ok(SolrClient {
            url: format!("{}://{}:{}", scheme, host, port),
            client: reqwest::Client::new(),
        })
    }

    /// Methods to get the status of a Solr instance
    pub async fn status(&self) -> Result<SolrSystemInfo> {
        let path = "solr/admin/info/system";

        let response = self
            .client
            .get(format!("{}/{}", self.url, path))
            .send()
            .await
            .map_err(|e| SolrClientError::RequestError(e))?
            .text()
            .await
            .map_err(|e| SolrClientError::RequestError(e))?;

        let response: SolrSystemInfo =
            serde_json::from_str(&response).map_err(|e| SolrClientError::DeserializeError(e))?;

        if let Some(error) = response.error {
            return Err(SolrClientError::UnexpectedError((error.code, error.msg)));
        } else {
            Ok(response)
        }
    }

    ///  Method to get a list of cores present in the Solr instance
    pub async fn cores(&self) -> Result<SolrCoreList> {
        let path = "solr/admin/cores";

        let response = self
            .client
            .get(format!("{}/{}", self.url, path))
            .send()
            .await
            .map_err(|e| SolrClientError::RequestError(e))?
            .text()
            .await
            .map_err(|e| SolrClientError::RequestError(e))?;

        let response: SolrCoreList =
            serde_json::from_str(&response).map_err(|e| SolrClientError::DeserializeError(e))?;

        if let Some(error) = response.error {
            return Err(SolrClientError::UnexpectedError((error.code, error.msg)));
        } else {
            Ok(response)
        }
    }

    /// Method to create SolrCore struct
    pub async fn core(&self, name: &str) -> Result<SolrCore> {
        let cores = self
            .cores()
            .await?
            .status
            .ok_or_else(|| SolrClientError::SpecifiedCoreNotFoundError)?;

        if !cores.contains_key(name) {
            return Err(SolrClientError::SpecifiedCoreNotFoundError);
        }

        Ok(SolrCore::new(name, &self.url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Normal system test of SolrClient creation
    #[test]
    fn test_create_solr_client() {
        let client = SolrClient::new("http://localhost", 8983).unwrap();
        assert_eq!(client.url, "http://localhost:8983");
    }

    /// Normal system test of SolrClient creation.
    ///
    /// Check the behavior when given a redundant URL.
    /// Only the schema and host of the given URL are extracted. So even if a URL with additional
    /// information beyond the schema and host is provided, it will be ignored.
    #[test]
    fn test_create_solr_client_with_redundant_url() {
        let client = SolrClient::new("http://localhost:8983/solr", 8983).unwrap();
        assert_eq!(client.url, "http://localhost:8983");
    }

    /// Anomaly system test of SolrClient creation.
    /// Creation fails if an invalid URL is given.
    #[test]
    fn test_create_solr_client_with_invalid_url() {
        let client = SolrClient::new("hogehoge", 3000);
        assert!(client.is_err());
    }

    /// Normal system test of SolrClient status acquisition
    ///
    /// Run this test with the Docker container started with the following command.
    ///
    /// ```ignore
    /// docker run --rm -d -p 8983:8983 solr:9.1.0 solr-precreate example
    /// ```
    #[tokio::test]
    #[ignore]
    async fn test_get_status() {
        let client = SolrClient::new("http://localhost", 8983).unwrap();

        let response = client.status().await.unwrap();
        assert_eq!(response.header.status, 0);
    }

    /// Normal system test of core list acquisition
    ///
    /// Run this test with the Docker container started with the following command.
    ///
    /// ```ignore
    /// docker run --rm -d -p 8983:8983 solr:9.1.0 solr-precreate example
    /// ```
    #[tokio::test]
    #[ignore]
    async fn test_get_cores() {
        let client = SolrClient::new("http://localhost", 8983).unwrap();

        let response = client.cores().await.unwrap();
        assert!(response.status.unwrap().contains_key("example"));
    }

    /// Normal system test of the function to get the core list as a vector of String.
    ///
    /// Run this test with the Docker container started with the following command.
    ///
    /// ```ignore
    /// docker run --rm -d -p 8983:8983 solr:9.1.0 solr-precreate example
    /// ```
    #[tokio::test]
    #[ignore]
    async fn test_get_cores_as_vec() {
        let client = SolrClient::new("http://localhost", 8983).unwrap();

        let response = client.cores().await.unwrap();
        let cores = response.as_vec().unwrap();
        assert_eq!(cores, vec![String::from("example")]);
    }

    /// Normal system test of the function to create SolrCore object
    ///
    /// Run this test with the Docker container started with the following command.
    ///
    /// ```ignore
    /// docker run --rm -d -p 8983:8983 solr:9.1.0 solr-precreate example
    /// ```
    #[tokio::test]
    #[ignore]
    async fn test_get_core() {
        let client = SolrClient::new("http://localhost", 8983).unwrap();

        let core = client.core("example").await.unwrap();
        assert_eq!(core.name, String::from("example"));
    }

    /// Anomaly system test when a nonexistent core name is specified.
    /// SolrClient::core() method will return error.
    ///
    /// Run this test with the Docker container started with the following command.
    ///
    /// ```ignore
    /// docker run --rm -d -p 8983:8983 solr:9.1.0 solr-precreate example
    /// ```
    #[tokio::test]
    #[ignore]
    async fn test_get_non_existent_core() {
        let client = SolrClient::new("http://localhost", 8983).unwrap();

        let core = client.core("hoge").await;
        assert!(core.is_err());
    }
}
