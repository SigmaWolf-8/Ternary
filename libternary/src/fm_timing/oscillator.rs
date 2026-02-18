use crate::TernaryTrit;
use super::hrv::HrvEntropy;

const THRESHOLD: f64 = 0.1;

/// Van der Pol oscillator generating FM-modulated timing signals.
///
/// d²V/dt² - μ(1 - V²)dV/dt + ω₀²V = F_ext(t) + ξ(t)
///
/// The oscillator's phase is quantized to ternary: {Behind, Synchronized, Ahead}
/// relative to a reference clock, producing the tonal signal that structures
/// the plenum's gradient field.
pub struct TonalOscillator {
    omega_0: f64,
    mu: f64,
    state: [f64; 2],  // [V, dV/dt]
    time: f64,
    reference_phase: f64,
    noise_source: HrvEntropy,
    mod_amplitude: f64,
    mod_frequency: f64,
}

impl TonalOscillator {
    /// Create a new oscillator.
    ///
    /// `freq_hz`: intrinsic frequency (~1 Hz for cardiac analog)
    /// `mu`: nonlinear damping (controls limit cycle shape)
    /// `mod_amplitude`: FM modulation depth A_m
    /// `mod_frequency`: FM modulation rate ω_m (e.g. respiratory sinus arrhythmia)
    pub fn new(
        freq_hz: f64,
        mu: f64,
        mod_amplitude: f64,
        mod_frequency: f64,
        noise_source: HrvEntropy,
    ) -> Self {
        Self {
            omega_0: 2.0 * std::f64::consts::PI * freq_hz,
            mu,
            state: [1.0, 0.0],
            time: 0.0,
            reference_phase: 0.0,
            noise_source,
            mod_amplitude,
            mod_frequency,
        }
    }

