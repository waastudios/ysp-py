# ysp-web-py

> **致谢**
> 1. 感谢 [IPTV 总部](http://t.me/iptvorganization) 的分享
> 2. 感谢 Gary's Club 提供的 ysp-live v8.1 基础包

ysp-live v8.1 Python 单文件版 —— 央视/CGTN 直播流代理。无需 Docker，一条 `python3` 命令即跑。

## 特性

- 30 路频道（仅 CCTV + CGTN），3 个分组：央视FHD / 央视UHD / CGTN
- 默认锁死高码率（全频道 fhd）
- 支持 7 天回看（部分频道）
- 内置 EPG 聚合（`/epg.xml`），两个上游源合并
- 单文件，零依赖（仅需 Python 3.8+ 标准库）

## 快速开始

```bash
# 下载
curl -sSL https://raw.githubusercontent.com/waastudios/ysp-py/main/ysp-live.py -o ysp-live.py
curl -sSL https://raw.githubusercontent.com/waastudios/ysp-py/main/epg_agg.py -o epg_agg.py

# 运行（默认 8767 端口）
python3 ysp-live.py

# 自定义端口
python3 ysp-live.py --port 8080
```

播放器里填：
- 订阅：`http://<你的IP>:8767/cctv.m3u`
- 节目单：`http://<你的IP>:8767/epg.xml`

> `<你的IP>` 换成你服务器的公网 IP。`localhost` 只在服务器本机有效。

## 接口

| 地址 | 说明 |
|------|------|
| `/cctv.m3u` | 订阅（30 路） |
| `/epg.xml` | 聚合节目单（XMLTV） |
| `/diag` | 诊断信息 |
| `/health` | 健康检查 |
| `/<频道>.m3u8` | 频道直播流，如 `/cctv1.m3u8` |

## 频道分组

- **央视FHD** —— CCTV-1..17（1080p 高码率）
- **央视UHD** —— CCTV-4K、CTV-8K、CCTV-16 4K
- **CGTN** —— CGTN 英语/法语/俄语/阿语/西语/纪录

## 卸载

```bash
# 前台运行按 Ctrl+C 停止；后台运行则：
pkill -f ysp-live.py
rm -f ysp-live.py epg_agg.py
```
