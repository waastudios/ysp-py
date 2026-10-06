# ysp-py

> **Acknowledgments**
> 1. Thanks to [IPTV Official Group](http://t.me/iptvorganization) for sharing
> 2. Thanks to Gary's Club for the ysp-live v8.1 base

ysp-live v8.1 Python single-file edition — a CCTV/CGTN live stream proxy gateway. Runs anywhere with Python 3.8+, no Docker, no dependencies, no build step. Designed for APTV's Python backend service (import via remote URL), but works on any server, VPS, NAS, or local machine.

## What it does

- Proxies 30 CCTV/CGTN live channels through multiple upstream strategies (device protocol for true 4K → 1080p JCE → bkliveinfo), automatically falling back when one fails
- Generates a standard M3U playlist (`/cctv.m3u`) with proper `tvg-id`, logos, and 7-day catchup metadata
- Aggregates EPG from two upstream sources into a single local XMLTV (`/epg.xml`), filtered to exactly the 30 channels in the playlist — including CCTV-8K which most public EPGs lack
- 4K channels (CCTV-4K, CCTV-8K, CCTV-16 4K) request UHD streams via device protocol; all other channels locked to 1080p high bitrate (`fhd`)

## Features

- **30 channels, 3 groups**: 央视FHD (CCTV-1~17) / 央视UHD (4K/8K) / CGTN (6 languages) — no local stations, no clutter
- **True 4K**: device protocol for UHD channels, not upscaled 1080p
- **7-day catchup**: `catchup="append"` on supported channels, 7-day replay window
- **Built-in EPG**: `/epg.xml` merged from `epg.pw` + `zsdc.eu.org`, auto-refreshes every 6 hours, degrades gracefully if one source fails
- **Zero dependencies**: pure Python stdlib, single 179KB file
- **APTV-ready**: import the raw GitHub URL as a Python backend service, no server setup needed

## Quick Start

### Option A: APTV (iOS) — no server needed

1. In APTV, go to **Services** → **+** → add Python service via remote URL
2. Paste: `https://raw.githubusercontent.com/waastudios/ysp-py/main/ysp-live.py`
3. Start the service (default port 8767)
4. In APTV channels, add subscription: `http://localhost:8767/cctv.m3u`
5. Set EPG URL: `http://localhost:8767/epg.xml`

### Option B: Any server / VPS / local machine

```bash
# run directly from URL (single file, nothing to install)
curl -sSL https://raw.githubusercontent.com/waastudios/ysp-py/main/ysp-live.py | python3

# or download first, then run
curl -sSL https://raw.githubusercontent.com/waastudios/ysp-py/main/ysp-live.py -o ysp-live.py
python3 ysp-live.py                 # default port 8767
python3 ysp-live.py --port 8080    # custom port
python3 ysp-live.py --no-4k        # disable device protocol (1080p only)
```

Then in your player:
- Playlist: `http://<server-ip>:8767/cctv.m3u`
- EPG: `http://<server-ip>:8767/epg.xml`

> Replace `<server-ip>` with your machine's LAN or public IP. `localhost` only works on the machine running it.

## Endpoints

| Path | Description |
|------|-------------|
| `/cctv.m3u` | M3U playlist — 30 channels, grouped, with EPG link |
| `/epg.xml` | Aggregated XMLTV EPG — 30 channels, 6h auto-refresh |
| `/diag` | Live diagnostics (engine status, channel health) |
| `/health` | Simple health check |
| `/<channel>.m3u8` | Per-channel HLS stream, e.g. `/cctv1.m3u8`, `/cctv4k.m3u8` |
| `/proxy.ts` | TS segment proxy |

## Channel Groups

| Group | Channels |
|-------|----------|
| **央视FHD** | CCTV-1 综合 … CCTV-17 农业农村 (1080p high bitrate) |
| **央视UHD** | CCTV-4K, CCTV-8K, CCTV-16 4K (true 4K via device protocol) |
| **CGTN** | CGTN English, Français, Русский, العربية, Español, Documentary |

Plus 3 theatre channels (CCTV第一剧场 / 风云剧场 / 怀旧剧场) under 央视FHD.

## 4K channels note

When opening a 4K channel (CCTV-4K, CCTV-8K, CCTV-16 4K) for the first time, **please wait at least 1 minute** before expecting video.

4K streams go through the device protocol, which needs device registration, warm-up and stream fetching (~30-60s). Black screen or buffering during this time is normal; 4K video appears once warm-up completes.

> APTV has no Node.js runtime, so the `ysp-engine.js` WASM fallback engine is unavailable. 4K relies entirely on the device protocol. If device registration fails, 4K channels automatically fall back to 1080p.

## How the stream fallback works

For each channel request, the gateway tries in order:

1. **Device protocol** (4K/8K channels) — registered virtual device, highest quality
2. **1080p JCE** —央视 JCE API, high-bitrate HLS
3. **bkliveinfo** — backup API with signed URLs
4. (Docker edition only) **WASM engine** — Node.js fallback

If a layer fails, the next is tried transparently. The player always gets a working stream.

## EPG sources

| Source | Format | Coverage |
|--------|--------|----------|
| `epg.pw/xmltv/epg_CN.xml.gz` | numeric IDs → mapped | CCTV 1-17, 4K/8K, theatres, CGTN纪录/西/俄/阿/法 |
| `epg.zsdc.eu.org/t.xml.gz` | Chinese-name IDs | CGTN English, CCTV overflow |

Merged, deduplicated, filtered to the 30 playlist channels. Refreshes every 6 hours in background; serves stale cache if both sources fail.

## Uninstall

```bash
# APTV: stop the service and delete it in the app
# Server: Ctrl+C, then
rm -f ysp-live.py
```
