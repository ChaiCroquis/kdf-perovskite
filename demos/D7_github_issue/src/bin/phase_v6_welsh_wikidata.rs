//! Phase V6 — P4 Welsh + Wikidata pilot.
//!
//! 目的: Welsh Wikipedia からの random article に対し、Wikidata QID 経由
//! で「他言語版が存在するか」を判定し、Welsh-only article(= 少数言語固有概念)
//! を minority ground truth として定義する。
//!
//! 現段階では data acquisition + minority 比率の測定まで。
//! Graph 構築と KDF 適用は次 phase (V6-b) 候補。

use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Deserialize, Debug)]
struct RandomList {
    query: QueryRandom,
}

#[derive(Deserialize, Debug)]
struct QueryRandom {
    random: Vec<PageRef>,
}

#[derive(Deserialize, Debug)]
struct PageRef {
    #[allow(dead_code)]
    id: u64,
    title: String,
}

#[derive(Deserialize, Debug)]
struct PagePropsResp {
    query: PagePropsQuery,
}

#[derive(Deserialize, Debug)]
struct PagePropsQuery {
    pages: HashMap<String, PageDetails>,
}

#[derive(Deserialize, Debug)]
struct PageDetails {
    #[serde(default)]
    pageprops: Option<PageProps>,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PageProps {
    #[serde(default)]
    wikibase_item: Option<String>,
}

fn http_get(url: &str) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(20))
        .user_agent("KDF-research-pilot/0.1")
        .build();
    agent
        .get(url)
        .call()
        .map_err(|e| format!("HTTP error: {}", e))?
        .into_string()
        .map_err(|e| format!("read error: {}", e))
}

fn get_welsh_random_titles(n: usize) -> Result<Vec<String>, String> {
    let url = format!(
        "https://cy.wikipedia.org/w/api.php?action=query&list=random&rnnamespace=0&rnlimit={}&format=json",
        n.min(50)
    );
    let body = http_get(&url)?;
    let resp: RandomList = serde_json::from_str(&body).map_err(|e| format!("JSON parse: {}", e))?;
    Ok(resp.query.random.into_iter().map(|p| p.title).collect())
}

fn get_wikidata_qids_for_welsh(titles: &[String]) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    for chunk in titles.chunks(20) {
        let titles_param = chunk
            .iter()
            .map(|t| t.replace(' ', "_"))
            .collect::<Vec<_>>()
            .join("|");
        let url_encoded = urlencoding::encode(&titles_param).into_owned();
        let url = format!(
            "https://cy.wikipedia.org/w/api.php?action=query&titles={}&prop=pageprops&ppprop=wikibase_item&format=json",
            url_encoded
        );
        let body = http_get(&url)?;
        let resp: PagePropsResp =
            serde_json::from_str(&body).map_err(|e| format!("JSON parse: {}", e))?;
        for (_, details) in resp.query.pages {
            if let (Some(title), Some(props)) = (details.title, details.pageprops)
                && let Some(qid) = props.wikibase_item
            {
                map.insert(title, qid);
            }
        }
    }
    Ok(map)
}

#[derive(Deserialize, Debug)]
struct WikidataResp {
    entities: HashMap<String, WikidataEntity>,
}

#[derive(Deserialize, Debug)]
struct WikidataEntity {
    #[serde(default)]
    sitelinks: HashMap<String, serde_json::Value>,
}

fn check_enwiki_sitelinks(qids: &[String]) -> Result<HashMap<String, bool>, String> {
    // Return QID → has_enwiki_sitelink
    let mut result = HashMap::new();
    for chunk in qids.chunks(40) {
        let ids_param = chunk.join("|");
        let url = format!(
            "https://www.wikidata.org/w/api.php?action=wbgetentities&ids={}&props=sitelinks&sitefilter=enwiki&format=json",
            ids_param
        );
        let body = http_get(&url)?;
        let resp: WikidataResp =
            serde_json::from_str(&body).map_err(|e| format!("wd JSON: {}", e))?;
        for (qid, entity) in resp.entities {
            let has_en = entity.sitelinks.contains_key("enwiki");
            result.insert(qid, has_en);
        }
    }
    Ok(result)
}

