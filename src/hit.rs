use serde::Deserialize;

/// Prowlarr 검색 결과 한 건.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    pub title: String,
    #[serde(default)]
    pub size: u64,
    pub seeders: Option<u32>,
    #[serde(default)]
    pub indexer: String,
    pub magnet_url: Option<String>,
    pub download_url: Option<String>,
}

impl Hit {
    pub fn seeders(&self) -> u32 {
        self.seeders.unwrap_or(0)
    }

    pub fn link(&self) -> Option<&str> {
        self.magnet_url
            .as_deref()
            .or(self.download_url.as_deref())
            .filter(|s| !s.is_empty())
    }
}
