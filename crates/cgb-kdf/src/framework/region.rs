//! Hierarchical management regions (Claim 20-22)
//!
//! # Claim mapping
//!
//! | Claim | Requirement | Implementation |
//! |-------|-------------|----------------|
//! | 20 | Short-term (Region 1) + Long-term (Region 2) regions | [`RegionKind`] |
//! | 21 | Three regions (+ rare) with periods dt1:dt2:dt3 = 5:3:1 | [`RegionConfig`] |
//! | 22 | Different criteria/period/threshold/nonlinear-function params per region | [`RegionConfig::params`] |
//!
//! The canonical tick cadence is expressed with integer ticks to avoid
//! floating-point drift: Region 1 = 5 ticks/update, Region 2 = 3 ticks/update,
//! Region 3 = 1 tick/update. This yields the 5:3:1 ratio from Claim 21.

use super::decay::MasterSpecParams;

/// Management region kinds (Claim 20-21).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionKind {
    /// Region 1 — short-term management area (Claim 20: 短期管理用の第1管理領域)
    ShortTerm,
    /// Region 2 — long-term management area (Claim 20: 長期管理用の第2管理領域)
    LongTerm,
    /// Region 3 — rare protected management area (Claim 21)
    Rare,
}

impl RegionKind {
    /// Canonical integer tick period (Claim 21: dt1:dt2:dt3 = 5:3:1).
    #[inline]
    pub const fn tick_period(self) -> u64 {
        match self {
            Self::ShortTerm => 5,
            Self::LongTerm => 3,
            Self::Rare => 1,
        }
    }
}

/// Per-region configuration (Claim 22: 評価値の算出条件、算出周期、維持基準、
/// または前記非線形関数のパラメータの少なくとも一つを異ならせて適用).
#[derive(Clone, Debug)]
pub struct RegionConfig {
    pub kind: RegionKind,
    /// Master spec parameters for this region (may differ per Claim 22)
    pub params: MasterSpecParams,
    /// Maintenance threshold (評価値が前記維持基準)
    pub maintenance_threshold: f64,
    /// Current tick counter; when it hits `tick_period()` a region update fires.
    pub tick: u64,
}

impl RegionConfig {
    pub fn new(kind: RegionKind) -> Self {
        // Claim 22 allows param divergence; canonical defaults are identical to
        // MasterSpecParams, but thresholds differ per region.
        let maintenance_threshold = match kind {
            RegionKind::ShortTerm => 0.15,
            RegionKind::LongTerm => 0.05,
            RegionKind::Rare => 0.01,
        };
        Self {
            kind,
            params: MasterSpecParams::default(),
            maintenance_threshold,
            tick: 0,
        }
    }

    /// Advance the internal tick counter by 1 and return `true` when the
    /// region's period (Claim 21 dt1/dt2/dt3) elapses, at which point a new
    /// evaluation should be performed.
    pub fn tock(&mut self) -> bool {
        self.tick = self.tick.saturating_add(1);
        if self.tick >= self.kind.tick_period() {
            self.tick = 0;
            true
        } else {
            false
        }
    }
}

/// Hierarchical region manager holding all three regions (Claim 20-22).
#[derive(Clone, Debug)]
pub struct HierarchicalRegionManager {
    pub short_term: RegionConfig,
    pub long_term: RegionConfig,
    pub rare: RegionConfig,
    /// Number of global ticks advanced so far.
    pub global_ticks: u64,
}

impl Default for HierarchicalRegionManager {
    fn default() -> Self {
        Self {
            short_term: RegionConfig::new(RegionKind::ShortTerm),
            long_term: RegionConfig::new(RegionKind::LongTerm),
            rare: RegionConfig::new(RegionKind::Rare),
            global_ticks: 0,
        }
    }
}

impl HierarchicalRegionManager {
    /// Advance the global tick. Returns which regions should be re-evaluated
    /// on this tick, in order (short, long, rare).
    pub fn tick(&mut self) -> (bool, bool, bool) {
        self.global_ticks = self.global_ticks.saturating_add(1);
        let s = self.short_term.tock();
        let l = self.long_term.tock();
        let r = self.rare.tock();
        (s, l, r)
    }

    /// Ratio dt1:dt2:dt3 as computed from tick periods (must equal 5:3:1).
    pub fn period_ratio(&self) -> (u64, u64, u64) {
        (
            self.short_term.kind.tick_period(),
            self.long_term.kind.tick_period(),
            self.rare.kind.tick_period(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim21_dt_ratio_5_3_1() {
        let mgr = HierarchicalRegionManager::default();
        let (dt1, dt2, dt3) = mgr.period_ratio();
        assert_eq!(
            (dt1, dt2, dt3),
            (5, 3, 1),
            "Claim 21: dt1:dt2:dt3 must be 5:3:1"
        );
    }

    #[test]
    fn test_claim20_region_kinds_present() {
        let mgr = HierarchicalRegionManager::default();
        assert_eq!(mgr.short_term.kind, RegionKind::ShortTerm);
        assert_eq!(mgr.long_term.kind, RegionKind::LongTerm);
        assert_eq!(mgr.rare.kind, RegionKind::Rare);
    }

    #[test]
    fn test_claim22_region_threshold_differs() {
        // Claim 22: 維持基準 must be differentiable per region.
        let mgr = HierarchicalRegionManager::default();
        assert_ne!(
            mgr.short_term.maintenance_threshold,
            mgr.long_term.maintenance_threshold
        );
        assert_ne!(
            mgr.long_term.maintenance_threshold,
            mgr.rare.maintenance_threshold
        );
    }

    #[test]
    fn test_tick_cadence() {
        // Rare fires every tick (period 1), ShortTerm every 5, LongTerm every 3.
        let mut mgr = HierarchicalRegionManager::default();
        let mut fires = [(0u64, 0u64, 0u64); 15];
        for slot in fires.iter_mut() {
            let (s, l, r) = mgr.tick();
            *slot = (s as u64, l as u64, r as u64);
        }
        // Count fires in 15 ticks: short=3, long=5, rare=15
        let s_total: u64 = fires.iter().map(|t| t.0).sum();
        let l_total: u64 = fires.iter().map(|t| t.1).sum();
        let r_total: u64 = fires.iter().map(|t| t.2).sum();
        assert_eq!(s_total, 3, "short-term fires every 5 ticks → 3 fires in 15");
        assert_eq!(l_total, 5, "long-term fires every 3 ticks → 5 fires in 15");
        assert_eq!(r_total, 15, "rare fires every tick → 15 fires in 15");
    }
}
