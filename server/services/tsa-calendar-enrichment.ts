/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * TSA CALENDAR CONTEXT ENRICHMENT
 *
 * Repository: SigmaWolf-8/Ternary
 * Location:   server/services/tsa-calendar-enrichment.ts
 *
 * Resolves which calendar systems to embed in a TSA token by merging
 * policy-tier defaults with optional request-level supplements.
 * Calls the PlenumNET calendar service and returns a CalendarContext
 * for embedding as a non-critical TSTInfo extension.
 *
 * Best-effort: failure never blocks token issuance.
 */

export const CALENDAR_EXTENSION_OID = '1.3.6.1.4.1.0.100.2.1';

export const CALENDAR_SYSTEMS = {
  GREGORIAN:     'gregorian',
  HIJRI:         'hijri',
  SOLAR_HIJRI:   'solar-hijri',
  HEBREW:        'hebrew',
  JAPANESE:      'japanese',
  BUDDHIST:      'buddhist',
  CHINESE:       'chinese',
  INDIAN_SAKA:   'indian-saka',
  ETHIOPIAN:     'ethiopian',
  COPTIC:        'coptic',
  MAYAN:         'mayan',
  VEDIC:         'vedic',
  THIRTEEN_MOON: '13-moon',
  PERSIAN:       'persian',
  TIBETAN:       'tibetan',
  BENGALI:       'bengali',
  THAI_SOLAR:    'thai-solar',
  MINGUO:        'minguo',
  JUCHE:         'juche',
  BALINESE:      'balinese',
  BYZANTINE:     'byzantine',
  HOLOCENE:      'holocene',
  IGBO:          'igbo',
  AKAN:          'akan',
} as const;

export const KNOWN_CALENDAR_SYSTEMS = new Set(Object.values(CALENDAR_SYSTEMS));

const CALENDAR_KEY_MAP: Record<string, string> = {
  'hijri':        'islamic',
  'solar-hijri':  'persian',
  'hebrew':       'hebrew',
  'japanese':     'japaneseKoki',
  'buddhist':     'thaiBuddhist',
  'chinese':      'chineseSexagenary',
  'indian-saka':  'indianSaka',
  'ethiopian':    'ethiopian',
  'coptic':       'coptic',
  'mayan':        'mayanLongCount',
  'vedic':        'vedic',
  '13-moon':      'thirteenMoon',
  'persian':      'persian',
  'tibetan':      'tibetan',
  'bengali':      'bengali',
  'balinese':     'balinesePawukon',
  'byzantine':    'byzantine',
  'gregorian':    'gregorian',
};

export const POLICY_CALENDAR_CONFIG: Record<string, string[]> = {
  'DEFAULT':    [],
  'COMPLY':     [
    CALENDAR_SYSTEMS.HIJRI,
    CALENDAR_SYSTEMS.HEBREW,
    CALENDAR_SYSTEMS.JAPANESE,
    CALENDAR_SYSTEMS.BUDDHIST,
    CALENDAR_SYSTEMS.CHINESE,
    CALENDAR_SYSTEMS.INDIAN_SAKA,
    CALENDAR_SYSTEMS.SOLAR_HIJRI,
  ],
  'FORENSICS':  ['*'],
  'SENTINEL':   [],
  'SECURE':     [],
};

export interface CalendarDate {
  system: string;
  display: string;
  year: number | string;
  month: number | string;
  day: number | string;
  era?: string;
}

export interface CalendarContext {
  utcTimestamp: string;
  julianDayNumber: number;
  salviEpochDay: number;
  calendars: CalendarDate[];
  policyTier: string;
  source: {
    policy: string[];
    requested: string[];
  };
  extensionOid: string;
}

export interface CalendarServiceClient {
  convertDate(utcTimestamp: string): Promise<{
    julianDayNumber: number;
    salviEpochDay: number;
    calendars: Record<string, any>;
    allMappings: any[];
  }>;
}

export function resolveCalendarSystems(
  policyTier: string,
  requestCalendars?: string[],
): string[] {
  const policyDefaults = POLICY_CALENDAR_CONFIG[policyTier] || [];
  const requested = requestCalendars || [];

  if (policyDefaults[0] === '*' || requested.includes('*')) {
    return ['*'];
  }

  const merged = new Set<string>([...policyDefaults, ...requested]);

  const validated: string[] = [];
  for (const sys of merged) {
    if (KNOWN_CALENDAR_SYSTEMS.has(sys)) {
      validated.push(sys);
    } else {
      console.warn('Unknown calendar system requested (skipped)', {
        system: sys,
        policyTier,
        hint: `Valid systems: ${[...KNOWN_CALENDAR_SYSTEMS].join(', ')}`,
      });
    }
  }

  return validated;
}

export function classifyCalendarSources(
  policyTier: string,
  requestCalendars?: string[],
): { policy: string[]; requested: string[] } {
  const policyDefaults = POLICY_CALENDAR_CONFIG[policyTier] || [];
  const requested = requestCalendars || [];

  if (policyDefaults[0] === '*') {
    return { policy: ['*'], requested: [] };
  }
  if (requested.includes('*')) {
    return { policy: policyDefaults, requested: ['*'] };
  }

  const policySet = new Set(policyDefaults);
  const supplemented = requested.filter(
    sys => !policySet.has(sys) && KNOWN_CALENDAR_SYSTEMS.has(sys),
  );

  return {
    policy: policyDefaults,
    requested: supplemented,
  };
}

