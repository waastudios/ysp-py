//! EPG 聚合订阅模块 (ysp-web-rs 版)
//!
//! 从两个上游 EPG 源拉取节目单，只保留本项目实际在播的频道，
//! 合并去重后生成单一 XMLTV，供 /epg.xml 接口使用。
//!
//! 上游源:
//!   1. https://live.fanmingming.com/e.xml
//!   2. https://epg.112114.xyz/pp.xml.gz
//!
//! 本模块只服务 web 版（62 路），与 docker 版独立，不合并。

use std::{
    collections::{HashMap, HashSet},
    io::Read,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use quick_xml::{events::Event, Reader};
use tokio::sync::RwLock;
use tracing::warn;

/// 上游 EPG 源
const UPSTREAM_SOURCES: &[&str] = &[
    "https://live.fanmingming.com/e.xml",
    "https://epg.112114.xyz/pp.xml.gz",
];

/// 缓存刷新间隔：6 小时
const REFRESH_INTERVAL: Duration = Duration::from_secs(6 * 3600);

/// Rust 频道 slug -> EPG channel id 映射（62 路）
fn epg_id_for_slug(slug: &str) -> Option<&'static str> {
    Some(match slug {
        // 央视 FHD
        "cctv1" => "CCTV1",
        "cctv2" => "CCTV2",
        "cctv3" => "CCTV3",
        "cctv4" => "CCTV4",
        "cctv5" => "CCTV5",
        "cctv5plus" => "CCTV5+",
        "cctv6" => "CCTV6",
        "cctv7" => "CCTV7",
        "cctv8" => "CCTV8",
        "cctv9" => "CCTV9",
        "cctv10" => "CCTV10",
        "cctv11" => "CCTV11",
        "cctv12" => "CCTV12",
        "cctv13" => "CCTV13",
        "cctv14" => "CCTV14",
        "cctv15" => "CCTV15",
        "cctv16hd" => "CCTV16",
        "cctv164k" => "CCTV16",
        "cctv17" => "CCTV17",
        // 央视 UHD
        "cctv4k" => "CCTV4K",
        "cctv8k" => "CCTV-8K",
        // CGTN
        "cgtn" => "CGTN英语",
        "cgtnfayu" => "CGTN法语",
        "cgtneyu" => "CGTN俄语",
        "cgtnalaboyu" => "CGTN阿语",
        "cgtnxibanyayu" => "CGTN西语",
        "cgtnwaiyujilu" => "CGTN纪录",
        // 剧场
        "cctvfengyunjuchang" => "CCTV风云剧场",
        "cctvdiyijuchang" => "CCTV第一剧场",
        "cctvhuaijiujuchang" => "CCTV怀旧剧场",
        // 地方台（EPG id 即中文名）
        "beijingws" => "北京卫视",
        "jiangsuws" => "江苏卫视",
        "dongfangws" => "东方卫视",
        "zhejiangws" => "浙江卫视",
        "hunanws" => "湖南卫视",
        "hubeiws" => "湖北卫视",
        "guangdongws" => "广东卫视",
        "guangxiws" => "广西卫视",
        "heilongjiangws" => "黑龙江卫视",
        "hainanws" => "海南卫视",
        "chongqingws" => "重庆卫视",
        "shenzhenws" => "深圳卫视",
        "sichuanws" => "四川卫视",
        "henanws" => "河南卫视",
        "fujiandongnanws" => "东南卫视",
        "guizhouws" => "贵州卫视",
        "jiangxiws" => "江西卫视",
        "liaoningws" => "辽宁卫视",
        "anhuiws" => "安徽卫视",
        "hebeiws" => "河北卫视",
        "shandongws" => "山东卫视",
        "tianjinws" => "天津卫视",
        "jilinws" => "吉林卫视",
        "shannxiws" => "陕西卫视",
        "ningxiaws" => "宁夏卫视",
        "neimengguws" => "内蒙古卫视",
        "yunnanws" => "云南卫视",
        "shanxiws" => "山西卫视",
        "qinghaiws" => "青海卫视",
        "xizangws" => "西藏卫视",
        "xinjiangws" => "新疆卫视",
        // 其他
        "cetv1" => "CETV1",
        _ => return None,
    })
}

