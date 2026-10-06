# ysp-web-rs

> **致谢**
> 1. 感谢 [IPTV 总部](http://t.me/iptvorganization)分享的 ysp-web-rs 源码
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

- 订阅：
  ```
  http://<IP>:8767/list.m3u
  ```
- 聚合 EPG（62 路，6 小时刷新）：
  ```
  http://<IP>:8767/epg.xml
  ```
- 频道 API：`http://<IP>:8767/channels`
- 健康检查：`http://<IP>:8767/health`

## EPG 节目单订阅

本项目自带聚合 EPG 接口，开箱即用，不用再手动填第三方 EPG 源：

- 地址（复制即用，把 `<IP>` 换成你的 VPS 公网 IP 或内网 IP）：
  ```
  http://<IP>:8767/epg.xml
  ```
- 内容：只包含本项目 62 路频道的节目单（央视FHD/央视UHD/CGTN/地方台/其他），无用频道已过滤
- 数据源：自动合并以下两个上游 EPG（两源互补，单个源挂了不影响）：
  - `https://live.fanmingming.com/e.xml`
  - `https://epg.112114.xyz/pp.xml.gz`
- 刷新：每 6 小时自动更新；首次访问时后台拉取，稍等片刻再刷新即可

`/list.m3u` 订阅已默认指向该地址，播放器会自动加载节目单。

## 注意

- 设备注册在机房 VPS 上可用（已实测）。
- 4K 频道在海外 IP 可能被地域锁（央视版权政策）。
