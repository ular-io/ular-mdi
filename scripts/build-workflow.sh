#!/usr/bin/env bash
#
# .alfredworkflow 패키지를 로컬에서 굽는다 (CI release.yml 과 동일한 절차).
#
# 사용법:
#   scripts/build-workflow.sh [--universal] [--size N] [--mdi-version X.Y.Z]
#
#   --universal        Apple Silicon + Intel universal 바이너리 (기본: 현재 아키텍처)
#   --size N           PNG 굽는 크기 px (기본 88)
#   --mdi-version X    특정 @mdi/svg 버전 고정 (기본: latest)
#
# 산출물: <repo>/ular-mdi.alfredworkflow
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

UNIVERSAL=0
SIZE=88
MDI_VERSION="latest"
while [ $# -gt 0 ]; do
  case "$1" in
    --universal) UNIVERSAL=1; shift ;;
    --size) SIZE="$2"; shift 2 ;;
    --mdi-version) MDI_VERSION="$2"; shift 2 ;;
    *) echo "알 수 없는 옵션: $1" >&2; exit 2 ;;
  esac
done

# cargo PATH 보장
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

MDI_DIR=/tmp/mdi/node_modules/@mdi/svg

echo "==> 1/4 바이너리 빌드"
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

echo "==> 2/4 @mdi/svg 가져오기 ($MDI_VERSION)"
npm install "@mdi/svg@$MDI_VERSION" --prefix /tmp/mdi --no-audit --no-fund >/dev/null
VER="$(node -p "require('$MDI_DIR/package.json').version")"
echo "    버전: $VER"

echo "==> 3/4 아이콘 굽기 (${SIZE}px)"
"$BIN" bake \
  --svg-dir "$MDI_DIR/svg" \
  --meta    "$MDI_DIR/meta.json" \
  --out-dir "$REPO/assets" \
  --version "$VER" \
  --size "$SIZE"

echo "==> 4/4 패키지 조립"
PKG="$REPO/pkg"
rm -rf "$PKG"
mkdir -p "$PKG/icons"
cp "$REPO/workflow/info.plist" "$PKG/"
[ -f "$REPO/workflow/icon.png" ] && cp "$REPO/workflow/icon.png" "$PKG/"
cp "$BIN" "$PKG/mdi"
chmod +x "$PKG/mdi"
cp "$REPO/assets/meta.json" "$PKG/"
cp -R "$REPO/assets/icons/." "$PKG/icons/"
cp "$REPO/LICENSE" "$REPO/LICENSE-MDI" "$PKG/"

OUT="$REPO/ular-mdi.alfredworkflow"
rm -f "$OUT"
( cd "$PKG" && zip -qr -X "$OUT" . )
rm -rf "$PKG"

echo
echo "완료: $OUT"
echo "  크기: $(du -h "$OUT" | cut -f1)  파일: $(zipinfo -1 "$OUT" | wc -l | tr -d ' ')개  (@mdi/svg $VER)"