/// 从频道 slug 列表构建需要的 EPG id 集合
pub fn wanted_epg_ids(slugs: &[String]) -> HashSet<String> {
    slugs
        .iter()
        .filter_map(|s| epg_id_for_slug(s))
        .map(|s| s.to_string())
        .collect()
}

#[derive(Debug, Clone)]
struct Programme {
    channel: String,
    start: String,
    stop: String,
    title: String,
}

struct Parsed {
    channels: HashMap<String, String>, // id -> display-name
    programmes: Vec<Programme>,
}

/// 简单 XMLTV 解析：只提取 channel / programme
fn parse_xmltv(data: &[u8], wanted: &HashSet<String>) -> Result<Parsed> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().trim_text(true);

    let mut channels: HashMap<String, String> = HashMap::new();
    let mut programmes: Vec<Programme> = Vec::new();

    let mut buf = Vec::new();
    let mut cur_channel_id: Option<String> = None;
    let mut cur_display = String::new();
    let mut cur_pg: Option<Programme> = None;
    let mut in_title = false;
    let mut title_buf = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"channel" => {
                    cur_channel_id = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"id")
                        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()));
                    cur_display.clear();
                }
                b"display-name" => {
                    // display-name 文本在 Text 事件中处理
                }
                b"programme" => {
                    let mut channel = String::new();
                    let mut start = String::new();
                    let mut stop = String::new();
                    for a in e.attributes().filter_map(|a| a.ok()) {
                        let v = a.unescape_value().unwrap_or_default().into_owned();
                        match a.key.as_ref() {
                            b"channel" => channel = v,
                            b"start" => start = v,
                            b"stop" => stop = v,
                            _ => {}
                        }
                    }
                    if wanted.contains(&channel) {
                        cur_pg = Some(Programme {
                            channel,
                            start,
                            stop,
                            title: String::new(),
                        });
                        in_title = false;
                        title_buf.clear();
                    } else {
                        cur_pg = None;
                    }
                }
                b"title" => {
                    if cur_pg.is_some() {
                        in_title = true;
                        title_buf.clear();
                    }
                }
                _ => {}
            },
            Ok(Event::Text(e)) => {
                let txt = String::from_utf8_lossy(e.as_ref()).into_owned();
                if cur_pg.is_some() && in_title {
                    title_buf.push_str(&txt);
                } else if cur_channel_id.is_some() {
                    // 可能是 display-name 的文本（简化处理：channel 下的第一个文本）
                    if cur_display.is_empty() {
                        cur_display.push_str(&txt);
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"channel" => {
                    if let Some(cid) = cur_channel_id.take() {
                        if wanted.contains(&cid) && !channels.contains_key(&cid) {
                            channels.insert(
                                cid.clone(),
                                if cur_display.is_empty() {
                                    cid
                                } else {
                                    std::mem::take(&mut cur_display)
                                },
                            );
                        }
                    }
                    cur_display.clear();
                }
                b"title" => {
                    in_title = false;
                }
                b"programme" => {
                    if let Some(mut pg) = cur_pg.take() {
                        pg.title = std::mem::take(&mut title_buf);
                        programmes.push(pg);
                    }
                    in_title = false;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!("xmltv parse error: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(Parsed {
        channels,
        programmes,
    })
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// EPG 聚合器
#[derive(Clone)]
pub struct EpgAggregator {
    inner: Arc<RwLock<Inner>>,
}

struct Inner {
    wanted: HashSet<String>,
    xml: Option<String>,
    last_refresh: Option<Instant>,
    last_error: Option<String>,
    refreshing: bool,
    client: reqwest::Client,
}

impl EpgAggregator {
    pub fn new(wanted: HashSet<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent("Mozilla/5.0 (compatible; ysp-epg/1.0)")
            .build()
            .expect("build reqwest client");
        Self {
            inner: Arc::new(RwLock::new(Inner {
                wanted,
                xml: None,
                last_refresh: None,
                last_error: None,
                refreshing: false,
                client,
            })),
        }
    }

    /// 获取聚合 XML；缓存过期则触发后台刷新
    pub async fn get_xml(&self) -> Option<String> {
        let need_refresh = {
            let inner = self.inner.read().await;
            match (&inner.xml, inner.last_refresh) {
                (Some(_), Some(t)) => t.elapsed() > REFRESH_INTERVAL,
                _ => true,
            }
        };
        if need_refresh {
            self.trigger_refresh().await;
        }
        let inner = self.inner.read().await;
        inner.xml.clone()
    }

    pub async fn last_error(&self) -> Option<String> {
        self.inner.read().await.last_error.clone()
    }

    async fn trigger_refresh(&self) {
        {
            let mut inner = self.inner.write().await;
            if inner.refreshing {
                return;
            }
            inner.refreshing = true;
        }
        let this = self.clone();
        tokio::spawn(async move {
            let res = this.refresh_now().await;
            let mut inner = this.inner.write().await;
            inner.refreshing = false;
            if let Err(e) = res {
                inner.last_error = Some(format!("{:#}", e));
            }
        });
    }

    /// 同步刷新，返回是否成功
    pub async fn refresh_now(&self) -> Result<bool> {
        let xml = self.build().await?;
        let mut inner = self.inner.write().await;
        inner.xml = Some(xml);
        inner.last_refresh = Some(Instant::now());
        inner.last_error = None;
        Ok(true)
    }

    async fn fetch(&self, url: &str) -> Result<Vec<u8>> {
        let inner = self.inner.read().await;
        let client = inner.client.clone();
        let wanted_len = inner.wanted.len();
        drop(inner);
        let _ = wanted_len;

        let bytes = client
            .get(url)
            .send()
            .await
            .with_context(|| format!("fetch {}", url))?
            .bytes()
            .await
            .with_context(|| format!("read {}", url))?
            .to_vec();

        if url.ends_with(".gz") {
            let mut decoder = GzDecoder::new(&bytes[..]);
            let mut out = Vec::new();
            decoder
                .read_to_end(&mut out)
                .context("gunzip epg")?;
            Ok(out)
        } else {
            Ok(bytes)
        }
    }

    async fn build(&self) -> Result<String> {
        let wanted = { self.inner.read().await.wanted.clone() };

        let mut channels: HashMap<String, String> = HashMap::new();
        // (channel, start) -> programme，去重
        let mut seen: HashSet<(String, String)> = HashSet::new();
        let mut programmes: Vec<Programme> = Vec::new();

        for url in UPSTREAM_SOURCES {
            let data = match self.fetch(url).await {
                Ok(d) => d,
                Err(e) => {
                    warn!("epg fetch failed {}: {:#}", url, e);
                    continue;
                }
            };
            let parsed = match parse_xmltv(&data, &wanted) {
                Ok(p) => p,
                Err(e) => {
                    warn!("epg parse failed {}: {:#}", url, e);
                    continue;
                }
            };
            for (cid, name) in parsed.channels {
                channels.entry(cid).or_insert(name);
            }
            for pg in parsed.programmes {
                let key = (pg.channel.clone(), pg.start.clone());
                if seen.insert(key) {
                    programmes.push(pg);
                }
            }
        }

        // 生成 XMLTV
        let mut out = String::with_capacity(1 << 20);
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        out.push_str("<tv generator-info-name=\"ysp-web-rs-epg-aggregator\">\n");

        let mut cids: Vec<&String> = channels.keys().collect();
        cids.sort();
        for cid in cids {
            let name = &channels[cid];
            out.push_str(&format!(
                "  <channel id=\"{}\">\n    <display-name lang=\"zh\">{}</display-name>\n  </channel>\n",
                xml_escape(cid),
                xml_escape(name)
            ));
        }

        programmes.sort_by(|a, b| {
            a.channel
                .cmp(&b.channel)
                .then_with(|| a.start.cmp(&b.start))
        });
        for pg in &programmes {
            out.push_str(&format!(
                "  <programme channel=\"{}\" start=\"{}\" stop=\"{}\">\n    <title lang=\"zh\">{}</title>\n  </programme>\n",
                xml_escape(&pg.channel),
                xml_escape(&pg.start),
                xml_escape(&pg.stop),
                xml_escape(&pg.title)
            ));
        }

        out.push_str("</tv>\n");
        Ok(out)
    }
}
