# ysp-py

[English](README.md)

> **致谢**
> 1. 感谢 [IPTV 总部](http://t.me/iptvorganization) 的分享
> 2. 感谢 Gary's Club 提供的 ysp-live v8.1 基础包

ysp-live v8.1 Python 单文件版 —— 央视/CGTN 直播流代理网关。有 Python 3.8 就能跑，无需 Docker、零依赖、不用编译。为 APTV 的 Python 后端服务设计（远程链接直接导入），也可在任何服务器、VPS、NAS 或本机运行。

## 它能做什么

- 通过多层上游策略代理 30 路央视/CGTN 直播（设备协议真 4K → 1080p JCE → bkliveinfo），某层失败自动降级，播放器永远拿到可用流
- 生成标准 M3U 订阅（`/cctv.m3u`），带正确的 `tvg-id`、台标和 7 天回看元数据
- 内置 EPG 聚合：两个上游源合并成一份本地 XMLTV（`/epg.xml`），只含订阅里的 30 路频道——包括多数公共 EPG 没有的 CCTV-8K
- 4K 频道（CCTV-4K、CCTV-8K、CCTV-16 4K）走设备协议拿真 4K；其余频道全部锁死 1080p 高码率（`fhd`）

## 特性

- **30 路、3 分组**：央视FHD（CCTV-1~17）/ 央视UHD（4K/8K）/ CGTN（6 种语言）——无地方台、无杂台
- **真 4K**：UHD 频道走设备协议，不是 1080p 拉伸
- **7 天回看**：支持的频道带 `catchup="append"`，7 天回看窗口
- **内置节目单**：`/epg.xml` 由 `epg.pw` + `zsdc.eu.org` 聚合，6 小时自动刷新，单源挂了自动降级
- **零依赖**：纯 Python 标准库，单个 179KB 文件
- **APTV 即用**：把 GitHub raw 链接填进 APTV 的 Python 服务就行，不用搭服务器

## 快速开始

### 方案 A：APTV（iOS）——不用服务器

1. APTV 里进 **服务** → **+** → 通过远程链接添加 Python 服务
2. 粘贴：`https://raw.githubusercontent.com/waastudios/ysp-py/main/ysp-live.py`
3. 启动服务（默认 8767 端口）
4. APTV 频道里添加订阅：`http://localhost:8767/cctv.m3u`
5. 节目单地址填：`http://localhost:8767/epg.xml`

### 方案 B：服务器 / VPS / 本机

```bash
# 直接从 URL 运行（单文件，什么都不用装）
curl -sSL https://raw.githubusercontent.com/waastudios/ysp-py/main/ysp-live.py | python3

# 或先下载再运行
curl -sSL https://raw.githubusercontent.com/waastudios/ysp-py/main/ysp-live.py -o ysp-live.py
python3 ysp-live.py                 # 默认 8767 端口
python3 ysp-live.py --port 8080    # 自定义端口
python3 ysp-live.py --no-4k        # 禁用设备协议（纯 1080p）
```

播放器里填：
- 订阅：`http://<服务器IP>:8767/cctv.m3u`
- 节目单：`http://<服务器IP>:8767/epg.xml`

> `<服务器IP>` 换成机器的局域网或公网 IP。`localhost` 只在运行它的本机有效。

## 接口

| 地址 | 说明 |
|------|------|
| `/cctv.m3u` | M3U 订阅 —— 30 路、已分组、带节目单链接 |
| `/epg.xml` | 聚合 XMLTV 节目单 —— 30 路、6 小时自动刷新 |
| `/diag` | 实时诊断（引擎状态、频道健康度） |
| `/health` | 健康检查 |
| `/<频道>.m3u8` | 单频道 HLS 流，如 `/cctv1.m3u8`、`/cctv4k.m3u8` |
| `/proxy.ts` | TS 切片代理 |

## 频道分组

| 分组 | 频道 |
|------|------|
| **央视FHD** | CCTV-1 综合 … CCTV-17 农业农村（1080p 高码率） |
| **央视UHD** | CCTV-4K、CCTV-8K、CCTV-16 4K（设备协议真 4K） |
| **CGTN** | CGTN 英语、法语、俄语、阿语、西语、纪录 |

另有 3 路剧场频道（CCTV第一剧场 / 风云剧场 / 怀旧剧场）归在央视FHD。

## 4K 频道说明

首次打开 4K 频道（CCTV-4K、CCTV-8K、CCTV-16 4K）时，**请至少等待 1 分钟**再看画面。

原因是 4K 流走设备协议，需要先完成设备注册、预热和拉流，整个过程约 30~60 秒。等待期间播放器可能黑屏或转圈，属正常现象，预热完成后即出 4K 画面。

> APTV 无 Node.js 环境，`ysp-engine.js` WASM 兜底引擎不可用，4K 完全依赖设备协议。若设备注册失败，4K 频道会自动降级为 1080P。

## 直播流 fallback 机制

每次请求频道时，网关按顺序尝试：

1. **设备协议**（4K/8K 频道）——注册虚拟设备，画质最高
2. **1080p JCE** —— 央视 JCE 接口，高码率 HLS
3. **bkliveinfo** —— 备用接口，带签名 URL
4. （仅 Docker 版）**WASM 引擎** —— Node.js 兜底

某层失败自动试下一层，播放器无感知。

## 节目单上游源

| 来源 | 格式 | 覆盖 |
|------|------|------|
| `epg.pw/xmltv/epg_CN.xml.gz` | 数字 ID → 映射 | CCTV 1-17、4K/8K、剧场、CGTN纪录/西/俄/阿/法 |
| `epg.zsdc.eu.org/t.xml.gz` | 中文名 ID | CGTN 英语、CCTV 补漏 |

合并去重，只保留订阅里的 30 路。后天每 6 小时刷新；两源都挂时继续 serving 旧缓存。

## 卸载

```bash
# APTV：在 App 里停止服务并删除
# 服务器：Ctrl+C，然后
rm -f ysp-live.py
```
