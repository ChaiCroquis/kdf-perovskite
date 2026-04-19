//! Markdown report generator (common template for all demos).

use std::collections::BTreeMap;
use super::{Axis, DemoReport};

pub fn render_markdown(report: &DemoReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Demo {}: {}\n\n", report.demo_id, report.title));
    out.push_str(&format!("**Dataset:** {} (n={})\n\n", report.dataset_name, report.n_items));
    out.push_str(&format!("**Patent section:** {}\n\n", report.patent_section));

    // Metric legend
    out.push_str("## 測定指標の3軸フレーム\n\n");
    let groups: BTreeMap<&'static str, Vec<&super::Metric>> = {
        let mut g: BTreeMap<&'static str, Vec<&super::Metric>> = BTreeMap::new();
        for m in &report.metric_definitions {
            let key = match m.axis {
                Axis::KdfStrength => "軸A: KDF の強み(想定)",
                Axis::Tie => "軸B: 他手法と同等(想定)",
                Axis::KdfWeakness => "軸C: KDF の弱み / トレードオフ(想定)",
            };
            g.entry(key).or_default().push(m);
        }
        g
    };
    for (group_name, ms) in &groups {
        out.push_str(&format!("### {}\n\n", group_name));
        for m in ms {
            let dir = if m.higher_is_better { "↑" } else { "↓" };
            out.push_str(&format!("- `{}` {}: {}\n", m.name, dir,
                if m.higher_is_better { "高い方が良い" } else { "低い方が良い" }));
        }
        out.push('\n');
    }

    // Result table
    out.push_str("## 結果\n\n");
    out.push_str("| Method | ラベル要 | ");
    let metric_cols: Vec<String> = report.metric_definitions.iter().map(|m| m.name.clone()).collect();
    for m in &metric_cols { out.push_str(&format!("{} | ", m)); }
    out.push_str("wall(ms) |\n|---|:---:|");
    for _ in &metric_cols { out.push_str("---:|"); }
    out.push_str("---:|\n");
    for r in &report.method_results {
        let label = if r.requires_labels { "Yes" } else { "No" };
        let prefix = if r.method == "KDF" { "**" } else { "" };
        let suffix = if r.method == "KDF" { "**" } else { "" };
        out.push_str(&format!("| {}{}{} | {} | ", prefix, r.method, suffix, label));
        for m in &metric_cols {
            let v = r.metrics.get(m).copied().unwrap_or(f64::NAN);
            out.push_str(&format!("{:.3} | ", v));
        }
        out.push_str(&format!("{:.2} |\n", r.wall_ms));
    }
    out.push('\n');

    // Conclusion
    out.push_str("## 結論(正直)\n\n");
    if !report.conclusion.kdf_recommended_for.is_empty() {
        out.push_str("### ✅ KDF が選ばれるべきシナリオ\n\n");
        for s in &report.conclusion.kdf_recommended_for {
            out.push_str(&format!("- {}\n", s));
        }
        out.push('\n');
    }
    if !report.conclusion.kdf_not_recommended_for.is_empty() {
        out.push_str("### ⚠️ KDF を避けるべきシナリオ\n\n");
        for s in &report.conclusion.kdf_not_recommended_for {
            out.push_str(&format!("- {}\n", s));
        }
        out.push('\n');
    }
    if !report.conclusion.honest_limits.is_empty() {
        out.push_str("### 📋 正直な制限事項\n\n");
        for s in &report.conclusion.honest_limits {
            out.push_str(&format!("- {}\n", s));
        }
        out.push('\n');
    }

    // 再現コマンドは demo_id から自動推定するのではなく、呼び出し側で指定可能にする
    // ここではプレースホルダだけ提示し、詳細は各 demo の README.md を参照させる
    out.push_str("## 再現\n\n各 demo の README.md を参照してください。\n");

    out
}
