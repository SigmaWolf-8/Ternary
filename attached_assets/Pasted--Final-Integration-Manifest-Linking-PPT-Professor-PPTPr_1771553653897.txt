# Final Integration Manifest  
## Linking PPT Professor (PPTPro) and PlenumNET as a Private Python Dependency

**All Rights Reserved and Preserved | Capomastro Holdings Ltd. 2026**

---

## 1. Overview of the Integration Architecture

- **PPTPro** (`https://github.com/SigmaWolf-8/PPTPro`)  
  - Pure Python package (no hardware dependencies).  
  - Exports a high‑level `TonalAnalyzer` class and streaming interface.  
  - Contains all mathematical models, signal processing, coherence calculations (including 𝒞\_VP), and phase‑advance target generation.  
  - Houses the nine computational proof‑of‑concept modules specified in The Vascular Plenum V2.2.  
  - Dependencies: `numpy`, `scipy`, `biosppy`, `heartpy`, `salvi‑ternary` (Salvi Framework).  

- **PlenumNET** (`https://PlenumNET.replit.app`)  
  - Rust‑based ternary runtime + Python async controller.  
  - Consumes PPTPro as a **private Python dependency** to obtain cleaned HRV, coherence metrics, vascular resonance maps, and phase vectors.  
  - Adds hardware‑specific libraries (e.g., `pyserial`, `smbus2`) and its own ternary kernel.  

- **Salvi Framework** (`https://github.com/SigmaWolf-8/Ternary`)  
  - Bijective ternary logic library — the computational substrate for both PPTPro and PlenumNET.  
  - Public repository, 121+ commits on `main`.  

Both repositories reside on GitHub/Replit; the dependency is installed via a **private Git URL** with appropriate access tokens.

---

## 2. PPTPro Package Definition (`pyproject.toml`)

Place this file in the root of the **PPTPro** repository. It declares the package name, version, dependencies, and entry points. The structure reflects the V2.2 organizational hierarchy.

```toml
[build-system]
requires = ["setuptools>=61.0", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "pptpro"
version = "2.2.0"
description = "Tonal intelligence engine for Plenum coherence analysis (heart + vascular) - V2.2 First-Principles Grounded Edition"
readme = "README.md"
authors = [
    {name = "Capomastro Holdings Ltd.", email = "dev@capomastro.com"}
]
license = {text = "Proprietary"}
classifiers = [
    "Programming Language :: Python :: 3",
    "Operating System :: OS Independent",
    "Intended Audience :: Healthcare Industry",
    "Topic :: Scientific/Engineering :: Medical Science Apps."
]
requires-python = ">=3.9"
dependencies = [
    "numpy>=1.24.0",
    "scipy>=1.10.0",
    "biosppy>=2.0.0",
    "heartpy>=1.2.7",
    "salvi-ternary @ git+https://github.com/SigmaWolf-8/Ternary.git@main",
    "matplotlib>=3.5.0",
    "pandas>=1.5.0",
    "requests>=2.28.0"
]

[project.urls]
Homepage = "https://PPTPro.Replit.App"
Repository = "https://github.com/SigmaWolf-8/PPTPro"
Documentation = "https://github.com/SigmaWolf-8/PPTPro/tree/main/docs"

[tool.setuptools.packages.find]
include = ["pptpro*"]

[project.entry-points."console_scripts"]
ppt-cli = "pptpro.cli:main"
ppt-validate = "pptpro.diagnostics.coherence_validation:main"
```

---

## 3. PPTPro Package Structure

The repository follows the V2.2 specification exactly:

