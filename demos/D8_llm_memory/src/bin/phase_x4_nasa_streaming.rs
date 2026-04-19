//! Phase X Step 5 — Claim 14 / 25 / 27-32 の realistic streaming validation on NASA HTTP log.
//!
//! ## 背景と目的
//!
//! F-069 / F-071 で Claim 5 / 14 / 20-32 が LoCoMo のような **static query task** で
//! selection benefit を生まないことが判明した。paper v0.2 は「真の use case は
//! streaming / 連続運用」と narrowing したが、その主張自体が未検証だった。
//!
//! 本 F-072 は NASA HTTP access log(1995-07-01、50k records 実データ、F-025 で
//! 既知 x2.3 static baseline)を **時系列 replay** し、streaming scenario で
//! 動的制御 component(Claim 14 exp decay + Claim 25 ActivationScore +
//! Claim 27-32 MetaController)が rare error(4xx/5xx)保持に **benefit を生むか**
//! を決定的に測定する。
//!
//! ## データ
//!
//! - File: [`benchmarks/real_data/data/nasa-http/access.log`](../../../benchmarks/real_data/data/nasa-http/access.log)
//! - 50,000 records、Common Log Format
//! - 時間範囲: 1995-07-01 00:00:01 〜 19:15:55(約 19 時間連続)
//! - Rare = HTTP status 400 / 401 / 403 / 404 / 500 / 502 / 503 / 504
//!
//! ## グラフ設計
//!
//! 二部グラフ:
//! - nodes = client IP ∪ resource URL
//! - edges = 1 record ごとに IP ↔ resource
//! - **rare ground truth = status が 4xx/5xx の resource ID 集合**
//! - 測定: 全 resource から top-30% を選択し、rare resource の recall を測定
//!
//! ## 実験構成 — streaming simulation
//!
//! 50,000 records を時系列順に並べ、**500 records / window** で 100 windows の
//! 時系列 replay を実施。各 window 到着時:
//!
//! 1. 該当 window の edge(IP ↔ resource)を graph に追加
//! 2. `DecayManager.tick()` で時刻を進める
//! 3. `DecayManager.apply_edge_decay()` で edge weight を exp(-λ·dt) 減衰
//! 4. `ActivationScore.record_event()` + `advance_tick()`(Claim 25)
//! 5. avg⟨k⟩ 計測 → `MetaController.step()` で α 更新(Claim 27-32)
//! 6. 最終 window まで完了後、top-30% resource を **5 条件で選択** し rare recall を比較
//!
//! ## 比較条件
//!
//! - **C0 Static one-shot**: 全 50k records を一括処理、KDF classify、top-30% 選択
//!   (F-025 再現の baseline — 静的 snapshot の reference)
//! - **C1 Streaming + Claim 14 decay**: window ごとに edge 追加 + decay、最終的に
//!   decayed edge weight を node rank signal として使用
//! - **C2 C1 + Claim 25 activation**: ActivationScore を temporal_score として加算
//! - **C3 C1 + Claim 27-32 meta α**: MetaController で α adapt(null control —
//!   classifier は α を使わないので C1 と同等のはず)
//! - **C4 Full streaming**: C1 + C2 + C3 の統合
//!
//! ## 測定
//!
//! - Final rare recall @ keep 30%(全 50k records 処理後の最終状態で)
//! - Trajectory: 各 window での partial rare recall(streaming 経過観測)
//! - α_edge trajectory(MetaController 適応の可視化)
//! - ActivationScore 分布(Claim 25 稼働確認)

use cgb_kdf::framework::multimodal::{select_top_k_multi_modal, MultiModalWeights};
use cgb_kdf::{
    ActivationScore, DecayManager, Layer, MasterSpecParams, MetaController, NodeClassifier,
};
use std::collections::{HashMap, HashSet};

// =============================================================================
// NASA log parser
// =============================================================================

#[derive(Debug, Clone)]
struct LogRecord {
    ip: String,
    resource: String,
    status: u16,
}

