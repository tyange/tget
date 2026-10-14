use anyhow::{Context, Result, bail};

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

    pub async fn add(&self, link: &str, save_path: Option<&str>) -> Result<()> {
        let mut form = vec![("urls", link)];
        if let Some(p) = save_path {
            form.push(("savepath", p));
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
}
