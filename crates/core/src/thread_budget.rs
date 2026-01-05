//! Thread budget management for multi-session mining.
//!
//! Default behavior: WARN_ONLY - no automatic thread changes.
//! User can enable AUTO_DISTRIBUTE or ENFORCE_LIMIT if desired.

use serde::{Deserialize, Serialize};

/// CPU budget management mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetMode {
    /// No management - ignore thread counts entirely
    Off,
    /// Detect overcommit and warn, but don't change threads [DEFAULT]
    #[default]
    WarnOnly,
    /// Suggest thread split for new sessions (don't modify existing)
    AutoDistribute,
    /// Cap total threads to budget (may reduce per-session) - advanced
    EnforceLimit,
}

/// Budget preset for thread calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BudgetPreset {
    /// ~50% of cores
    Eco,
    /// ~80% of cores [DEFAULT]
    #[default]
    Balanced,
    /// 100% of cores
    Max,
}

impl BudgetPreset {
    pub fn factor(&self) -> f32 {
        match self {
            Self::Eco => 0.5,
            Self::Balanced => 0.8,
            Self::Max => 1.0,
        }
    }
}

/// Thread budget settings (persisted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadBudgetSettings {
    pub mode: BudgetMode,
    pub preset: BudgetPreset,
    pub max_concurrent_sessions: u32,
}

impl Default for ThreadBudgetSettings {
    fn default() -> Self {
        Self {
            mode: BudgetMode::WarnOnly,
            preset: BudgetPreset::Balanced,
            max_concurrent_sessions: 3,
        }
    }
}

/// Budget calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub effective_cores: u32,
    pub budget_threads: u32,
    pub total_requested: u32,
    pub is_overcommitted: bool,
    pub overcommit_ratio: f32,
    pub suggested_per_session: u32,
}

/// Calculate thread budget based on system info
pub fn calculate_budget(
    settings: &ThreadBudgetSettings,
    active_session_count: u32,
    total_threads_requested: u32,
) -> BudgetStatus {
    let effective_cores = num_cpus::get() as u32;
    let budget_threads = (effective_cores as f32 * settings.preset.factor()).floor() as u32;
    
    let is_overcommitted = total_threads_requested > budget_threads;
    let overcommit_ratio = if budget_threads > 0 {
        total_threads_requested as f32 / budget_threads as f32
    } else {
        0.0
    };
    
    let session_count = active_session_count.max(1);
    let suggested_per_session = budget_threads / session_count;
    
    BudgetStatus {
        effective_cores,
        budget_threads,
        total_requested: total_threads_requested,
        is_overcommitted,
        overcommit_ratio,
        suggested_per_session: suggested_per_session.max(1),
    }
}

/// Get suggested threads for a new session
pub fn suggest_threads_for_new_session(
    settings: &ThreadBudgetSettings,
    current_active_sessions: u32,
) -> u32 {
    let effective_cores = num_cpus::get() as u32;
    let budget = (effective_cores as f32 * settings.preset.factor()).floor() as u32;
    let future_sessions = current_active_sessions + 1;
    (budget / future_sessions).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_calculation() {
        let settings = ThreadBudgetSettings::default();
        let status = calculate_budget(&settings, 2, 8);
        
        assert!(status.effective_cores > 0);
        assert!(status.budget_threads > 0);
        assert!(status.suggested_per_session > 0);
    }

    #[test]
    fn test_overcommit_detection() {
        let settings = ThreadBudgetSettings {
            mode: BudgetMode::WarnOnly,
            preset: BudgetPreset::Eco, // 50%
            max_concurrent_sessions: 3,
        };
        
        let cores = num_cpus::get() as u32;
        let budget = (cores as f32 * 0.5).floor() as u32;
        
        // Request more than budget
        let status = calculate_budget(&settings, 2, budget + 4);
        assert!(status.is_overcommitted);
        assert!(status.overcommit_ratio > 1.0);
    }

    #[test]
    fn test_budget_mode_default() {
        let mode = BudgetMode::default();
        assert_eq!(mode, BudgetMode::WarnOnly);
    }

    #[test]
    fn test_suggest_threads() {
        let settings = ThreadBudgetSettings::default();
        let suggested = suggest_threads_for_new_session(&settings, 0);
        assert!(suggested > 0);
        
        let suggested_with_existing = suggest_threads_for_new_session(&settings, 2);
        assert!(suggested_with_existing <= suggested);
    }
}
