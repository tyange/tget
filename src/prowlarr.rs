use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::cli::Args;
use crate::hit::Hit;

/// 로컬 JSON 파일에서 검색 결과를 읽는다 (개발·테스트용).
pub fn load_fixture(path: &Path) -> Result<Vec<Hit>> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("{} 을 읽지 못했어요", path.display()))?;
    Ok(serde_json::from_str(&raw)?)
}

/// Prowlarr API 로 검색한다.
pub async fn search(http: &reqwest::Client, args: &Args) -> Result<Vec<Hit>> {
    let api_key = args
        .prowlarr_api_key
        .as_deref()
        .context("PROWLARR_API_KEY 환경변수나 --prowlarr-api-key 가 필요해요")?;
    let url = format!("{}/api/v1/search", args.prowlarr_url.trim_end_matches('/'));

    http.get(url)
        .header("X-Api-Key", api_key)
        .query(&[("query", args.query.as_str()), ("type", "search")])
        .send()
        .await
        .context("Prowlarr에 연결하지 못했어요")?
        .error_for_status()
        .context("Prowlarr가 요청을 거절했어요. API 키를 확인하세요")?
        .json()
        .await
        .context("Prowlarr 응답을 해석하지 못했어요")
}
