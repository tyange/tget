use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
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

    /// 최소 시더 수
    #[arg(long, default_value_t = 1)]
    min_seeders: u32,

    /// 선택 없이 시더가 가장 많은 결과를 바로 받는다
    #[arg(short, long)]
    first: bool,

    /// 결과만 보여주고 추가하지 않는다
    #[arg(long)]
    dry_run: bool,

    /// Prowlarr 대신 로컬 JSON 파일에서 검색 결과를 읽는다 (개발용)
    #[arg(long)]
    fixture: Option<PathBuf>,

    #[arg(long, env = "PROWLARR_URL", default_value = "http://localhost:9696")]
    prowlarr_url: String,

    #[arg(long, env = "PROWLARR_API_KEY", hide_env_values = true)]
    prowlarr_api_key: Option<String>,

    /// 저장 경로 (qBittorrent 기준)
    #[arg(long)]
    save_path: Option<String>,

    #[arg(long, env = "QBIT_URL", default_value = "http://localhost:8080")]
    qbit_url: String,

    #[arg(long, env = "QBIT_USER", default_value = "admin")]
    qbit_user: String,

    #[arg(long, env = "QBIT_PASS", hide_env_values = true, default_value = "")]
    qbit_pass: String,
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

impl Hit {
    fn seeders(&self) -> u32 {
        self.seeders.unwrap_or(0)
    }

    fn link(&self) -> Option<&str> {
        self.magnet_url
            .as_deref()
            .or(self.download_url.as_deref())
            .filter(|s| !s.is_empty())
    }
}

async fn search(http: &reqwest::Client, args: &Args) -> Result<Vec<Hit>> {
    if let Some(path) = &args.fixture {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("{} 을 읽지 못했어요", path.display()))?;
        return Ok(serde_json::from_str(&raw)?);
    }

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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let http = reqwest::Client::builder().cookie_store(true).build()?;

    let mut hits = search(&http, &args).await?;

    hits.retain(|h| h.seeders() >= args.min_seeders && h.link().is_some());
    hits.sort_by(|a, b| b.seeders().cmp(&a.seeders()));
    hits.truncate(args.limit);

    if hits.is_empty() {
        bail!("'{}' 에 해당하는 토렌트를 찾지 못했어요", args.query);
    }

    print_hits(&hits);

    if args.dry_run {
        return Ok(());
    }

    let hit = if args.first {
        &hits[0]
    } else {
        match prompt_choice(hits.len())? {
            Some(i) => &hits[i],
            None => {
                println!("취소했어요");
                return Ok(());
            }
        }
    };

    add_to_qbit(&http, &args, hit.link().unwrap()).await?;
    println!("→ qBittorrent에 추가됨: {}", hit.title);
    Ok(())
}

async fn add_to_qbit(http: &reqwest::Client, args: &Args, link: &str) -> Result<()> {
    let base = args.qbit_url.trim_end_matches('/');

    let body = http
        .post(format!("{base}/api/v2/auth/login"))
        .header("Referer", base)
        .form(&[("username", &args.qbit_user), ("password", &args.qbit_pass)])
        .send()
        .await
        .context("qBittorrent에 연결하지 못했어요")?
        .text()
        .await?;
    if body.trim() != "Ok." {
        bail!("qBittorrent 로그인에 실패했어요. 사용자 이름과 비밀번호를 확인하세요");
    }

    let mut form = vec![("urls", link.to_string())];
    if let Some(p) = &args.save_path {
        form.push(("savepath", p.clone()));
    }
    let res = http
        .post(format!("{base}/api/v2/torrents/add"))
        .header("Referer", base)
        .form(&form)
        .send()
        .await
        .context("qBittorrent에 토렌트를 추가하지 못했어요")?;
    let status = res.status();
    let body = res.text().await?;
    if !status.is_success() || body.contains("Fails") {
        bail!("qBittorrent가 추가를 거절했어요 ({status}): {}", body.trim());
    }

    Ok(())
}

fn prompt_choice(max: usize) -> Result<Option<usize>> {
    loop {
        print!("선택 (1-{max}, 엔터=취소)> ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let line = line.trim();

        if line.is_empty() || line.eq_ignore_ascii_case("q") {
            return Ok(None);
        }
        match line.parse::<usize>() {
            Ok(n) if (1..=max).contains(&n) => return Ok(Some(n - 1)),
            _ => println!("1-{max} 사이 숫자를 입력하세요"),
        }
    }
}

fn print_hits(hits: &[Hit]) {
    for (i, h) in hits.iter().enumerate() {
        println!(
            "{:>2}) {}\n     {}  seeders {}  [{}]",
            i + 1,
            h.title,
            human_size(h.size),
            h.seeders(),
            h.indexer
        );
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    format!("{v:.1}{}", UNITS[u])
}