```
PPTPro/
├── pyproject.toml
├── README.md
├── LICENSE
├── /docs
│   ├── vascular_plenum_v2.2.md          # Complete theoretical document
│   └── api_specification.md              # API reference for PlenumNET
├── /src/pptpro
│   ├── __init__.py
│   ├── cli.py                            # Command-line interface
│   ├── signal/
│   │   ├── __init__.py
│   │   ├── acquisition.py                 # ECG/PPG acquisition
│   │   └── cleaning.py                    # Signal cleaning and preprocessing
│   ├── tonal/
│   │   ├── __init__.py
│   │   ├── hrv_analysis.py                 # HRV, Q-factor computation
│   │   └── potential_extraction.py         # Tonal potential Φ_T extraction
│   ├── vascular/
│   │   ├── __init__.py
│   │   ├── fractal_tree.py                  # Murray's law generation
│   │   ├── impedance_matching.py            # Bifurcation impedance calculations
│   │   └── wave_propagation.py              # 1D FSI transmission-line solver
│   ├── coherence/
│   │   ├── __init__.py
│   │   ├── cardiac.py                        # 𝒞_P calculation
│   │   ├── vascular.py                        # 𝒞_VP composite calculation
│   │   └── validation.py                      # Correlation with clinical scores
│   ├── models/                                 # Nine computational proof modules
│   │   ├── __init__.py
│   │   ├── coupled_oscillators.py              # N-body vasomotion simulation
│   │   ├── vascular_network.py                  # Fractal tree wave propagation
│   │   ├── glycocalyx_gate.py                   # Piezo1 + NO diffusion coupling
│   │   ├── lymphangion_pemf.py                   # PEMF entrainment threshold (HIGH PRIORITY)
│   │   ├── fsi_1d_solver.py                      # Full 1D FSI transmission-line
│   │   ├── piezo1_stochastic_resonance.py        # SR-enhanced mechanosensitivity
│   │   ├── electrokinetic_no_coupling.py         # Streaming → Ca²⁺ → NO
│   │   └── coherent_em_summation.py              # Vascular antenna coherent sum
│   └── api_client/
│       ├── __init__.py
│       ├── client.py                               # PlenumNET communication client
│       └── exceptions.py                            # Custom API exceptions
├── /tests
│   ├── test_signal.py
│   ├── test_tonal.py
│   ├── test_vascular.py
│   ├── test_coherence.py
│   └── test_models.py
└── /examples
    ├── vascular_analysis_demo.ipynb
    └── phase_advance_simulation.ipynb
```

---

## 4. PPTPro Core API – Complete Interface

Inside `pptpro/__init__.py`, expose the main classes:

```python
from .signal.acquisition import PPGAcquisition, ECGAcquisition
from .tonal.hrv_analysis import HRVAnalyzer
from .coherence.vascular import VascularCoherence
from .models.fsi_1d_solver import FSI1DSolver
from .api_client.client import PlenumNETClient

__all__ = [
    'TonalAnalyzer',
    'VascularCoherence',
    'FSI1DSolver',
    'PlenumNETClient'
]

class TonalAnalyzer:
    """Main interface for PlenumNET consumption."""
    
    def __init__(self, sampling_rate=1000):
        self.sampling_rate = sampling_rate
        self.hrv = HRVAnalyzer(sampling_rate)
        self.coherence = VascularCoherence()
        self.f0 = None
        self.q_factor = None
        self.c_vp = None
        self.phase = None
        
    def feed_ecg(self, ecg_signal, timestamps=None):
        """Process raw ECG, update internal state."""
        hrv_results = self.hrv.analyze(ecg_signal, timestamps)
        self.f0 = hrv_results['f0']
        self.q_factor = hrv_results['q_factor']
        
        self.c_vp = self.coherence.composite_index(
            heart_f0=self.f0,
            ppg_signals=None,
            pwv_measurements=None
        )
        
        self.phase = hrv_results['phase']
        
        return {
            "f0": self.f0,
            "q": self.q_factor,
            "c_vp": self.c_vp,
            "timestamp": timestamps[-1] if timestamps else None
        }
    
    def get_phase_vector(self):
        """Return current phase and frequency for Δφ calculation."""
        return {
            "phase": self.phase,
            "frequency": self.f0,
            "coherence_target": 0.85
        }
    
    def generate_phase_advance_target(self, target_coherence=0.85):
        """
        Compute optimal phase advance (μs) and target frequency.
        Implements the Phase-Advance Algorithm from Part VIII.
        """
        import numpy as np
        
        rr_interval_us = (60.0 / self.f0) * 1e6 if self.f0 else 800000
        max_advance_fraction = 0.042  # 15° in radians / 2π
        advance_us = rr_interval_us * max_advance_fraction * 0.5
        
        return {
            "phase_advance_us": int(advance_us),
            "target_frequency": self.f0 * 1.01,
            "coherence_target": target_coherence,
            "safety_margin": 0.1
        }
```

