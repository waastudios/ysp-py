# ysp-web-rs

> **致谢**
> 1. 感谢 IPTV 总部分享的 ysp-web-rs 源码
> 2. 感谢 NOX 大神的 CMG 解密原始实现

**English README**: [README.md](README.md)

Rust 实现的央视频直播——62 路，WASM CMG 解密。

## 包含内容

- **62 路频道**，带分组（`group-title`）：央视FHD / 央视UHD / CGTN / 地方台 / 其他
  - 央视FHD (21)：CCTV-1~17、5+、6、三个剧场
  - 央视UHD (3)：CCTV-4K、CCTV-8K、CCTV-16 4K
  - CGTN (6)：CGTN 主频道 + 5 个语种
  - 地方台 (31)：省级卫视
  - 其他 (1)：CETV-1
- **Rust 单二进制**，WASM CMG 解密（官方播放器逻辑）
- Rust 原生 TS 解密与重混
- 单端口（8767）

## Docker 部署

```bash
docker build -f Dockerfile -t ysp-web-rs .
docker run -d --name ysp-web-rs -p 8767:8767 --restart unless-stopped ysp-web-rs
```

## 使用

- 订阅：`http://<IP>:8767/list.m3u`
- 频道 API：`http://<IP>:8767/channels`
- 健康检查：`http://<IP>:8767/health`

## 注意

- 设备注册在机房 VPS 上可用（已实测）。
- 4K 频道在海外 IP 可能被地域锁（央视版权政策）。
