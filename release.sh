#!/usr/bin/env bash
# Publie une version : ./release.sh 0.2.0 "Ce qui change"
# Compile sur la machine de build (Linux + cargo-xwin), signe, puis crée la release GitHub.
# Les applis installées la voient dans les 6 h (ou au prochain lancement) et proposent la mise à jour.
set -euo pipefail

V="${1:?version, ex. 0.2.0}"
NOTES="${2:-Version $V}"
BUILD="${FOYER_BUILD:-rayan@192.168.1.244}"
SSH=(ssh -i "$HOME/.ssh/claude_box" -o IdentitiesOnly=yes)
REPO="arayan48/foyer"
ROOT="$(cd "$(dirname "$0")" && pwd)"
OUT="$ROOT/dist/v$V"

cd "$ROOT"
sed -i '' "s/^version = \".*\"/version = \"$V\"/" agent/Cargo.toml
python3 -c "import json,sys;p='app/src-tauri/tauri.conf.json';c=json.load(open(p));c['version']='$V';json.dump(c,open(p,'w'),indent=2)"

rsync -a --exclude target --exclude node_modules --exclude dist --exclude src-tauri/binaries -e "${SSH[*]}" agent app "$BUILD:/srv/foyer/"
"${SSH[@]}" "$BUILD" bash -s <<'EOF'
set -euo pipefail
. ~/.cargo/env
cd /srv/foyer/agent && cargo build --release --target x86_64-pc-windows-gnu
cd /srv/foyer/app
mkdir -p src-tauri/binaries
cp ../agent/target/x86_64-pc-windows-gnu/release/foyer-agent.exe src-tauri/binaries/foyer-agent-x86_64-pc-windows-msvc.exe
npm ci --silent
export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/foyer.key)" TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
npx tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc
npx tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --config src-tauri/tauri.server.json
EOF

mkdir -p "$OUT"
N=/srv/foyer/app/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis
scp -q -i "$HOME/.ssh/claude_box" -o IdentitiesOnly=yes "$BUILD:$N/Foyer_${V}_x64-setup.exe" "$OUT/Foyer-Client.exe"
scp -q -i "$HOME/.ssh/claude_box" -o IdentitiesOnly=yes "$BUILD:$N/Foyer_${V}_x64-setup.exe.sig" "$OUT/Foyer-Client.exe.sig"
scp -q -i "$HOME/.ssh/claude_box" -o IdentitiesOnly=yes "$BUILD:$N/Foyer\\ Serveur_${V}_x64-setup.exe" "$OUT/Foyer-Serveur.exe"
scp -q -i "$HOME/.ssh/claude_box" -o IdentitiesOnly=yes "$BUILD:$N/Foyer\\ Serveur_${V}_x64-setup.exe.sig" "$OUT/Foyer-Serveur.exe.sig"

# manifestes lus par le module de mise à jour
for kind in Client Serveur; do
  python3 - "$OUT" "$kind" "$V" "$NOTES" "$REPO" <<'PY'
import json, sys, datetime
out, kind, v, notes, repo = sys.argv[1:]
sig = open(f"{out}/Foyer-{kind}.exe.sig").read().strip()
m = {"version": v, "notes": notes, "pub_date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
     "platforms": {"windows-x86_64": {"signature": sig, "url": f"https://github.com/{repo}/releases/download/v{v}/Foyer-{kind}.exe"}}}
json.dump(m, open(f"{out}/{kind.lower()}.json", "w"), indent=2)
PY
done

git add -A
git commit -qm "Version $V" || true
git push -q origin HEAD
gh release create "v$V" --repo "$REPO" --title "Foyer $V" --notes "$NOTES" \
  "$OUT/Foyer-Serveur.exe" "$OUT/Foyer-Client.exe" "$OUT/client.json" "$OUT/serveur.json"
echo "Publié : https://github.com/$REPO/releases/tag/v$V"
