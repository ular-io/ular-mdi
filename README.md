# ular-mdi

가볍고 빠른 오프라인 **Material Design Icons (Pictogrammers MDI)** 검색 Alfred 워크플로우.

매 검색마다 온라인 호출/렌더링을 하던 기존 워크플로우와 달리, 로컬에 구워둔 PNG 캐시와 `meta.json` 필터링만으로 동작합니다. 아이콘은 첫 검색 때 한 번 내려받습니다(약 10초).

> Pictogrammers MDI는 커뮤니티 프로젝트로, Google Material Symbols와는 별개입니다.

<!-- TODO: 검색 중 아이콘이 실시간으로 표시되는 스크린샷 -->

## 사용법

| 키워드 | 동작 |
| :--- | :--- |
| `mdi <검색어>` | 아이콘 이름·aliases·tags 로 퍼지/부분 검색. 결과에 아이콘 미리보기 표시. |
| `mdi-update` | 최신 `@mdi/svg` 와 비교해 신규 아이콘만 증분 갱신. 첫 내려받기 실패 후 재시도에도 사용. |

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
2. `mdi` 를 처음 입력하면 "MDI 아이콘 내려받는 중…" 이 표시되고, 백그라운드로 최신 `@mdi/svg` 를 받아 PNG 를 굽는다. 완료되면 결과가 자동으로 표시된다. Mac 마다 한 번.

### Gatekeeper (서명/공증 없이 배포)

차단 시 둘 중 하나:

- **시스템 설정 → 개인정보 보호 및 보안 → "그래도 열기"**, 또는
- 터미널에서 격리 속성 제거:
  ```sh
  xattr -dr com.apple.quarantine ~/path/to/workflow
  ```

## 동작 방식

- 배포 패키지에는 단일 정적 바이너리(`mdi`)와 `info.plist` 만 들어 있다. Alfred 설정 폴더는 iCloud·Google Drive 등으로 동기화되므로, 30MB 넘는 아이콘을 패키지에서 빼서 동기화 용량을 줄인다.
- 아이콘(PNG + augmented `meta.json`)은 동기화되지 않는 `$alfred_workflow_data`(`~/Library/Application Support/Alfred/Workflow Data/io.ular.mdi`)에 둔다.
- 검색 시 `meta.json` 이 없으면 백그라운드로 `mdi update` 를 시작하고(잠금 파일 `.updating` 으로 중복 방지, 15분 지나면 무효), Script Filter `rerun` 으로 완료를 기다린다. 실패하면 `update-failed.txt` 에 원인을 남기고 자동 재시도하지 않는다(`mdi-update` 로 재시도).
- `meta.json` 에는 각 아이콘의 path `d` 데이터를 미리 추출해 넣어, SVG/path 출력에 원본 SVG 파일이 불필요.
- 이전 버전처럼 워크플로 폴더에 `meta.json`/`icons/` 가 있으면 그것도 읽는다(overlay 우선, 없으면 워크플로 폴더).

## 빌드

`.alfredworkflow` 패키지를 로컬에서 굽는다 (CI 와 동일한 절차):

```sh
./scripts/build-workflow.sh              # 현재 아키텍처
./scripts/build-workflow.sh --universal  # Apple Silicon + Intel
```

산출물은 `ular-mdi.alfredworkflow`. 실제 배포는 태그 푸시 시 `.github/workflows/release.yml` 가 자동 수행한다.

## 라이선스

- 본 워크플로우 코드: **MIT** (`LICENSE`).
- 번들된 아이콘(PNG·path 데이터): **[Pictogrammers MDI](https://pictogrammers.com/library/mdi/)** — Pictogrammers Free License (아이콘 Apache 2.0). 전문·고지는 `LICENSE-MDI` 참조. SVG→PNG 변환·path 추출 외 원본 아트워크 변경 없음. Apache 2.0은 상표권을 부여하지 않으므로 브랜드/로고 아이콘은 각 상표권자 권리가 별도로 적용됩니다.
- SVG 렌더링: [resvg](https://github.com/linebender/resvg) 계열 (**MPL-2.0**).