fn main() {
    println!("# Phase V6 — P4 Welsh + Wikidata pilot\n");

    println!("Step 1: 50 random Welsh Wikipedia articles を取得...");
    let titles = match get_welsh_random_titles(50) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("titles 取得失敗: {}", e);
            std::process::exit(1);
        }
    };
    println!("  取得成功: {} articles", titles.len());
    if titles.len() >= 5 {
        println!("  例: {}, {}, {}", titles[0], titles[1], titles[2]);
    }

    println!("\nStep 2: 各 article の Wikidata QID を取得...");
    let qid_map = match get_wikidata_qids_for_welsh(&titles) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("QID 取得失敗: {}", e);
            std::process::exit(1);
        }
    };
    println!(
        "  {} / {} articles に QID あり",
        qid_map.len(),
        titles.len()
    );

    let qids: Vec<String> = qid_map.values().cloned().collect();

    println!("\nStep 3: 各 QID の enwiki sitelink 有無を確認...");
    let enwiki_map = match check_enwiki_sitelinks(&qids) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("sitelink 確認失敗: {}", e);
            std::process::exit(1);
        }
    };

    let welsh_only: Vec<&String> = enwiki_map
        .iter()
        .filter(|&(_, &has_en)| !has_en)
        .map(|(qid, _)| qid)
        .collect();
    let cross_lang: Vec<&String> = enwiki_map
        .iter()
        .filter(|&(_, &has_en)| has_en)
        .map(|(qid, _)| qid)
        .collect();

    println!("\n## 結果\n");
    println!("| カテゴリ | 件数 | 比率 |");
    println!("|---|---:|---:|");
    println!("| 全 articles (titles 取得) | {} | 100% |", titles.len());
    println!(
        "| QID 取得成功 | {} | {:.1}% |",
        qid_map.len(),
        100.0 * qid_map.len() as f64 / titles.len() as f64
    );
    println!(
        "| **Welsh-only (minority ground truth)** | **{}** | **{:.1}%** |",
        welsh_only.len(),
        100.0 * welsh_only.len() as f64 / enwiki_map.len().max(1) as f64
    );
    println!(
        "| enwiki 等との cross-lingual | {} | {:.1}% |",
        cross_lang.len(),
        100.0 * cross_lang.len() as f64 / enwiki_map.len().max(1) as f64
    );

    println!("\n## 解釈\n");
    if welsh_only.len() >= 5 {
        println!(
            "✅ **Pilot 成功**: Welsh-only article を {} 件特定可能。これにより、",
            welsh_only.len()
        );
        println!("  - Minority ground truth の**データ取得は実現可能**");
        println!("  - 次 step: category graph の構築 + KDF による rare preservation 測定");
        println!("  - 推定 P4 完全検証所要時間: 追加 1-2 時間の Rust 実装");
    } else if !welsh_only.is_empty() {
        println!(
            "⚠️ **Pilot 部分成功**: Welsh-only article が {} 件のみ。",
            welsh_only.len()
        );
        println!("  - Sample size を 200+ に拡大すれば意味ある比率になる可能性");
        println!(
            "  - あるいは Welsh 特有の cultural/geographic concept を狙って sample する方が効率的"
        );
    } else {
        println!("❌ **Pilot 結果がゼロ**: 今回の random 50 は全て cross-lingual だった。");
        println!("  - Welsh-only は確率的には 5-15% と推定されるため、50 の sample では不安定");
        println!("  - より大きな sample (500+) か targeted selection が必要");
    }

    println!("\n## 現状記録\n");
    println!("**P4 Welsh/Wikidata pilot は technical feasibility を確認した**:");
    println!("- Welsh Wikipedia API アクセス: ✅");
    println!("- Wikidata QID 取得: ✅");
    println!("- enwiki sitelink 判定: ✅");
    println!("- Minority ground truth 定義(Welsh-only = enwiki link 無し): ✅");
    println!("- Full P4 experiment(category graph + KDF): **本 pilot の範囲外**, future work");
}
