use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::cli::Args;

/// 설정 파일(config.toml)의 모양. 전부 생략할 수 있다.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileConfig {
    pub prowlarr_url: Option<String>,
    pub prowlarr_api_key: Option<String>,
    pub qbit_url: Option<String>,
    pub qbit_user: Option<String>,
    pub qbit_pass: Option<String>,
    pub save_path: Option<String>,
}

/// 명령줄 > 환경변수 > 설정 파일 > 기본값 순으로 확정한 최종 설정.
#[derive(Debug)]
pub struct Settings {
    pub prowlarr_url: String,
    pub prowlarr_api_key: Option<String>,
    pub qbit_url: String,
    pub qbit_user: String,
    pub qbit_pass: String,
    pub save_path: Option<String>,
}

impl Settings {
    pub fn load(args: &Args) -> Result<Settings> {
        let file = read_file(args.config.as_deref())?;
        Ok(merge(args, file))
    }
}

fn default_path() -> Option<PathBuf> {
    let base = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => dirs::home_dir()?.join(".config"),
    };
    Some(base.join("tget").join("config.toml"))
}

/// `--config` 로 지정한 파일은 반드시 있어야 하고, 기본 위치의 파일은 없어도 된다.
fn read_file(explicit: Option<&Path>) -> Result<FileConfig> {
    let (path, required) = match explicit {
        Some(path) => (path.to_path_buf(), true),
        None => match default_path() {
            Some(path) => (path, false),
            None => return Ok(FileConfig::default()),
        },
    };

    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && !required => {
            return Ok(FileConfig::default());
        }
        Err(e) => {
            return Err(e)
                .with_context(|| format!("설정 파일 {} 을 읽지 못했어요", path.display()));
        }
    };
    toml::from_str(&raw)
        .with_context(|| format!("설정 파일 {} 의 형식이 올바르지 않아요", path.display()))
}

fn merge(args: &Args, file: FileConfig) -> Settings {
    Settings {
        prowlarr_url: args
            .prowlarr_url
            .clone()
            .or(file.prowlarr_url)
            .unwrap_or_else(|| "http://localhost:9696".into()),
        prowlarr_api_key: args.prowlarr_api_key.clone().or(file.prowlarr_api_key),
        qbit_url: args
            .qbit_url
            .clone()
            .or(file.qbit_url)
            .unwrap_or_else(|| "http://localhost:8080".into()),
        qbit_user: args
            .qbit_user
            .clone()
            .or(file.qbit_user)
            .unwrap_or_else(|| "admin".into()),
        qbit_pass: args
            .qbit_pass
            .clone()
            .or(file.qbit_pass)
            .unwrap_or_default(),
        save_path: args.save_path.clone().or(file.save_path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // 테스트를 돌리는 셸에 QBIT_URL 같은 환경변수가 있으면 결과가 달라질 수 있다.
    fn args(extra: &[&str]) -> Args {
        let mut v = vec!["tget", "x"];
        v.extend_from_slice(extra);
        Args::parse_from(v)
    }

    fn file_with_url(url: &str) -> FileConfig {
        FileConfig {
            qbit_url: Some(url.into()),
            ..Default::default()
        }
    }

    #[test]
    fn cli_beats_file() {
        let s = merge(
            &args(&["--qbit-url", "http://cli:1"]),
            file_with_url("http://file:2"),
        );
        assert_eq!(s.qbit_url, "http://cli:1");
    }

    #[test]
    fn file_used_when_cli_missing() {
        let s = merge(&args(&[]), file_with_url("http://file:2"));
        assert_eq!(s.qbit_url, "http://file:2");
    }

    #[test]
    fn defaults_when_nothing_given() {
        let s = merge(&args(&[]), FileConfig::default());
        assert_eq!(s.qbit_url, "http://localhost:8080");
        assert_eq!(s.qbit_user, "admin");
        assert_eq!(s.prowlarr_url, "http://localhost:9696");
        assert_eq!(s.prowlarr_api_key, None);
    }

    #[test]
    fn parses_toml_and_rejects_unknown_keys() {
        let ok: FileConfig = toml::from_str("qbit_user = \"me\"\nsave_path = \"/dl\"").unwrap();
        assert_eq!(ok.qbit_user.as_deref(), Some("me"));
        assert!(toml::from_str::<FileConfig>("qbti_url = \"x\"").is_err());
    }
}
