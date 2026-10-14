use anyhow::{Context, Result, bail};
use serde::Deserialize;

/// `/api/v2/torrents/info` 응답의 한 항목 (쓰는 칸만).
#[derive(Debug, Deserialize)]
pub struct TorrentInfo {
    pub name: String,
    /// 0.0 ~ 1.0
    pub progress: f64,
    /// 초당 바이트
    pub dlspeed: u64,
    pub state: String,
}

/// 로그인을 마친 qBittorrent 클라이언트. `login` 에 성공해야만 만들 수 있다.
pub struct Qbit {
    http: reqwest::Client,
    base: String,
}

impl Qbit {
    pub async fn login(http: reqwest::Client, url: &str, user: &str, pass: &str) -> Result<Self> {
        let base = url.trim_end_matches('/').to_string();

        let body = http
            .post(format!("{base}/api/v2/auth/login"))
            .header("Referer", &base)
            .form(&[("username", user), ("password", pass)])
            .send()
            .await
            .context("qBittorrent에 연결하지 못했어요")?
            .text()
            .await?;
        if body.trim() != "Ok." {
            bail!("qBittorrent 로그인에 실패했어요. 사용자 이름과 비밀번호를 확인하세요");
        }

        Ok(Self { http, base })
    }

    pub async fn add(&self, link: &str, save_path: Option<&str>, tag: Option<&str>) -> Result<()> {
        let mut form = vec![("urls", link)];
        if let Some(p) = save_path {
            form.push(("savepath", p));
        }
        if let Some(t) = tag {
            form.push(("tags", t));
        }
        let res = self
            .http
            .post(format!("{}/api/v2/torrents/add", self.base))
            .header("Referer", &self.base)
            .form(&form)
            .send()
            .await
            .context("qBittorrent에 토렌트를 추가하지 못했어요")?;
        let status = res.status();
        let body = res.text().await?;
        if !status.is_success() || body.contains("Fails") {
            bail!(
                "qBittorrent가 추가를 거절했어요 ({status}): {}",
                body.trim()
            );
        }

        Ok(())
    }

    /// 태그가 붙은 토렌트만 조회한다. 메타데이터를 받기 전에는 빈 배열일 수 있다.
    pub async fn info_by_tag(&self, tag: &str) -> Result<Vec<TorrentInfo>> {
        self.http
            .get(format!("{}/api/v2/torrents/info", self.base))
            .header("Referer", &self.base)
            .query(&[("tag", tag)])
            .send()
            .await
            .context("qBittorrent에서 진행률을 조회하지 못했어요")?
            .error_for_status()
            .context("qBittorrent가 진행률 조회를 거절했어요")?
            .json()
            .await
            .context("qBittorrent 응답을 해석하지 못했어요")
    }
}
