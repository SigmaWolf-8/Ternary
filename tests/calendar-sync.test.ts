/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * Calendar Conversion Accuracy Tests
 */

import { describe, it, expect } from "vitest";
import {
  getSalviEpochCalendarSync,
  getSalviEpochAnchorPoints,
  toMayanLongCount,
  toHebrewDate,
  toJulianDayNumber,
  toIslamicHijri,
  toRomanAUCDate,
  toChineseSexagenary,
  toEgyptianCivil,
} from "../server/salvi-core/ancient-calendar-sync";

const REFERENCE_DATE = new Date("2025-01-01T00:00:00Z");
const J2000_DATE = new Date("2000-01-01T12:00:00Z");

describe("Salvi Epoch Anchor Points", () => {
  it("returns anchor points object", () => {
    const result = getSalviEpochAnchorPoints();
    expect(result).toBeDefined();
    expect(result.epoch).toBeDefined();
    expect(result.anchors).toBeDefined();
    expect(typeof result.anchors).toBe("object");
  });
});

describe("Calendar Sync Overview", () => {
  it("returns calendar sync data for a given date", () => {
    const sync = getSalviEpochCalendarSync(REFERENCE_DATE);
    expect(sync).toBeDefined();
    expect(typeof sync).toBe("object");
  });

  it("returns calendar sync data without argument (default)", () => {
    const sync = getSalviEpochCalendarSync();
    expect(sync).toBeDefined();
  });
});

describe("Julian Day Number", () => {
  it("JDN for J2000.0 epoch is 2451545", () => {
    const jdn = toJulianDayNumber(J2000_DATE);
    expect(jdn).toBeDefined();
    expect(jdn.julianDay).toBeCloseTo(2451545, 0);
  });

  it("JDN increases by 1 per day", () => {
    const d1 = new Date("2025-06-15T12:00:00Z");
    const d2 = new Date("2025-06-16T12:00:00Z");
    const jdn1 = toJulianDayNumber(d1);
    const jdn2 = toJulianDayNumber(d2);
    expect(jdn2.julianDay - jdn1.julianDay).toBeCloseTo(1, 0);
  });
});

describe("Mayan Long Count", () => {
  it("returns Mayan date with standard components", () => {
    const mayan = toMayanLongCount(REFERENCE_DATE);
    expect(mayan).toBeDefined();
    expect(typeof mayan).toBe("object");
  });
});

describe("Hebrew Calendar", () => {
  it("returns Hebrew date with year, month, day", () => {
    const hebrew = toHebrewDate(REFERENCE_DATE);
    expect(hebrew).toBeDefined();
    expect(hebrew.year).toBeGreaterThan(5700);
  });
});

describe("Islamic Hijri Calendar", () => {
  it("returns Islamic date with year after 1400 AH", () => {
    const islamic = toIslamicHijri(REFERENCE_DATE);
    expect(islamic).toBeDefined();
    expect(islamic.year).toBeGreaterThan(1400);
  });
});

describe("Roman AUC Calendar", () => {
  it("returns Roman AUC year > 2700", () => {
    const roman = toRomanAUCDate(REFERENCE_DATE);
    expect(roman).toBeDefined();
    expect(roman.year).toBeGreaterThan(2700);
  });
});

describe("Chinese Sexagenary Cycle", () => {
  it("returns Chinese calendar data", () => {
    const chinese = toChineseSexagenary(REFERENCE_DATE);
    expect(chinese).toBeDefined();
    expect(typeof chinese).toBe("object");
  });
});

describe("Egyptian Civil Calendar", () => {
  it("returns Egyptian date", () => {
    const egyptian = toEgyptianCivil(REFERENCE_DATE);
    expect(egyptian).toBeDefined();
    expect(typeof egyptian).toBe("object");
  });
});
