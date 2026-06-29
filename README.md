# ular-mdi

가볍고 빠른 오프라인 **Material Design Icons (Pictogrammers MDI)** 검색 Alfred 워크플로우.

매 검색마다 온라인 호출/렌더링을 하던 기존 워크플로우와 달리, 미리 구워둔 PNG 캐시와 로컬 `meta.json` 필터링만으로 동작합니다. 설치 직후 바로 검색 가능.

> Pictogrammers MDI는 커뮤니티 프로젝트로, Google Material Symbols와는 별개입니다.

<!-- TODO: 검색 중 아이콘이 실시간으로 표시되는 스크린샷 -->

## 사용법

| 키워드 | 동작 |
| :--- | :--- |
| `mdi <검색어>` | 아이콘 이름·aliases·tags 로 퍼지/부분 검색. 결과에 아이콘 미리보기 표시. |
| `mdi-update` | 최신 `@mdi/svg` 와 비교해 신규 아이콘만 증분 갱신. |

### `mdi` 검색 결과에서

| 키 | 출력 |
| :--- | :--- |
| `Enter` | 아이콘 이름 (`mdi-account`) |
| `Cmd+Enter` | SVG 코드 전체 |
| `Opt+Enter` | SVG path data |
| `Ctrl+Enter` | PNG 파일 |

예) `trash` → `delete`, `acct` → `account` 매칭(부분/퍼지).

## 설정 (Alfred Workflow Configuration)

| 변수 | 기본값 | 설명 |
| :--- | :--- | :--- |
| `mdi_size` | `88` | PNG 굽는 크기(px) |
| `mdi_color` | `#000000` | 아이콘 색 (다크모드 대응 시 `#ffffff`) |
| `mdi_limit` | `30` | 검색 결과 최대 개수 |

## 설치

1. [Releases](https://github.com/ular-io/ular-mdi/releases) 에서 `ular-mdi.alfredworkflow` 다운로드 후 더블클릭.
2. 미리 구운 PNG + `meta.json` 이 동봉되어 있어 즉시 검색 가능.

### Gatekeeper (서명/공증 없이 배포)

차단 시 둘 중 하나:

- **시스템 설정 → 개인정보 보호 및 보안 → "그래도 열기"**, 또는
- 터미널에서 격리 속성 제거:
  ```sh
  xattr -dr com.apple.quarantine ~/path/to/workflow
  ```

## 동작 방식

- 배포 패키지에 단일 정적 바이너리(`mdi`) + 미리 구운 PNG 캐시 + augmented `meta.json` 동봉.
- `meta.json` 에는 각 아이콘의 path `d` 데이터를 미리 추출해 넣어, SVG/path 출력에 원본 SVG 파일이 불필요.
- `mdi-update` 는 신규 PNG를 `$alfred_workflow_data` 에 기록(overlay). 검색은 overlay 우선, 없으면 동봉본으로 폴백.

## 빌드

```sh
cargo build --release
# 릴리스 자산 굽기 (CI 와 동일)
npm install @mdi/svg@latest --prefix /tmp/mdi
./target/release/mdi bake \
  --svg-dir /tmp/mdi/node_modules/@mdi/svg/svg \
  --meta    /tmp/mdi/node_modules/@mdi/svg/meta.json \
  --out-dir assets --version <ver> --size 88
```

## 라이선스

- 본 워크플로우 코드: **MIT** (`LICENSE`).
- 번들된 아이콘(PNG·path 데이터): **[Pictogrammers MDI](https://pictogrammers.com/library/mdi/)** — Pictogrammers Free License (아이콘 Apache 2.0). 전문·고지는 `LICENSE-MDI` 참조. SVG→PNG 변환·path 추출 외 원본 아트워크 변경 없음. Apache 2.0은 상표권을 부여하지 않으므로 브랜드/로고 아이콘은 각 상표권자 권리가 별도로 적용됩니다.
- SVG 렌더링: [resvg](https://github.com/linebender/resvg) 계열 (**MPL-2.0**).