/// Common Log Format parser (no regex — manual split since format is fixed).
/// Format: `<ip> - - [<timestamp>] "<method> <resource> <proto>" <status> <size>`
fn parse_line(s: &str) -> Option<LogRecord> {
    let ip_end = s.find(" - - ")?;
    let ip = &s[..ip_end];
    let bracket_end = s.find(']')?;
    let rest = &s[bracket_end + 1..];
    let quote_start = rest.find('"')?;
    let quote_rest = &rest[quote_start + 1..];
    let quote_end = quote_rest.find('"')?;
    let request = &quote_rest[..quote_end];
    // request = "GET /path HTTP/1.0"
    let mut parts = request.split(' ');
    let _method = parts.next()?;
    let resource = parts.next()?;
    let after_quote = &quote_rest[quote_end + 1..];
    let trimmed = after_quote.trim_start();
    let status_str = trimmed.split(' ').next()?;
    let status: u16 = status_str.parse().ok()?;
    Some(LogRecord {
        ip: ip.to_string(),
        resource: resource.to_string(),
        status,
    })
}

fn load_nasa_records(path: &str) -> Vec<LogRecord> {
    let raw = std::fs::read_to_string(path).expect("read NASA access.log");
    raw.lines().filter_map(parse_line).collect()
}

// =============================================================================
// Graph builder (bipartite: IP ∪ Resource)
// =============================================================================

fn build_ids(
    records: &[LogRecord],
    rare_codes: &HashSet<u16>,
) -> (
    HashMap<String, u32>,           // entity → id
    Vec<(u32, u32)>,                 // (ip_id, res_id) per record (preserves order)
    HashSet<u32>,                    // rare resource IDs (ground truth)
    HashSet<u32>,                    // all resource IDs
) {
    let mut ids: HashMap<String, u32> = HashMap::new();
    let mut edges_ordered: Vec<(u32, u32)> = Vec::new();
    let mut rare_res: HashSet<u32> = HashSet::new();
    let mut res_ids: HashSet<u32> = HashSet::new();

    for r in records {
        let ip_key = format!("ip:{}", r.ip);
        let ip_id = match ids.get(&ip_key) {
            Some(&v) => v,
            None => {
                let nid = ids.len() as u32;
                ids.insert(ip_key, nid);
                nid
            }
        };
        let res_key = format!("res:{}", r.resource);
        let res_id = match ids.get(&res_key) {
            Some(&v) => v,
            None => {
                let nid = ids.len() as u32;
                ids.insert(res_key, nid);
                nid
            }
        };
        edges_ordered.push((ip_id, res_id));
        res_ids.insert(res_id);
        if rare_codes.contains(&r.status) {
            rare_res.insert(res_id);
        }
    }

    (ids, edges_ordered, rare_res, res_ids)
}

// =============================================================================
// Utilities
// =============================================================================

fn avg_degree_per_layer(
    edges: &[(u32, u32, f64)],
    layer_of: &HashMap<u32, Layer>,
) -> (f64, f64) {
    let mut deg: HashMap<u32, u32> = HashMap::new();
    for &(u, v, _) in edges {
        *deg.entry(u).or_insert(0) += 1;
        *deg.entry(v).or_insert(0) += 1;
    }
    let (mut sum_e, mut n_e) = (0.0_f64, 0_u32);
    let (mut sum_c, mut n_c) = (0.0_f64, 0_u32);
    for (&node, &layer) in layer_of {
        let d = deg.get(&node).copied().unwrap_or(0) as f64;
        match layer {
            Layer::Edge | Layer::Garbage => {
                sum_e += d;
                n_e += 1;
            }
            Layer::Core => {
                sum_c += d;
                n_c += 1;
            }
            Layer::Rare => {}
        }
    }
    let avg_e = if n_e > 0 { sum_e / n_e as f64 } else { 0.0 };
    let avg_c = if n_c > 0 { sum_c / n_c as f64 } else { 0.0 };
    (avg_e, avg_c)
}

fn resource_rare_recall(
    selected: &HashSet<u32>,
    rare_res: &HashSet<u32>,
) -> f64 {
    if rare_res.is_empty() {
        return 1.0;
    }
    let hit = selected.intersection(rare_res).count() as f64;
    hit / rare_res.len() as f64
}

/// Select top-K resource IDs (restricted to resources, not IPs) by a score function.
fn select_top_k_resources(
    scores: &[(u32, f64)],
    res_ids: &HashSet<u32>,
    keep: usize,
) -> HashSet<u32> {
    let mut filtered: Vec<(u32, f64)> = scores
        .iter()
        .copied()
        .filter(|(id, _)| res_ids.contains(id))
        .collect();
    filtered.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    filtered.into_iter().take(keep).map(|(id, _)| id).collect()
}

