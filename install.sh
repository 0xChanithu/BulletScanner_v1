#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; BIN="$HOME/.local/bin"
command -v cargo >/dev/null || { echo 'Install cargo first'; exit 1; }
command -v nmap >/dev/null || { echo 'Install nmap first'; exit 1; }
mkdir -p "$BIN"
cargo build --release --manifest-path "$ROOT/rust-scanner/Cargo.toml"
cp "$ROOT/rust-scanner/target/release/miniscanner-portscan" "$BIN/miniscanner-portscan"
cp "$ROOT/scripts/scan.py" "$BIN/scan.py"
printf '#!/usr/bin/env bash\nexec python3 "%s/scan.py" "$@"\n' "$BIN" > "$BIN/scan"
chmod +x "$BIN/scan" "$BIN/scan.py" "$BIN/miniscanner-portscan"
echo "Installed. Add $BIN to PATH if needed."
