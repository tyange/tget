use std::fs;

use anyhow::Result;
use clap::Parser;
use serde::Deserialize;

/// 파일명으로 토렌트를 검색하고, 있으면 qBittorrent로 다운로드를 시작한다.
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// 검색할 파일명
    query: String,

    /// 보여줄 결과 수
    #[arg(short, long, default_value_t = 10)]
    limit: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Hit {
    title: String,
    #[serde(default)]
    size: u64,
    seeders: Option<u32>,
    #[serde(default)]
    indexer: String,
    magnet_url: Option<String>,
    download_url: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let raw = fs::read_to_string("fixtures/search.json")?;
    let hits: Vec<Hit> = serde_json::from_str(&raw)?;

    println!("'{}' 검색 결과 {}개", args.query, hits.len());
    for hit in &hits {
        println!("{hit:#?}");
    }
    Ok(())
}
