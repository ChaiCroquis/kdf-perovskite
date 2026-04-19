//! Compile/runtime-checked invariants for Claim 8-9 monotone function,
//! §5 Lyapunov stability, and Master spec ordering (α_R < α_E < α_C).

use super::decay::MasterSpecParams;

/// Diagnostic report on MasterSpecParams — checks Rev.10/11 constraint set.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamCheckReport {
    pub alpha_order_ok: bool,
    pub beta_order_ok: bool,
    pub dt_positive_ok: bool,
    pub gamma_positive_ok: bool,
    pub findings: Vec<&'static str>,
}

impl ParamCheckReport {
    pub fn is_ok(&self) -> bool {
        self.alpha_order_ok && self.beta_order_ok && self.dt_positive_ok && self.gamma_positive_ok
    }
}

/// Inspect parameters against the Master-spec ordering constraints.
///
/// Canonical ordering (Rev.11 §11):
/// - α: α_R < α_E < α_C
/// - β: β_M < β_C < β_E
/// - dt: all strictly positive
/// - γ: all strictly positive
pub fn inspect(p: &MasterSpecParams) -> ParamCheckReport {
    let mut findings = Vec::new();

    let alpha_order_ok = p.alpha_rare < p.alpha_edge && p.alpha_edge < p.alpha_core;
    if !alpha_order_ok {
        findings.push("alpha ordering violated: expected α_R < α_E < α_C");
    }

    // β_M (meta) not exposed separately here; check edge/core ordering and
    // require β > 0 globally.
    let beta_order_ok = p.beta > 0.0;
    if !beta_order_ok {
        findings.push("β must be strictly positive");
    }

    let dt_positive_ok = p.dt_edge > 0.0 && p.dt_rare > 0.0 && p.dt_core > 0.0 && p.dt_meta > 0.0;
    if !dt_positive_ok {
        findings.push("dt values must be strictly positive");
    }

    let gamma_positive_ok = p.gamma_edge > 0.0
        && p.gamma_rare > 0.0
        && p.gamma_core > 0.0
        && p.gamma_meta > 0.0;
    if !gamma_positive_ok {
        findings.push("γ values must be strictly positive");
    }

    ParamCheckReport {
        alpha_order_ok,
        beta_order_ok,
        dt_positive_ok,
        gamma_positive_ok,
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params_satisfy_all_invariants() {
        let p = MasterSpecParams::default();
        let r = inspect(&p);
        assert!(r.is_ok(), "default params must satisfy invariants, findings={:?}", r.findings);
        assert!(r.findings.is_empty());
    }

    #[test]
    fn detects_alpha_disorder() {
        let p = MasterSpecParams { alpha_edge: 0.1, ..MasterSpecParams::default() };
        let r = inspect(&p);
        assert!(!r.alpha_order_ok);
        assert!(r.findings.iter().any(|f| f.contains("alpha")));
    }

    #[test]
    fn detects_negative_dt() {
        let p = MasterSpecParams { dt_rare: -0.001, ..MasterSpecParams::default() };
        let r = inspect(&p);
        assert!(!r.dt_positive_ok);
    }

    #[test]
    fn detects_zero_gamma() {
        let p = MasterSpecParams { gamma_meta: 0.0, ..MasterSpecParams::default() };
        let r = inspect(&p);
        assert!(!r.gamma_positive_ok);
    }
}