// =============================================================================
// Main streaming simulation
// =============================================================================

#[derive(Debug, Default, Clone)]
struct WindowMetric {
    n_edges: usize,
    n_nodes: usize,
    avg_k_edge: f64,
    avg_k_core: f64,
    alpha_edge: f64,
    rare_recall: f64,
}

fn run_condition_streaming(
    label: &str,
    edges_ordered: &[(u32, u32)],
    n_total_nodes: usize,
    res_ids: &HashSet<u32>,
    rare_res: &HashSet<u32>,
    window_size: usize,
    use_decay: bool,
    use_activation: bool,
    use_meta: bool,
) -> (f64, Vec<WindowMetric>) {
    let n_windows = edges_ordered.len() / window_size;
    let mut dm = DecayManager::master_spec();
    let mut act = ActivationScore::default();
    let meta = MetaController::default();
    let mut params = MasterSpecParams::default();

    let mut cur_edges: Vec<(u32, u32, f64)> = Vec::new();
    let mut trajectory: Vec<WindowMetric> = Vec::new();
    let mut alpha_edge_current = params.alpha_edge;

    // Trajectory sampling: record metrics every 10 windows to avoid log spam
    let trajectory_every = (n_windows / 20).max(1);

    for w in 0..n_windows {
        let start = w * window_size;
        let end = (start + window_size).min(edges_ordered.len());
        for &(u, v) in &edges_ordered[start..end] {
            cur_edges.push((u, v, 1.0));
            if use_activation {
                act.record_event(u);
                act.record_event(v);
            }
        }

        // Classify current graph
        let mut cls = NodeClassifier::default();
        let class = cls.classify(n_total_nodes, &cur_edges);
        let layer_of = class.layers.clone();

        // Apply decay (Claim 14)
        if use_decay {
            dm.initialize_with_edges(class.clone(), &cur_edges);
            dm.tick();
            dm.apply_edge_decay();
            // Reflect decayed edge weights back into cur_edges(for future decay cascade)
            for e in cur_edges.iter_mut() {
                if let Some(w_new) = dm.get_edge_weight(e.0, e.1) {
                    e.2 = w_new;
                }
            }
        }

        // Activation tick (Claim 25)
        if use_activation {
            act.advance_tick();
        }

        // MetaController (Claim 27-32)
        let (avg_k_e, avg_k_c) = avg_degree_per_layer(&cur_edges, &layer_of);
        if use_meta {
            let _ = meta.step(&mut params, avg_k_e, avg_k_c);
            alpha_edge_current = params.alpha_edge;
        }

        // Trajectory sampling
        if w % trajectory_every == 0 || w == n_windows - 1 {
            // Compute partial rare recall at this window
            let keep_partial = (res_ids.len() as f64 * 0.30).ceil() as usize;
            let scores_partial: Vec<(u32, f64)> = (0..n_total_nodes as u32)
                .map(|id| {
                    let base = match layer_of.get(&id).copied().unwrap_or(Layer::Edge) {
                        Layer::Rare => 1.0,
                        Layer::Core => 0.67,
                        Layer::Edge => 0.33,
                        Layer::Garbage => 0.0,
                    };
                    let act_boost = if use_activation {
                        let mx = act.levels.values().cloned().fold(1e-9, f64::max);
                        (act.get(id) / mx).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    (id, 0.7 * base + 0.3 * act_boost)
                })
                .collect();
            let sel_partial = select_top_k_resources(&scores_partial, res_ids, keep_partial);
            let rr = resource_rare_recall(&sel_partial, rare_res);
            trajectory.push(WindowMetric {
                n_edges: cur_edges.len(),
                n_nodes: n_total_nodes,
                avg_k_edge: avg_k_e,
                avg_k_core: avg_k_c,
                alpha_edge: alpha_edge_current,
                rare_recall: rr,
            });
        }
    }

    // Final selection using final state
    let mut final_cls = NodeClassifier::default();
    let final_class = final_cls.classify(n_total_nodes, &cur_edges);
    let layer_of = final_class.layers.clone();
    let keep = (res_ids.len() as f64 * 0.30).ceil() as usize;

    let scores: Vec<(u32, f64)> = (0..n_total_nodes as u32)
        .map(|id| {
            let base = match layer_of.get(&id).copied().unwrap_or(Layer::Edge) {
                Layer::Rare => 1.0,
                Layer::Core => 0.67,
                Layer::Edge => 0.33,
                Layer::Garbage => 0.0,
            };
            let act_boost = if use_activation {
                let mx = act.levels.values().cloned().fold(1e-9, f64::max);
                (act.get(id) / mx).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (id, 0.7 * base + 0.3 * act_boost)
        })
        .collect();
    let selected = select_top_k_resources(&scores, res_ids, keep);
    let recall = resource_rare_recall(&selected, rare_res);

    println!(
        "  {:25} final_recall={:.4} edges_processed={} n_windows={}",
        label, recall, cur_edges.len(), n_windows
    );
    (recall, trajectory)
}

fn c0_static_baseline(
    edges_all: &[(u32, u32, f64)],
    n_total_nodes: usize,
    res_ids: &HashSet<u32>,
    rare_res: &HashSet<u32>,
) -> f64 {
    let mut cls = NodeClassifier::default();
    let class = cls.classify(n_total_nodes, edges_all);
    let layer_of = class.layers;
    let keep = (res_ids.len() as f64 * 0.30).ceil() as usize;
    let scores: Vec<(u32, f64)> = (0..n_total_nodes as u32)
        .map(|id| {
            let base = match layer_of.get(&id).copied().unwrap_or(Layer::Edge) {
                Layer::Rare => 1.0,
                Layer::Core => 0.67,
                Layer::Edge => 0.33,
                Layer::Garbage => 0.0,
            };
            (id, base)
        })
        .collect();
    let selected = select_top_k_resources(&scores, res_ids, keep);
    resource_rare_recall(&selected, rare_res)
}

fn random_baseline(
    res_ids: &HashSet<u32>,
    rare_res: &HashSet<u32>,
    seed: u64,
) -> f64 {
    use rand::prelude::*;
    use rand::rngs::SmallRng;
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut res_vec: Vec<u32> = res_ids.iter().copied().collect();
    res_vec.shuffle(&mut rng);
    let keep = (res_ids.len() as f64 * 0.30).ceil() as usize;
    let selected: HashSet<u32> = res_vec.into_iter().take(keep).collect();
    resource_rare_recall(&selected, rare_res)
}

fn main() {
    let path = "benchmarks/real_data/data/nasa-http/access.log";
    println!("# Phase X Step 5 — NASA HTTP streaming validation(F-072 candidate)\n");

    let rare_codes: HashSet<u16> = [400, 401, 403, 404, 500, 502, 503, 504].into_iter().collect();
    println!("Loading NASA HTTP access log from {}", path);
    let records = load_nasa_records(path);
    println!("Parsed {} records", records.len());

    let (ids, edges_ordered, rare_res, res_ids) = build_ids(&records, &rare_codes);
    let n_total_nodes = ids.len();
    let n_edges_total = edges_ordered.len();
    println!();
    println!("## Data statistics");
    println!();
    println!("- 総 records(edges): {}", n_edges_total);
    println!("- total unique nodes (IPs + resources): {}", n_total_nodes);
    println!("- total resources: {}", res_ids.len());
    println!("- rare resources(4xx/5xx ground truth): {}", rare_res.len());
    println!("- rare ratio in resources: {:.2}%", rare_res.len() as f64 / res_ids.len() as f64 * 100.0);
    println!();

    // Static baselines
    let edges_all_f64: Vec<(u32, u32, f64)> = edges_ordered.iter().map(|&(u, v)| (u, v, 1.0)).collect();
    println!("## Static baselines\n");
    let c0 = c0_static_baseline(&edges_all_f64, n_total_nodes, &res_ids, &rare_res);
    let rnd = {
        let mut sum = 0.0_f64;
        for s in 0..5_u64 {
            sum += random_baseline(&res_ids, &rare_res, 1000 + s);
        }
        sum / 5.0
    };
    println!("  {:25} final_recall={:.4}", "Random (5-seed mean)", rnd);
    println!("  {:25} final_recall={:.4} (F-025 再現 reference)", "C0 Static KDF", c0);

    // Streaming conditions
    let window_size = 500_usize;
    let n_windows = n_edges_total / window_size;
    println!();
    println!("## Streaming conditions(window={} records, n_windows={})", window_size, n_windows);
    println!();

    let (c1, traj_c1) = run_condition_streaming(
        "C1 +Claim14 decay",
        &edges_ordered, n_total_nodes, &res_ids, &rare_res,
        window_size, true, false, false,
    );
    let (c2, traj_c2) = run_condition_streaming(
        "C2 C1+Claim25 act",
        &edges_ordered, n_total_nodes, &res_ids, &rare_res,
        window_size, true, true, false,
    );
    let (c3, _traj_c3) = run_condition_streaming(
        "C3 C1+Claim27-32 meta",
        &edges_ordered, n_total_nodes, &res_ids, &rare_res,
        window_size, true, false, true,
    );
    let (c4, traj_c4) = run_condition_streaming(
        "C4 Full streaming",
        &edges_ordered, n_total_nodes, &res_ids, &rare_res,
        window_size, true, true, true,
    );

    // Summary
    println!();
    println!("## 最終 rare-recall 比較 @ keep 30% of resources");
    println!();
    println!("| 条件 | final rare recall | Δ vs C0 Static | Δ vs Random |");
    println!("|---|---:|---:|---:|");
    println!("| Random (5-seed)      | {:.4} |  {:+.4} | — |", rnd, rnd - c0);
    println!("| **C0 Static KDF**    | **{:.4}** | — | {:+.4} |", c0, c0 - rnd);
    println!("| C1 +Claim14 decay    | {:.4} | {:+.4} | {:+.4} |", c1, c1 - c0, c1 - rnd);
    println!("| C2 C1+Claim25 act    | {:.4} | {:+.4} | {:+.4} |", c2, c2 - c0, c2 - rnd);
    println!("| C3 C1+Claim27-32 meta | {:.4} | {:+.4} | {:+.4} |", c3, c3 - c0, c3 - rnd);
    println!("| C4 Full streaming    | {:.4} | {:+.4} | {:+.4} |", c4, c4 - c0, c4 - rnd);

    // Trajectory for C4 + C1 (show streaming drift)
    println!();
    println!("## Trajectory(C1 decay-only vs C4 full、抜粋)");
    println!();
    println!("| window | n_edges | rare_recall C1 | rare_recall C4 | α_edge C4 |");
    println!("|---:|---:|---:|---:|---:|");
    for (i, (m1, m4)) in traj_c1.iter().zip(traj_c4.iter()).enumerate() {
        if i % 2 == 0 {
            println!(
                "| {} | {} | {:.4} | {:.4} | {:.3} |",
                i, m1.n_edges, m1.rare_recall, m4.rare_recall, m4.alpha_edge
            );
        }
    }

    // ActivationScore summary from C2 trajectory
    let traj_c2_last = traj_c2.last().cloned().unwrap_or_default();
    println!();
    println!("## C2 (+activation) の最終 window 計測");
    println!("- avg ⟨k⟩_edge: {:.3}", traj_c2_last.avg_k_edge);
    println!("- avg ⟨k⟩_core: {:.3}", traj_c2_last.avg_k_core);

    println!();
    println!("## 判定(自動)");
    println!();
    let best_streaming = c1.max(c2).max(c3).max(c4);
    if best_streaming > c0 + 0.005 {
        println!("- ✅ **streaming 動的制御が C0 static を +{:.4} 上回る** → Claim 14/25/27-32 の streaming scenario での value proposition が empirical に validated", best_streaming - c0);
    } else if (best_streaming - c0).abs() <= 0.005 {
        println!("- ⚠️ **streaming 動的制御は C0 static と同等**(Δ ≤ 0.005)→ NASA log の時系列特性では rare error がほぼ均等分布のため、decay / activation の benefit が出ない可能性");
    } else {
        println!("- ❌ **streaming 動的制御が C0 static を下回る(Δ={:.4})** → streaming-aware ranking が rare error を先に捨ててしまう、F-069/F-071 と同じ pattern の generalization", best_streaming - c0);
    }
    println!();
    println!("F-025 基準値(NASA-HTTP 実データ static KDF baseline): 0.237、Random 0.102");
    println!("本実験の C0 Static = {:.4}、Random(5-seed) = {:.4}(再現性確認)", c0, rnd);
}