function mapCalendarEntry(mapping: any, systemName: string): CalendarDate {
  const display = mapping.salviEpochEquivalent || mapping.description || String(mapping.yearInCalendar);
  return {
    system: systemName,
    display,
    year: mapping.yearInCalendar ?? 0,
    month: mapping.month ?? 0,
    day: mapping.day ?? 0,
    ...(mapping.cyclicPosition ? { era: mapping.cyclicPosition } : {}),
  };
}

export async function enrichWithCalendars(
  utcTimestamp: string,
  policyTier: string,
  calendarClient: CalendarServiceClient,
  requestCalendars?: string[],
): Promise<CalendarContext | null> {
  const systems = resolveCalendarSystems(policyTier, requestCalendars);
  if (systems.length === 0) {
    return null;
  }

  try {
    const result = await calendarClient.convertDate(utcTimestamp);

    const serviceKeyToMapping = new Map<string, any>();
    for (const [serviceKey, calObj] of Object.entries(result.calendars)) {
      if (!calObj || typeof calObj !== 'object') continue;
      const formatted = (calObj as any).formatted;
      if (formatted) {
        const mapping = result.allMappings.find((m: any) =>
          m.salviEpochEquivalent === formatted || m.description?.includes(formatted)
        );
        if (mapping) {
          serviceKeyToMapping.set(serviceKey, mapping);
        }
      }
    }
    for (const m of result.allMappings) {
      for (const [serviceKey, calObj] of Object.entries(result.calendars)) {
        if (serviceKeyToMapping.has(serviceKey)) continue;
        if (!calObj || typeof calObj !== 'object') continue;
        const calYear = (calObj as any).year;
        if (calYear !== undefined && m.yearInCalendar === calYear) {
          serviceKeyToMapping.set(serviceKey, m);
          break;
        }
      }
    }

    let calendars: CalendarDate[];

    if (systems[0] === '*') {
      calendars = [];
      for (const [specName, serviceKey] of Object.entries(CALENDAR_KEY_MAP)) {
        const mapping = serviceKeyToMapping.get(serviceKey);
        if (mapping) {
          const calObj = result.calendars[serviceKey];
          const enriched = { ...mapping };
          if (calObj) {
            enriched.month = (calObj as any).month;
            enriched.day = (calObj as any).day;
          }
          calendars.push(mapCalendarEntry(enriched, specName));
        }
      }
      for (const m of result.allMappings) {
        const alreadyMapped = [...serviceKeyToMapping.values()].some(
          mapped => mapped.calendarSystem === m.calendarSystem
        );
        if (!alreadyMapped) {
          const sysName = m.calendarSystem.toLowerCase().replace(/\s+/g, '-');
          calendars.push(mapCalendarEntry(m, sysName));
        }
      }
    } else {
      calendars = [];
      for (const sys of systems) {
        const serviceKey = CALENDAR_KEY_MAP[sys];
        if (serviceKey) {
          const mapping = serviceKeyToMapping.get(serviceKey);
          if (mapping) {
            const calObj = result.calendars[serviceKey];
            const enriched = { ...mapping };
            if (calObj) {
              enriched.month = (calObj as any).month;
              enriched.day = (calObj as any).day;
            }
            calendars.push(mapCalendarEntry(enriched, sys));
          }
        } else {
          const mapping = result.allMappings.find((m: any) =>
            m.calendarSystem.toLowerCase().includes(sys.toLowerCase())
          );
          if (mapping) {
            calendars.push(mapCalendarEntry(mapping, sys));
          }
        }
      }
    }

    if (calendars.length === 0) {
      console.warn('Calendar service returned no matching systems', {
        requested: systems,
        available: Object.keys(result.calendars),
      });
      return null;
    }

    const source = classifyCalendarSources(policyTier, requestCalendars);

    const jdnMapping = result.allMappings.find((m: any) =>
      m.calendarSystem === 'Julian Day Number'
    );
    const julianDayNumber = jdnMapping?.daysSinceCalendarOrigin || 0;

    const epochDate = new Date('2025-04-01T00:00:00.000Z');
    const targetDate = new Date(utcTimestamp);
    const salviEpochDay = Math.floor((targetDate.getTime() - epochDate.getTime()) / 86_400_000);

    return {
      utcTimestamp,
      julianDayNumber,
      salviEpochDay,
      calendars,
      policyTier,
      source,
      extensionOid: CALENDAR_EXTENSION_OID,
    };

  } catch (error) {
    console.warn('Calendar enrichment failed (non-fatal)', {
      policyTier,
      utcTimestamp,
      requestedSystems: systems,
      error: (error as Error).message,
    });
    return null;
  }
}

export function serializeForExtension(context: CalendarContext): string {
  return JSON.stringify({
    v: 1,
    oid: context.extensionOid,
    utc: context.utcTimestamp,
    jdn: context.julianDayNumber,
    sed: context.salviEpochDay,
    tier: context.policyTier,
    src: context.source,
    cal: context.calendars.map(c => ({
      sys: c.system,
      d: c.display,
      y: c.year,
      m: c.month,
      day: c.day,
      ...(c.era ? { era: c.era } : {}),
    })),
  });
}
