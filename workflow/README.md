# Alfred 워크플로우 템플릿

릴리스 패키징(`.github/workflows/release.yml`)이 이 디렉터리의 파일을 패키지 루트로 복사한다.

| 파일 | 상태 | 설명 |
| :--- | :--- | :--- |
| `info.plist` | **작성 완료** | bundleid `io.ular.mdi`. 아래 구조대로 생성됨 (`plutil -lint` OK). |
| `icon.png` | 미작성 (선택) | 워크플로우 아이콘 256×256. 없으면 CI가 건너뜀. |

## info.plist 에 정의할 오브젝트 (설계)

- **Script Filter** — 키워드 `mdi`, "Argument Optional" + with-space.
  - Script: `./mdi search "$1"` (with input as `{query}`, type=argv).
  - 연결:
    - 기본(Enter) → **Copy to Clipboard** (text) — arg = `mdi-<name>`
    - `cmd` → **Copy to Clipboard** (text) — arg = SVG 코드
    - `alt` → **Copy to Clipboard** (text) — arg = path data
    - `ctrl` → **Copy/Paste 파일** 액션 — arg = PNG 경로
  - modifier별 arg/subtitle 은 Script Filter JSON 의 `mods` 가 제공.
- **Keyword** — `mdi-update`, "No Argument" → **Run Script** `./mdi update` → (선택) **Post Notification**.
  - 단, 알림은 바이너리가 osascript 로 직접 표시하므로 Post Notification 오브젝트는 선택.
- **Workflow Configuration** — `mdi_size` / `mdi_color` / `mdi_limit` 사용자 노출.

> 실제 `info.plist` 는 `alfred` 스킬로 UUID·연결·plist 구조를 정확히 생성한다.
