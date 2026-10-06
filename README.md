# ysp-web-rs

> **Acknowledgments**
> 1. Thanks to [IPTV Official Group](http://t.me/iptvorganization) for sharing the ysp-web-rs source code
> 2. Thanks to NOX for the original CMG decryption implementation

**中文文档**: [README-CN.md](README-CN.md)

CCTV live streaming via Rust — 62 channels with WASM-based CMG decryption.

## What's inside

- **62 channels** with groups (`group-title`): 央视FHD / 央视UHD / CGTN / 地方台 / 其他
  - 央视FHD (21): CCTV-1~17, 5+, 6, 3 theater channels
  - 央视UHD (3): CCTV-4K, CCTV-8K, CCTV-16 4K
  - CGTN (6): CGTN + 5 language channels
  - 地方台 (31): Provincial satellite channels
  - 其他 (1): CETV-1
- **Rust single binary** with WASM-based CMG decryption (official player logic)
- TS decrypt & remux in native Rust
- Single port (8767)

## Docker deploy

```bash
docker build -f Dockerfile -t ysp-web-rs .
docker run -d --name ysp-web-rs -p 8767:8767 --restart unless-stopped ysp-web-rs
```

## Usage

- Subscription:
  ```
  http://<IP>:8767/list.m3u
  ```
- Aggregated EPG (62 channels, refreshes every 6h):
  ```
  http://<IP>:8767/epg.xml
  ```
- Channels API: `http://<IP>:8767/channels`
- Health: `http://<IP>:8767/health`

## EPG Guide Subscription

This project ships a built-in aggregated EPG endpoint (see Usage above for the URL) — no need to configure third-party EPG sources manually:

- Content: programme guide for only the 62 channels in this project (CCTV FHD / CCTV UHD / CGTN / regional / others); irrelevant channels are filtered out
- Upstream sources (merged automatically; the two complement each other, one going down won't break the other):
  - `https://live.fanmingming.com/e.xml`
  - `https://epg.112114.xyz/pp.xml.gz`
- Refresh: auto-updates every 6 hours; fetched in the background on first access, just retry after a moment

The `/list.m3u` playlist already points to this address, so players load the guide automatically.

## Notes

- Device registration works from datacenter VPS (tested).
- 4K channels may be geo-blocked from overseas IPs (CCTV copyright policy).
