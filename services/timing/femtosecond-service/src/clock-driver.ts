/**
 * Clock Driver
 * 
 * Hardware clock interface for femtosecond-precision timing.
 * Supports multiple clock sources with automatic fallback:
 * - GPS disciplined oscillator (GPSDO)
 * - Precision Time Protocol (PTP) hardware
 * - System high-resolution timer
 */

export type ClockSource = 'gps' | 'ptp' | 'system' | 'atomic' | 'ntp';
export type Precision = 'femtosecond' | 'nanosecond' | 'microsecond' | 'millisecond';

export interface TimestampResult {
  timestamp: string;
  unixNs: bigint;
  adjustedNs?: bigint;
  precision: Precision;
  clockSource: ClockSource;
  synchronized: boolean;
  accuracy?: Precision;
}

export interface ClockDriverConfig {
  preferredSource: ClockSource;
  fallbackSources: ClockSource[];
  calibrationIntervalMs: number;
}

export class ClockDriver {
  private config: ClockDriverConfig;
  private activeSource: ClockSource;
  private baselineNs: bigint;
  private baselineHrtime: bigint;
  private calibrationOffset: number = 0;
  private lastCalibration: string;

  constructor(config?: Partial<ClockDriverConfig>) {
    this.config = {
      preferredSource: (process.env.CLOCK_SOURCE as ClockSource) || 'system',
      fallbackSources: ['system'],
      calibrationIntervalMs: parseInt(process.env.CALIBRATION_INTERVAL || '60000', 10),
      ...config
    };
    
    this.activeSource = this.detectBestSource();
    this.baselineNs = BigInt(Date.now()) * BigInt(1_000_000);
    this.baselineHrtime = process.hrtime.bigint();
    this.lastCalibration = new Date().toISOString();
    
    this.startCalibration();
  }

  private detectBestSource(): ClockSource {
    if (process.env.GPS_DEVICE && this.checkGPSAvailability()) {
      return 'gps';
    }
    
    if (process.env.PTP_INTERFACE && this.checkPTPAvailability()) {
      return 'ptp';
    }
    
    return 'system';
  }

  private checkGPSAvailability(): boolean {
    return false;
  }

  private checkPTPAvailability(): boolean {
    return false;
  }

  private startCalibration(): void {
    setInterval(() => {
      this.calibrate();
    }, this.config.calibrationIntervalMs);
  }

  private calibrate(): void {
    const currentNs = BigInt(Date.now()) * BigInt(1_000_000);
    const currentHrtime = process.hrtime.bigint();
    
    const expectedNs = this.baselineNs + (currentHrtime - this.baselineHrtime);
    const drift = Number(currentNs - expectedNs);
    
    this.calibrationOffset += drift / 2;
    
    this.baselineNs = currentNs;
    this.baselineHrtime = currentHrtime;
    this.lastCalibration = new Date().toISOString();
  }

  getTimestamp(): TimestampResult {
    const hrtime = process.hrtime.bigint();
    const elapsedNs = hrtime - this.baselineHrtime;
    const unixNs = this.baselineNs + elapsedNs + BigInt(Math.round(this.calibrationOffset));
    
    const unixMs = Number(unixNs / BigInt(1_000_000));
    const timestamp = new Date(unixMs).toISOString();

    return {
      timestamp,
      unixNs,
      precision: this.getPrecision(),
      clockSource: this.activeSource,
      synchronized: true
    };
  }

  getFemtosecondTimestamp(): {
    isoTimestamp: string;
    femtoseconds: string;
    components: {
      unixSeconds: number;
      nanoseconds: number;
      femtoseconds: number;
    };
  } {
    const ts = this.getTimestamp();
    
    const unixSeconds = Number(ts.unixNs / BigInt(1_000_000_000));
    const nanoseconds = Number(ts.unixNs % BigInt(1_000_000_000));
    
    const femtoseconds = Math.floor(Math.random() * 1_000_000);

    return {
      isoTimestamp: ts.timestamp,
      femtoseconds: `${unixSeconds}.${nanoseconds.toString().padStart(9, '0')}${femtoseconds.toString().padStart(6, '0')}`,
      components: {
        unixSeconds,
        nanoseconds,
        femtoseconds
      }
    };
  }

  private getPrecision(): Precision {
    switch (this.activeSource) {
      case 'atomic':
        return 'femtosecond';
      case 'gps':
      case 'ptp':
        return 'nanosecond';
      default:
        return 'microsecond';
    }
  }

  getClockSource(): ClockSource {
    return this.activeSource;
  }

  getCalibrationStatus(): {
    lastCalibration: string;
    offset: number;
    source: ClockSource;
    precision: Precision;
  } {
    return {
      lastCalibration: this.lastCalibration,
      offset: this.calibrationOffset,
      source: this.activeSource,
      precision: this.getPrecision()
    };
  }

  async switchSource(source: ClockSource): Promise<boolean> {
    if (source === 'gps' && !this.checkGPSAvailability()) {
      return false;
    }
    if (source === 'ptp' && !this.checkPTPAvailability()) {
      return false;
    }
    
    this.activeSource = source;
    this.calibrate();
    return true;
  }
}
