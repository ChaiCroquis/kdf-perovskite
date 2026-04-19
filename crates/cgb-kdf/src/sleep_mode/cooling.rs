//! Adaptive cooling scheduler for simulated annealing

/// Adaptive Cooling Scheduler for Simulated Annealing
pub struct AdaptiveCoolingScheduler {
    /// Initial temperature
    pub initial_temperature: f64,
    /// Final temperature (convergence)
    pub final_temperature: f64,
    /// Normal cooling rate
    pub normal_cooling_rate: f64,
    /// Slow cooling rate (near phase transition)
    pub slow_cooling_rate: f64,
    /// Variance threshold for phase transition detection
    pub variance_threshold: f64,
    /// Window size for variance computation
    pub window_size: usize,
    /// Current temperature
    temperature: f64,
    /// Energy history
    energy_history: Vec<f64>,
    /// Step counter
    step: u64,
}

impl AdaptiveCoolingScheduler {
    /// Create a new scheduler
    pub fn new(
        initial_temperature: f64,
        final_temperature: f64,
        normal_cooling_rate: f64,
        slow_cooling_rate: f64,
        variance_threshold: f64,
        window_size: usize,
    ) -> Self {
        Self {
            initial_temperature,
            final_temperature,
            normal_cooling_rate,
            slow_cooling_rate,
            variance_threshold,
            window_size,
            temperature: initial_temperature,
            energy_history: Vec::new(),
            step: 0,
        }
    }

    /// Update temperature based on current energy
    pub fn update(&mut self, current_energy: f64) -> f64 {
        self.energy_history.push(current_energy);
        self.step += 1;

        // Compute variance in window
        let window: Vec<f64> = if self.energy_history.len() >= self.window_size {
            self.energy_history[self.energy_history.len() - self.window_size..].to_vec()
        } else {
            self.energy_history.clone()
        };

        let variance = if window.len() > 1 {
            let mean: f64 = window.iter().sum::<f64>() / window.len() as f64;
            window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / window.len() as f64
        } else {
            0.0
        };

        // Choose cooling rate
        let cooling_rate = if variance > self.variance_threshold {
            self.slow_cooling_rate // Near phase transition
        } else {
            self.normal_cooling_rate
        };

        self.temperature *= cooling_rate;
        self.temperature
    }

    /// Check if converged
    pub fn is_converged(&self) -> bool {
        self.temperature <= self.final_temperature
    }

    /// Reset scheduler
    pub fn reset(&mut self) {
        self.temperature = self.initial_temperature;
        self.energy_history.clear();
        self.step = 0;
    }

    /// Get current temperature
    pub fn get_temperature(&self) -> f64 {
        self.temperature
    }
}

impl Default for AdaptiveCoolingScheduler {
    fn default() -> Self {
        Self::new(1.0, 0.001, 0.99, 0.9999, 0.1, 100)
    }
}
