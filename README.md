# tget

파일명으로 토렌트를 검색하고, 고르면 qBittorrent 로 다운로드를 시작하는 Rust CLI.
[Prowlarr](https://prowlarr.com) 로 검색하고 qBittorrent Web API 로 넘기는 얇은 접착제입니다.

> 리눅스 ISO, 공개 데이터셋, 퍼블릭 도메인 영상처럼 **합법적으로 배포되는 파일**을 받는 용도입니다.

## 설치

```bash
cargo install --path .     # ~/.cargo/bin/tget
```

## 준비물

- Prowlarr (기본 `http://localhost:9696`) 와 인덱서, API 키
- qBittorrent Web UI (기본 `http://localhost:8080`) 의 사용자 이름·비밀번호

## 설정

기본 위치 `$XDG_CONFIG_HOME/tget/config.toml` (없으면 `~/.config/tget/config.toml`).
파일이 없어도 되고, `--config <경로>` 로 지정한 파일은 반드시 있어야 합니다. 모든 항목은 생략할 수 있습니다.

```toml
prowlarr_url = "http://192.168.0.10:9696"
prowlarr_api_key = "abcdef..."
qbit_url = "http://192.168.0.10:8080"
qbit_user = "admin"
qbit_pass = "secret"
save_path = "/downloads"
```

비밀번호가 들어 있으니 `chmod 600 ~/.config/tget/config.toml` 을 권장합니다.

우선순위: **명령줄 옵션 > 환경변수(`PROWLARR_URL`, `PROWLARR_API_KEY`, `QBIT_URL`, `QBIT_USER`, `QBIT_PASS`) > 설정 파일 > 기본값**

## 사용

```bash
tget "ubuntu 24.04"                 # 검색 → 번호 선택 → qBittorrent 에 추가
tget ubuntu --dry-run               # 결과만 보기
tget ubuntu --first --watch         # 시더 1등을 바로 받고 진행률 보기 (Ctrl+C 로 그만 보기)
tget ubuntu --min-seeders 5 -l 5    # 시더 5 이상, 5개까지
tget ubuntu --save-path /downloads  # 저장 경로 (qBittorrent 기준)
```

## 개발

서버 없이 `fixtures/search.json` 으로 돌려 볼 수 있습니다.

```bash
cargo run -- ubuntu --fixture fixtures/search.json --dry-run
cargo test
```
