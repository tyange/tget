use std::path::PathBuf;

use clap::Parser;

/// 파일명으로 토렌트를 검색하고, 있으면 qBittorrent로 다운로드를 시작한다.
#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    /// 검색할 파일명
    pub query: String,

    /// 보여줄 결과 수
    #[arg(short, long, default_value_t = 10)]
    pub limit: usize,

    /// 최소 시더 수
    #[arg(long, default_value_t = 1)]
    pub min_seeders: u32,

    /// 선택 없이 시더가 가장 많은 결과를 바로 받는다
    #[arg(short, long)]
    pub first: bool,

    /// 결과만 보여주고 추가하지 않는다
    #[arg(long)]
    pub dry_run: bool,

    /// Prowlarr 대신 로컬 JSON 파일에서 검색 결과를 읽는다 (개발용)
    #[arg(long)]
    pub fixture: Option<PathBuf>,

    /// 추가한 뒤 다운로드 진행률을 지켜본다 (Ctrl+C 로 그만 보기)
    #[arg(short, long)]
    pub watch: bool,

    /// 설정 파일 경로 (기본: $XDG_CONFIG_HOME/tget/config.toml 또는 ~/.config/tget/config.toml)
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Prowlarr 주소 (기본: http://localhost:9696)
    #[arg(long, env = "PROWLARR_URL")]
    pub prowlarr_url: Option<String>,

    #[arg(long, env = "PROWLARR_API_KEY", hide_env_values = true)]
    pub prowlarr_api_key: Option<String>,

    /// 저장 경로 (qBittorrent 기준)
    #[arg(long)]
    pub save_path: Option<String>,

    /// qBittorrent 주소 (기본: http://localhost:8080)
    #[arg(long, env = "QBIT_URL")]
    pub qbit_url: Option<String>,

    /// qBittorrent 사용자 이름 (기본: admin)
    #[arg(long, env = "QBIT_USER")]
    pub qbit_user: Option<String>,

    #[arg(long, env = "QBIT_PASS", hide_env_values = true)]
    pub qbit_pass: Option<String>,
}
