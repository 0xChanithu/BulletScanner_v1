# MiniScanner X v2

Lightweight hybrid scanner for authorized lab environments: Rust async TCP discovery with one retry, followed by Nmap service/version and optional OS detection.

## Kali install
```bash
sudo apt update
sudo apt install -y cargo nmap python3
chmod +x install.sh
./install.sh
export PATH="$HOME/.local/bin:$PATH"
```

## Usage
```bash
scan 192.168.1.10
scan 192.168.1.10 --ports 1-65535
scan 192.168.1.10 --concurrency 500 --timeout-ms 700 --retries 1
scan 192.168.1.10 --no-os
```
Only scan systems you own or are authorized to test.
