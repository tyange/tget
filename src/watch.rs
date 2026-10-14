use std::io::{self, Write};
use std::time::Duration;

use anyhow::Result;

use crate::qbit::{Qbit, TorrentInfo};
use crate::ui::human_size;

const BAR_WIDTH: usize = 20;
const LINE_WIDTH: usize = 60;

/// 태그가 붙은 토렌트가 100% 가 될 때까지 2초마다 조회해 한 줄로 보여 준다.
pub async fn watch(qbit: &Qbit, tag: &str) -> Result<()> {
    println!("진행률을 지켜보는 중… (Ctrl+C로 그만 보기, 다운로드는 계속돼요)");
    loop {
        // 메타데이터를 받기 전에는 목록에 아직 없을 수 있다. 에러가 아니라 대기다.
        let line = match qbit.info_by_tag(tag).await?.first() {
            Some(t) if t.progress >= 1.0 => {
                println!("\r{:<LINE_WIDTH$}", format_line(t));
                println!("완료: {}", t.name);
                return Ok(());
            }
            Some(t) => format_line(t),
            None => "메타데이터를 기다리는 중…".to_string(),
        };
        // 이전 줄이 더 길었을 때 찌꺼기가 남지 않도록 같은 폭까지 공백으로 채운다.
        print!("\r{line:<LINE_WIDTH$}");
        io::stdout().flush()?;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

fn format_line(t: &TorrentInfo) -> String {
    let p = t.progress.clamp(0.0, 1.0);
    let filled = ((p * BAR_WIDTH as f64).round() as usize).min(BAR_WIDTH);
    format!(
        "[{}{}] {:>5.1}%  {}/s  {}",
        "#".repeat(filled),
        "-".repeat(BAR_WIDTH - filled),
        p * 100.0,
        human_size(t.dlspeed),
        t.state
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn torrent(progress: f64) -> TorrentInfo {
        TorrentInfo {
            name: "x".into(),
            progress,
            dlspeed: 3 * 1024 * 1024,
            state: "downloading".into(),
        }
    }

    #[test]
    fn half_progress_line() {
        assert_eq!(
            format_line(&torrent(0.5)),
            "[##########----------]  50.0%  3.0MB/s  downloading"
        );
    }

    #[test]
    fn bar_never_exceeds_width() {
        assert!(format_line(&torrent(1.0000001)).starts_with("[####################]"));
        assert!(format_line(&torrent(0.0)).starts_with("[--------------------]"));
    }
}
