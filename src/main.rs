use std::fs;
use std::io::{self, Write};

use anyhow::{Result, bail};
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

fn main() -> Result<()> {
    let args = Args::parse();
    let raw = fs::read_to_string("fixtures/search.json")?;
    let mut hits: Vec<Hit> = serde_json::from_str(&raw)?;

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

    println!("→ 선택: {}", hit.title);
    println!("  링크: {}", hit.link().unwrap());
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
