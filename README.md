# ysp-web-py

> **Acknowledgments**
> 1. Thanks to [IPTV Official Group](http://t.me/iptvorganization) for sharing
> 2. Thanks to Gary's Club for the ysp-live v8.1 base

ysp-live v8.1 Python single-file version — CCTV/CGTN live stream proxy. No Docker required, just `python3`.

## Features

- 30 channels (CCTV + CGTN only), 3 groups: 央视FHD / 央视UHD / CGTN
- High bitrate locked by default (fhd for all channels)
- 7-day catchup for supported channels
- Built-in EPG aggregator (`/epg.xml`), merged from two upstream sources
- Single file, zero dependencies (Python 3.8+ stdlib only)

## Quick Start

```bash
# download
curl -sSL https://raw.githubusercontent.com/waastudios/ysp-web-rs/main/ysp-live.py -o ysp-live.py
curl -sSL https://raw.githubusercontent.com/waastudios/ysp-web-rs/main/epg_agg.py -o epg_agg.py

# run (default port 8767)
python3 ysp-live.py

# custom port
python3 ysp-live.py --port 8080
```

Then open in your player:
- Subscription: `http://<your-ip>:8767/cctv.m3u`
- EPG: `http://<your-ip>:8767/epg.xml`

> Replace `<your-ip>` with your server's public IP. `localhost` only works on the server itself.

## Endpoints

| Path | Description |
|------|-------------|
| `/cctv.m3u` | Playlist (30 channels) |
| `/epg.xml` | Aggregated EPG (XMLTV) |
| `/diag` | Diagnostics |
| `/health` | Health check |
| `/<channel>.m3u8` | Channel stream, e.g. `/cctv1.m3u8` |

## Channel Groups

- **央视FHD** — CCTV-1..17 (1080p high bitrate)
- **央视UHD** — CCTV-4K, CCTV-8K, CCTV-16 4K
- **CGTN** — CGTN English/French/Russian/Arabic/Spanish/Documentary

## Uninstall

```bash
# stop the process (Ctrl+C if running in foreground)
# or if running in background:
pkill -f ysp-live.py
rm -f ysp-live.py epg_agg.py
```
