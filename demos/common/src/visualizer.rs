//! Thin adaptor: the Rust side only writes JSON. The Python script reads it.

use std::path::Path;

use super::DemoReport;

/// Write JSON + stub Markdown report. The Python visualizer will produce SVGs.
pub fn emit_artifacts(report: &DemoReport, out_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    let json = out_dir.join("report.json");
    report.write_json(&json)?;
    let md = out_dir.join("report.md");
    std::fs::write(&md, super::report::render_markdown(report))?;
    Ok(())
}
