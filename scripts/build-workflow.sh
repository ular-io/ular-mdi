#!/usr/bin/env bash
#
# .alfredworkflow 패키지를 로컬에서 굽는다 (CI release.yml 과 동일한 절차).
#
# 사용법:
#   scripts/build-workflow.sh [--universal]
#
#   --universal        Apple Silicon + Intel universal 바이너리 (기본: 현재 아키텍처)
#
# 패키지에는 아이콘(PNG·meta.json)을 넣지 않는다. Alfred 설정 폴더는 클라우드로 동기화되는데
# 아이콘이 30MB 넘게 차지하기 때문이다. 아이콘은 첫 검색 때 $alfred_workflow_data 로 내려받는다.
#
# 산출물: <repo>/ular-mdi.alfredworkflow
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

UNIVERSAL=0
while [ $# -gt 0 ]; do
  case "$1" in
    --universal) UNIVERSAL=1; shift ;;
    *) echo "알 수 없는 옵션: $1" >&2; exit 2 ;;
  esac
done

# cargo PATH 보장
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

echo "==> 1/2 바이너리 빌드"
if [ "$UNIVERSAL" -eq 1 ]; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null
  cargo build --release --target aarch64-apple-darwin
  cargo build --release --target x86_64-apple-darwin
  BIN="$REPO/target/mdi-universal"
  lipo -create -output "$BIN" \
    target/aarch64-apple-darwin/release/mdi \
    target/x86_64-apple-darwin/release/mdi
else
  cargo build --release
  BIN="$REPO/target/release/mdi"
fi
echo "    바이너리: $BIN"

echo "==> 2/2 패키지 조립"
PKG="$REPO/pkg"
rm -rf "$PKG"
mkdir -p "$PKG"
cp "$REPO/workflow/info.plist" "$PKG/"
[ -f "$REPO/workflow/icon.png" ] && cp "$REPO/workflow/icon.png" "$PKG/"
cp "$BIN" "$PKG/mdi"
chmod +x "$PKG/mdi"
cp "$REPO/LICENSE" "$REPO/LICENSE-MDI" "$PKG/"

OUT="$REPO/ular-mdi.alfredworkflow"
rm -f "$OUT"
( cd "$PKG" && zip -qr -X "$OUT" . )
rm -rf "$PKG"

echo
echo "완료: $OUT"
echo "  크기: $(du -h "$OUT" | cut -f1)  파일: $(zipinfo -1 "$OUT" | wc -l | tr -d ' ')개"
