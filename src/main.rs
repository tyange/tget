mod cli;
mod config;
mod hit;
mod prowlarr;
mod qbit;
mod ui;
mod watch;

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use clap::Parser;

use cli::Args;
use config::Settings;
use qbit::Qbit;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let settings = Settings::load(&args)?;
    let http = reqwest::Client::builder().cookie_store(true).build()?;

    let hits = match &args.fixture {
        Some(path) => prowlarr::load_fixture(path)?,
        None => prowlarr::search(&http, &settings, &args.query).await?,
    };

    let hits = hit::rank(hits, args.min_seeders, args.limit);

    if hits.is_empty() {
        bail!("'{}' 에 해당하는 토렌트를 찾지 못했어요", args.query);
    }

    ui::print_hits(&hits);

    if args.dry_run {
        return Ok(());
    }

    let hit = if args.first {
        &hits[0]
    } else {
        match ui::prompt_choice(hits.len())? {
            Some(i) => &hits[i],
            None => {
                println!("취소했어요");
                return Ok(());
            }
        }
    };

    let qbit = Qbit::login(
        http,
        &settings.qbit_url,
        &settings.qbit_user,
        &settings.qbit_pass,
    )
    .await?;
    // add 는 해시를 돌려주지 않으므로, 우리만 아는 태그를 붙여 두고 그 태그로 조회한다.
    let tag = args.watch.then(new_tag);
    qbit.add(
        hit.link().unwrap(),
        settings.save_path.as_deref(),
        tag.as_deref(),
    )
    .await?;
    println!("→ qBittorrent에 추가됨: {}", hit.title);

    if let Some(tag) = tag {
        watch::watch(&qbit, &tag).await?;
    }
    Ok(())
}

fn new_tag() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("tget-{nanos}")
}
