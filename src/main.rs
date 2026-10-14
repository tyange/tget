mod cli;
mod hit;
mod prowlarr;
mod qbit;
mod ui;

use anyhow::{Result, bail};
use clap::Parser;

use cli::Args;
use qbit::Qbit;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let http = reqwest::Client::builder().cookie_store(true).build()?;

    let hits = match &args.fixture {
        Some(path) => prowlarr::load_fixture(path)?,
        None => prowlarr::search(&http, &args).await?,
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

    let qbit = Qbit::login(http, &args.qbit_url, &args.qbit_user, &args.qbit_pass).await?;
    qbit.add(hit.link().unwrap(), args.save_path.as_deref())
        .await?;
    println!("→ qBittorrent에 추가됨: {}", hit.title);
    Ok(())
}