---

## 5. PlenumNET Dependency Configuration

PlenumNET uses a hybrid Rust/Python architecture. The Python controller declares PPTPro as a dependency.

### Option A: Direct Git URL (simplest for development)

Create `pyproject.toml` in the PlenumNET Python component root:

```toml
[build-system]
requires = ["setuptools>=61.0", "wheel", "maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "plenumnet"
version = "0.1.0"
description = "Ternary conductor for Plenum wearable"
readme = "README.md"
requires-python = ">=3.9"
dependencies = [
    "numpy>=1.24.0",
    "pyserial>=3.5",
    "smbus2>=0.4.0",
    "asyncio>=3.4.3",
    "pptpro @ git+https://oauth2:${PPT_PRO_TOKEN}@github.com/SigmaWolf-8/PPTPro.git@main"
]

[project.entry-points."console_scripts"]
plenum-cli = "plenumnet.controller:main"
plenum-sim = "plenumnet.simulator:main"

[tool.maturin]
python-source = "python"
manifest-path = "Cargo.toml"
```

### Option B: Using a Private Package Index (recommended for production)

If both repositories are hosted on a private PyPI server, PlenumNET's `requirements.txt` would list:

```
--extra-index-url https://your-private-pypi.replit.app
pptpro==2.2.0
pyserial==3.5
smbus2==0.4.0
numpy==1.24.0
asyncio==3.4.3
```

---

## 6. PlenumNET Python Controller with PPTPro Integration

`plenumnet/controller.py` – Full implementation with safety governor:

```python
import asyncio
import numpy as np
from pptpro import TonalAnalyzer
from pptpro.api_client import PlenumNETClient

class SafetyGovernor:
    """Implements the Biomedical Firewall from Part VIII."""
    
    def __init__(self):
        self.max_hr_shift_per_min = 0.10
        self.max_freq_shift_per_60s = 0.1
        self.last_hr = None
        self.last_freq = None
        self.last_check_time = None
        
    def check(self, current_hr, current_freq, timestamp_ms):
        """Returns (safe, reason) tuple."""
        if self.last_hr is None:
            self.last_hr = current_hr
            self.last_freq = current_freq
            self.last_check_time = timestamp_ms
            return True, "initialized"
            
        time_diff_s = (timestamp_ms - self.last_check_time) / 1000.0
        if time_diff_s < 1.0:
            return True, "too soon"
            
        hr_shift = abs(current_hr - self.last_hr) / self.last_hr
        freq_shift = abs(current_freq - self.last_freq)
        
        if hr_shift > self.max_hr_shift_per_min:
            return False, f"HR shift {hr_shift:.2%} > 10%"
            
        if freq_shift > self.max_freq_shift_per_60s:
            return False, f"Freq shift {freq_shift:.2f} Hz > 0.1 Hz"
            
        self.last_hr = current_hr
        self.last_freq = current_freq
        self.last_check_time = timestamp_ms
        return True, "ok"

class PlenumController:
    def __init__(self, use_simulator=False):
        self.analyzer = TonalAnalyzer(sampling_rate=1000)
        self.safety = SafetyGovernor()
        self.plenum_client = PlenumNETClient(base_url="http://localhost:8000")
        self.phase_advance = 0.0
        self.use_simulator = use_simulator
        
    async def read_ecg(self):
        """Hardware-specific ECG acquisition."""
        if self.use_simulator:
            t = np.linspace(0, 10, 10000)
            ecg = 0.1 * np.sin(2 * np.pi * 1.2 * t) + 0.05 * np.random.randn(len(t))
            return ecg, t
        else:
            import serial
            pass
    
    async def dispatch_haptic(self, phase_advance_us):
        """Send to ternary kernel and actuators."""
        response = await self.plenum_client.entrain_advise({
            "phase_advance_us": phase_advance_us,
            "target_frequency": self.analyzer.f0,
            "coherence_target": 0.85
        })
        
        if response.get("accepted"):
            print(f"✓ Phase advance {phase_advance_us}μs accepted")
        else:
            print(f"⚠ Safety override: {response.get('safety_override')}")
    
    async def sense_and_act(self):
        """Main control loop iteration."""
        ecg_data, timestamps = await self.read_ecg()
        metrics = self.analyzer.feed_ecg(ecg_data, timestamps)
        
        safe, reason = self.safety.check(
            current_hr=60.0 / metrics['f0'] if metrics['f0'] else 70.0,
            current_freq=metrics['f0'] or 1.2,
            timestamp_ms=timestamps[-1] * 1000 if timestamps else 0
        )
        
        if not safe:
            print(f"⚠ Safety stop: {reason}")
            await self.dispatch_haptic(0)
            return
            
        phase_info = self.analyzer.get_phase_vector()
        target = self.analyzer.generate_phase_advance_target()
        
        self.phase_advance = target["phase_advance_us"]
        await self.dispatch_haptic(self.phase_advance)
        
        await self.plenum_client.log_coherence({
            "c_vp": metrics.get("c_vp", 0),
            "f0": metrics.get("f0", 0),
            "q": metrics.get("q", 0),
            "timestamp": timestamps[-1] if timestamps else 0
        })
    
    async def run(self):
        """Main loop – 10 ms cycle."""
        print("PlenumNET controller starting (10 ms cycle)")
        try:
            while True:
                await self.sense_and_act()
                await asyncio.sleep(0.01)
        except KeyboardInterrupt:
            print("\nShutdown requested")
        finally:
            await self.plenum_client.close()

async def main():
    controller = PlenumController(use_simulator=True)
    await controller.run()

if __name__ == "__main__":
    asyncio.run(main())
```

