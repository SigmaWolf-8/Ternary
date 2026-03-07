/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Patent(s) Pending — All Rights Reserved
 * Applied Physics Division
 *
 * TSA CALENDAR CONTEXT ENRICHMENT
 * @version 3.1.0
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
  ABORIGINAL:     'aboriginal',
  THIRTEEN_MOON:  '13-moon',
  BYZANTINE:      'byzantine',
  ASSYRIAN:       'assyrian',
  JULIAN_DAY:     'julian-day',
  HEBREW:         'hebrew',
  MAYAN:          'mayan',
  AZTEC:          'aztec',
  KALI_YUGA:      'kali-yuga',
  NISGAA:         'nisgaa',
  EGYPTIAN:       'egyptian',
  CHINESE:        'chinese',
  KOREAN:         'korean',
  IGBO:           'igbo',
  YORUBA:         'yoruba',
  AKAN:           'akan',
  AMAZIGH:        'amazigh',
  ROMAN_AUC:      'roman-auc',
  JAPANESE:       'japanese',
  BUDDHIST:       'buddhist',
  JAIN:           'jain',
  TAMIL:          'tamil',
  VIETNAMESE:     'vietnamese',
  VIKRAM_SAMVAT:  'vikram-samvat',
  ETHIOPIAN:      'ethiopian',
  INDIAN_SAKA:    'indian-saka',
  COPTIC:         'coptic',
  KHMER:          'khmer',
  BENGALI:        'bengali',
  SOLAR_HIJRI:    'solar-hijri',
  HIJRI:          'hijri',
  ZOROASTRIAN:    'zoroastrian',
  BURMESE:        'burmese',
  JAVANESE:       'javanese',
  MALAYALAM:      'malayalam',
  NEPAL_SAMBAT:   'nepal-sambat',
  BALINESE:       'balinese',
  TIBETAN:        'tibetan',
  NANAKSHAHI:     'nanakshahi',
  GREGORIAN:      'gregorian',
  BAHAI:          'bahai',
  MINGUO:         'minguo',
} as const;

export const KNOWN_CALENDAR_SYSTEMS = new Set(Object.values(CALENDAR_SYSTEMS));

const CALENDAR_KEY_MAP: Record<string, string> = {
  'aboriginal':    'aboriginalSeasonal',
  '13-moon':       'thirteenMoon',
  'byzantine':     'byzantine',
  'assyrian':      'assyrian',
  'julian-day':    'julianDay',
  'hebrew':        'hebrew',
  'mayan':         'mayanLongCount',
  'aztec':         'aztecTonalpohualli',
  'kali-yuga':     'vedic',
  'nisgaa':        'nisgaaSeasonal',
  'egyptian':      'egyptian',
  'chinese':       'chineseSexagenary',
  'korean':        'koreanDangun',
  'igbo':          'igbo',
  'yoruba':        'yoruba',
  'akan':          'akan',
  'amazigh':       'berber',
  'roman-auc':     'romanAUC',
  'japanese':      'japaneseKoki',
  'buddhist':      'thaiBuddhist',
  'jain':          'jain',
  'tamil':         'tamil',
  'vietnamese':    'vietnamese',
  'vikram-samvat': 'vikramSamvat',
  'ethiopian':     'ethiopian',
  'indian-saka':   'indianSaka',
  'coptic':        'coptic',
  'khmer':         'khmer',
  'bengali':       'bengali',
  'solar-hijri':   'persian',
  'hijri':         'islamic',
  'zoroastrian':   'zoroastrianFasli',
  'burmese':       'burmese',
  'javanese':      'javanese',
  'malayalam':     'malayalam',
  'nepal-sambat':  'nepalSambat',
  'balinese':      'balinesePawukon',
  'tibetan':       'tibetan',
  'nanakshahi':    'nanakshahi',
  'gregorian':     'gregorian',
  'bahai':         'bahai',
  'minguo':        'minguo',
};

export const POLICY_CALENDAR_CONFIG: Record<string, string[]> = {
  'DEFAULT':    [],
  'COMPLY':     [
    CALENDAR_SYSTEMS.HIJRI,
    CALENDAR_SYSTEMS.SOLAR_HIJRI,
    CALENDAR_SYSTEMS.HEBREW,
    CALENDAR_SYSTEMS.JAPANESE,
    CALENDAR_SYSTEMS.BUDDHIST,
    CALENDAR_SYSTEMS.CHINESE,
    CALENDAR_SYSTEMS.INDIAN_SAKA,
    CALENDAR_SYSTEMS.VIKRAM_SAMVAT,
    CALENDAR_SYSTEMS.KOREAN,
    CALENDAR_SYSTEMS.NANAKSHAHI,
    CALENDAR_SYSTEMS.MINGUO,
  ],
  'FORENSICS':  ['*'],
  'SENTINEL':   [],
  'SECURE':     [],
};

export const CALENDAR_DISPLAY_NAMES: Record<string, string> = {
  'aboriginal':    'Aboriginal Seasonal (Dharawal)',
  '13-moon':       '13-Moon Harmonic',
  'byzantine':     'Byzantine (Anno Mundi)',
  'assyrian':      'Assyrian',
  'julian-day':    'Julian Day Number',
  'hebrew':        'Hebrew (Anno Mundi)',
  'mayan':         'Mayan Long Count',
  'aztec':         'Aztec Tonalpohualli',
  'kali-yuga':     'Vedic Kali Yuga',
  'nisgaa':        'Nisg\u0331a\'a Seasonal',
  'egyptian':      'Egyptian Civil',
  'chinese':       'Chinese Sexagenary',
  'korean':        'Korean (Dangun Era)',
  'igbo':          'Igbo',
  'yoruba':        'Yoruba',
  'akan':          'Akan',
  'amazigh':       'Amazigh / Berber',
  'roman-auc':     'Roman (Ab Urbe Condita)',
  'japanese':      'Japanese Imperial (K\u014dki)',
  'buddhist':      'Thai Buddhist Era',
  'jain':          'Jain (Vira Nirvana Samvat)',
  'tamil':         'Tamil',
  'vietnamese':    'Vietnamese',
  'vikram-samvat': 'Vikram Samvat',
  'ethiopian':     'Ethiopian / Ge\'ez',
  'indian-saka':   'Indian National (Saka)',
  'coptic':        'Coptic (Era of Martyrs)',
  'khmer':         'Khmer (Cambodian)',
  'bengali':       'Bengali / Bangla',
  'solar-hijri':   'Persian / Solar Hijri',
  'hijri':         'Islamic Hijri',
  'zoroastrian':   'Zoroastrian Fasli',
  'burmese':       'Burmese',
  'javanese':      'Javanese',
  'malayalam':     'Malayalam (Kollam Era)',
  'nepal-sambat':  'Nepal Sambat',
  'balinese':      'Balinese Pawukon',
  'tibetan':       'Tibetan (Rabjung)',
  'nanakshahi':    'Nanakshahi (Sikh)',
  'gregorian':     'Gregorian',
  'bahai':         'Bah\u00e1\'i (Bad\u00ed\')',
  'minguo':        'Minguo (Republic of China)',
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
    if (KNOWN_CALENDAR_SYSTEMS.has(sys as any)) {
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
    sys => !policySet.has(sys) && KNOWN_CALENDAR_SYSTEMS.has(sys as any),
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
      const formatted = (calObj as any).formatted || (calObj as any).longCount || (calObj as any).season;
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
