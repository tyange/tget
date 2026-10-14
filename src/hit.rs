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

/// 받을 수 없는 결과를 걸러내고 시더 많은 순으로 limit개만 남긴다.
pub fn rank(mut hits: Vec<Hit>, min_seeders: u32, limit: usize) -> Vec<Hit> {
    hits.retain(|h| h.seeders() >= min_seeders && h.link().is_some());
    hits.sort_by_key(|h| std::cmp::Reverse(h.seeders()));
    hits.truncate(limit);
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<Hit> {
        serde_json::from_str(include_str!("../fixtures/search.json")).unwrap()
    }

    fn hit(magnet: Option<&str>, download: Option<&str>) -> Hit {
        Hit {
            title: "a".into(),
            size: 0,
            seeders: Some(1),
            indexer: String::new(),
            magnet_url: magnet.map(Into::into),
            download_url: download.map(Into::into),
        }
    }

    #[test]
    fn parses_fixture() {
        let hits = fixture();
        assert_eq!(hits.len(), 5);
        assert_eq!(hits[3].seeders, None);
        assert_eq!(hits[4].link(), None);
    }

    #[test]
    fn magnet_is_preferred() {
        let h = hit(Some("magnet:?x"), Some("http://d"));
        assert_eq!(h.link(), Some("magnet:?x"));
        assert_eq!(hit(None, Some("http://d")).link(), Some("http://d"));
        assert_eq!(hit(Some(""), None).link(), None);
    }

    #[test]
    fn rank_filters_and_sorts() {
        let titles: Vec<String> = rank(fixture(), 1, 10)
            .into_iter()
            .map(|h| h.title)
            .collect();
        assert_eq!(
            titles,
            [
                "ubuntu-24.04.1-desktop-amd64.iso",
                "ubuntu-24.04-live-server-amd64.iso"
            ]
        );
    }

    #[test]
    fn rank_respects_limit() {
        assert_eq!(rank(fixture(), 0, 1).len(), 1);
    }
}