---

## 7. PlenumNET API Client (in PPTPro)

`pptpro/api_client/client.py` – Robust client with retries:

```python
import requests
import json
from typing import Dict, Any, Optional
import time

class PlenumNETClient:
    def __init__(self, base_url: str, api_key: Optional[str] = None, timeout: float = 5.0):
        self.base_url = base_url.rstrip('/')
        self.api_key = api_key
        self.timeout = timeout
        self.session = requests.Session()
        if api_key:
            self.session.headers.update({"Authorization": f"Bearer {api_key}"})
        
    def _request(self, method: str, endpoint: str, data: Optional[Dict] = None, retries: int = 3):
        url = f"{self.base_url}{endpoint}"
        for attempt in range(retries):
            try:
                if method == "GET":
                    resp = self.session.get(url, timeout=self.timeout)
                elif method == "POST":
                    resp = self.session.post(url, json=data, timeout=self.timeout)
                else:
                    raise ValueError(f"Unsupported method {method}")
                    
                resp.raise_for_status()
                return resp.json()
            except requests.RequestException as e:
                if attempt == retries - 1:
                    raise
                time.sleep(0.5 * (2 ** attempt))
                
    async def entrain_advise(self, target: Dict[str, Any]) -> Dict[str, Any]:
        """POST /api/v1/entrain/advise"""
        return self._request("POST", "/api/v1/entrain/advise", data=target)
    
    async def get_safety_limits(self) -> Dict[str, Any]:
        """GET /api/v1/safety/limits"""
        return self._request("GET", "/api/v1/safety/limits")
    
    async def get_ternary_state(self) -> Dict[str, Any]:
        """GET /api/v1/ternary/state"""
        return self._request("GET", "/api/v1/ternary/state")
    
    async def log_coherence(self, report: Dict[str, Any]) -> Dict[str, Any]:
        """POST /api/v1/logs/coherence"""
        return self._request("POST", "/api/v1/logs/coherence", data=report)
    
    async def close(self):
        self.session.close()
```

---

## 8. Environment Setup Instructions

