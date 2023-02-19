# solrust

Solr Client for Rust.

## Basic Usage

```rust
use serde_json::Value;
use solrust::client::solr::SolrClient;
use solrust::querybuilder::{
    common::SolrCommonQueryBuilder,
    q::QueryOperand,
    sort::SortOrderBuilder,
    standard::{SolrStandardQueryBuilder, StandardQueryBuilder},
};
use solrust::types::response::*;

#[tokio::main]
async fn main() {
    let solr = SolrClient::new("http://localhost", 8983).unwrap();
    let core = solr.core("example").await.unwrap();

    let q = QueryOperand("id:foo".to_string());
    let sort = SortOrderBuilder::new().desc("score").asc("id");
    let builder = StandardQueryBuilder::new().q(&q).sort(&sort);

    let response: SolrSelectResponse<Value> = core.select(&builder.build()).await.unwrap();

    println!("{:?}", response);
}
```

## Future Works

- Support for [Result Grouping](https://solr.apache.org/guide/solr/latest/query-guide/result-grouping.html).
- Support for Solr Cloud.