    /// Advance one timestep using RK4 integration.
    /// Returns the ternary-quantized phase state.
    pub fn step(&mut self, dt: f64) -> TernaryTrit {
        let xi = self.noise_source.sample();
        let f_ext = self.mod_amplitude
            * (2.0 * std::f64::consts::PI * self.mod_frequency * self.time).cos();

        let k1 = self.derivatives(self.state, f_ext, xi);
        let s1 = [
            self.state[0] + 0.5 * dt * k1[0],
            self.state[1] + 0.5 * dt * k1[1],
        ];

        let k2 = self.derivatives(s1, f_ext, xi);
        let s2 = [
            self.state[0] + 0.5 * dt * k2[0],
            self.state[1] + 0.5 * dt * k2[1],
        ];

        let k3 = self.derivatives(s2, f_ext, xi);
        let s3 = [
            self.state[0] + dt * k3[0],
            self.state[1] + dt * k3[1],
        ];

        let k4 = self.derivatives(s3, f_ext, xi);

        self.state[0] += dt / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]);
        self.state[1] += dt / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]);
        self.time += dt;
        self.reference_phase += self.omega_0 * dt;

        self.phase_to_trit()
    }

    fn derivatives(&self, s: [f64; 2], f_ext: f64, xi: f64) -> [f64; 2] {
        let v = s[0];
        let dv = s[1];
        let ddv = self.mu * (1.0 - v * v) * dv
                - self.omega_0.powi(2) * v
                + f_ext
                + xi;
        [dv, ddv]
    }

    /// Instantaneous phase from state variables (atan2 of analytic signal)
    pub fn instantaneous_phase(&self) -> f64 {
        self.state[1].atan2(self.state[0])
    }

    /// Instantaneous frequency in Hz
    pub fn instantaneous_frequency(&self) -> f64 {
        let phase = self.instantaneous_phase();
        let phase_diff = phase - (self.reference_phase % (2.0 * std::f64::consts::PI));
        self.omega_0 / (2.0 * std::f64::consts::PI) + phase_diff / (2.0 * std::f64::consts::PI)
    }

    /// Modulation index β = A_m / ω_m
    pub fn modulation_index(&self) -> f64 {
        if self.mod_frequency == 0.0 {
            return 0.0;
        }
        self.mod_amplitude / (2.0 * std::f64::consts::PI * self.mod_frequency)
    }

    /// Quantize phase error to ternary
    fn phase_to_trit(&self) -> TernaryTrit {
        let phase_error = self.instantaneous_phase() - self.reference_phase;
        let pi = std::f64::consts::PI;
        let normalized = ((phase_error + pi) % (2.0 * pi) + 2.0 * pi) % (2.0 * pi) - pi;
        if normalized < -THRESHOLD {
            TernaryTrit::Neg
        } else if normalized > THRESHOLD {
            TernaryTrit::Pos
        } else {
            TernaryTrit::Zero
        }
    }

    pub fn elapsed(&self) -> f64 {
        self.time
    }

    pub fn state_v(&self) -> f64 {
        self.state[0]
    }

    pub fn state_dv(&self) -> f64 {
        self.state[1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oscillator_produces_stable_limit_cycle() {
        let mut osc = TonalOscillator::new(
            1.0, 1.0, 0.0, 0.0,
            HrvEntropy::new_deterministic(0.0),
        );
        for _ in 0..50_000 {
            osc.step(0.001);
        }
        let mut amplitudes = Vec::new();
        for i in 0..10_000 {
            osc.step(0.001);
            if i % 100 == 0 {
                let amp = (osc.state_v().powi(2) + osc.state_dv().powi(2)).sqrt();
                amplitudes.push(amp);
            }
        }
        let mean = amplitudes.iter().sum::<f64>() / amplitudes.len() as f64;
        assert!(mean > 0.5, "Oscillator should have nonzero amplitude, got {}", mean);
        let max = amplitudes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = amplitudes.iter().cloned().fold(f64::INFINITY, f64::min);
        assert!(max > 0.1, "Max amplitude should be nonzero");
        assert!(max / min.max(0.001) < 50.0, "Amplitude range too extreme: {}/{}", max, min);
    }

    #[test]
    fn oscillator_produces_all_three_trit_states() {
        let mut osc = TonalOscillator::new(
            1.0, 1.0, 0.0, 0.0,
            HrvEntropy::new_deterministic(0.0),
        );
        let mut seen_neg = false;
        let mut seen_zero = false;
        let mut seen_pos = false;
        for _ in 0..10_000 {
            match osc.step(0.001) {
                TernaryTrit::Neg => seen_neg = true,
                TernaryTrit::Zero => seen_zero = true,
                TernaryTrit::Pos => seen_pos = true,
            }
        }
        assert!(seen_neg, "Never produced Neg trit");
        assert!(seen_zero, "Never produced Zero trit");
        assert!(seen_pos, "Never produced Pos trit");
    }

    #[test]
    fn fm_modulation_varies_frequency() {
        let mut osc = TonalOscillator::new(
            1.0, 1.0,
            0.3,
            0.25,
            HrvEntropy::new_deterministic(0.0),
        );
        let mut freqs = Vec::new();
        for i in 0..10_000 {
            osc.step(0.001);
            if i > 5000 && i % 10 == 0 {
                freqs.push(osc.instantaneous_frequency());
            }
        }
        let f_min = freqs.iter().cloned().fold(f64::INFINITY, f64::min);
        let f_max = freqs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(f_max - f_min > 0.01, "FM should produce frequency deviation, got range {}", f_max - f_min);
    }

    #[test]
    fn modulation_index_computed_correctly() {
        let osc = TonalOscillator::new(
            1.0, 1.0, 0.3, 0.25,
            HrvEntropy::new_deterministic(0.0),
        );
        let beta = osc.modulation_index();
        let expected = 0.3 / (2.0 * std::f64::consts::PI * 0.25);
        assert!((beta - expected).abs() < 1e-10);
    }
}