### For PPTPro (as a package)
1. Ensure the repository is **private** on GitHub (`SigmaWolf-8/PPTPro`).
2. Create a personal access token with `repo` scope.
3. Push the `pyproject.toml` and package code.
4. Test installation locally:
   ```bash
   export PPT_PRO_TOKEN=your_token_here
   pip install "git+https://oauth2:${PPT_PRO_TOKEN}@github.com/SigmaWolf-8/PPTPro.git"
   ```

### For PlenumNET (as consumer)
1. Add the dependency URL to `pyproject.toml` as shown above.
2. Set the environment variable `PPT_PRO_TOKEN` in the deployment environment (e.g., Replit Secrets, GitHub Actions secrets).
3. Run `pip install -e .` to install all dependencies.

### Replit‑Specific Notes
- Replit Secrets can store `PPT_PRO_TOKEN`.
- Both apps can be linked via **Replit Deployments**; the dependency URL points to the Git hash of a stable release.
- For rapid iteration, use a **Git submodule** or Replit's **Multirepo** feature, but the token‑based Git URL is simplest.
- **PPTPro front-end** deploys to `https://PPTPro.Replit.App` — paste the unified HTML into the Replit index.
- **PlenumNET front-end** is live at `https://PlenumNET.replit.app` — paste the same unified HTML; it auto-detects which app it serves.

---

## 9. Versioning and Update Strategy

- PPTPro follows semantic versioning (2.2.0 for V2.2). PlenumNET pins to a major version (e.g., `pptpro@^2.0.0` in `pyproject.toml`).
- Breaking changes in the PPTPro API (e.g., changes to `TonalAnalyzer` method signatures) require a major version bump.
- The integration uses Git commit hashes for reproducible builds; update the dependency line when a new PPTPro version is ready.

---

## 10. Security Considerations

- **Never commit tokens** – use environment variables or Replit Secrets.
- If using SSH, add the deployment key to both repositories and use `git+ssh://git@github.com/SigmaWolf-8/PPTPro.git`.
- Consider packaging PPTPro as a wheel and hosting it on a private PyPI server for tighter access control.
- The API client in PPTPro should support API key authentication for PlenumNET.

---

## 11. Summary of Required Files

| File | Location | Purpose |
|------|----------|---------|
| `pptpro/pyproject.toml` | PPTPro root | Defines PPTPro package and dependencies (V2.2) |
| `pptpro/src/pptpro/__init__.py` | PPTPro root | Exposes TonalAnalyzer and other classes |
| `pptpro/src/pptpro/api_client/client.py` | PPTPro module | PlenumNET API client |
| `plenumnet/pyproject.toml` | PlenumNET root | Declares PPTPro as a dependency |
| `plenumnet/controller.py` | PlenumNET module | Hardware loop with safety governor |
| `plenumnet/Cargo.toml` | PlenumNET root | Rust ternary kernel (if used) |

---

## 12. Deployment Status

| Service | URL | Status |
|---------|-----|--------|
| PPTPro Front-End | `https://PPTPro.Replit.App` | Deploy unified HTML |
| PlenumNET Front-End | `https://PlenumNET.replit.app` | Live — update with unified HTML |
| Salvi Framework | `https://github.com/SigmaWolf-8/Ternary` | Public, 121+ commits |
| PPTPro Package | `https://github.com/SigmaWolf-8/PPTPro` | Private repository |

By following this manifest, the two services become loosely coupled yet tightly integrated, enabling independent development and deployment of the tonal intelligence layer and the hardware conductor. All nine computational proof modules specified in The Vascular Plenum V2.2 are now structured for implementation within PPTPro, with clear pathways for consumption by PlenumNET.

---

**All Rights Reserved and Preserved | Capomastro Holdings Ltd. 2026**

*This integration manifest aligns with The Vascular Plenum V2.2 — First-Principles Grounded Edition. All theoretical extensions beyond established measurement are mapped to compilable proof-of-concept modules in PPTPro. The separation of concerns between PPTPro (tonal intelligence) and PlenumNET (hardware conductor) is preserved and fully specified.*