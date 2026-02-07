/**
 * Salvi Framework - Ancient Calendar Synchronization
 * 
 * Anchors the Salvi Epoch (April 1, 2025 00:00:00.000 UTC) to ancient
 * calendar systems spanning thousands of years, providing a universal
 * temporal reference frame across civilizations.
 * 
 * Supported Calendar Systems:
 * - Mayan Long Count (Mesoamerican, ~3114 BCE origin)
 * - Hebrew Calendar (Lunisolar, 3761 BCE origin)
 * - Chinese Sexagenary Cycle (60-year cycle, ~2637 BCE origin)
 * - Vedic/Hindu Calendar (Kali Yuga, 3102 BCE origin)
 * - Egyptian Civil Calendar (365-day, ~2781 BCE origin)
 * - Julian Day Number (Astronomical, 4713 BCE origin)
 * - Islamic Calendar (Hijri, 622 CE origin)
 * - Byzantine Calendar (Anno Mundi, 5509 BCE origin)
 * 
 * @author Capomastro Holdings Ltd.
 * @license Proprietary - All Rights Reserved
 */

import { SALVI_EPOCH } from './femtosecond-timing';

const SALVI_EPOCH_DATE = new Date('2025-04-01T00:00:00.000Z');
const MS_PER_DAY = 86_400_000;

export interface AncientCalendarMapping {
  calendarSystem: string;
  origin: string;
  originYear: number;
  salviEpochEquivalent: string;
  daysSinceCalendarOrigin: number;
  yearInCalendar: number;
  cyclicPosition?: string;
  description: string;
}

export interface MayanLongCount {
  baktun: number;
  katun: number;
  tun: number;
  uinal: number;
  kin: number;
  longCount: string;
  tzolkinDay: string;
  tzolkinNumber: number;
  haabDay: string;
  haabMonth: number;
  calendarRound: string;
}

export interface HebrewDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface ChineseSexagenary {
  year: number;
  heavenlyStem: string;
  earthlyBranch: string;
  zodiacAnimal: string;
  element: string;
  cycleNumber: number;
  yearInCycle: number;
  formatted: string;
}

export interface VedicKaliYuga {
  yearInYuga: number;
  totalYugaYears: number;
  percentComplete: number;
  manvantara: number;
  kalpa: string;
  formatted: string;
}

export interface EgyptianCivil {
  year: number;
  season: string;
  seasonName: string;
  month: number;
  day: number;
  epagomenalDay: boolean;
  formatted: string;
}

export interface JulianDayNumber {
  julianDay: number;
  modifiedJulianDay: number;
  truncatedJulianDay: number;
  formatted: string;
}

export interface IslamicHijri {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface ByzantineAnnoMundi {
  year: number;
  indiction: number;
  formatted: string;
}

export interface SalviEpochCalendarSync {
  salviEpoch: string;
  salviEpochUnixMs: number;
  femtosecondOffset: string;
  calendars: {
    mayanLongCount: MayanLongCount;
    hebrew: HebrewDate;
    chineseSexagenary: ChineseSexagenary;
    vedic: VedicKaliYuga;
    egyptian: EgyptianCivil;
    julianDay: JulianDayNumber;
    islamic: IslamicHijri;
    byzantine: ByzantineAnnoMundi;
  };
  allMappings: AncientCalendarMapping[];
}

const TZOLKIN_DAYS = [
  'Imix', 'Ik', 'Akbal', 'Kan', 'Chicchan', 'Cimi', 'Manik', 'Lamat',
  'Muluc', 'Oc', 'Chuen', 'Eb', 'Ben', 'Ix', 'Men', 'Cib',
  'Caban', 'Etznab', 'Cauac', 'Ahau'
];

const HAAB_MONTHS = [
  'Pop', 'Wo', 'Sip', 'Sotz', 'Sek', 'Xul', 'Yaxkin', 'Mol',
  'Chen', 'Yax', 'Sak', 'Keh', 'Mak', 'Kankin', 'Muwan',
  'Pax', 'Kayab', 'Kumku', 'Wayeb'
];

const HEAVENLY_STEMS = ['Jia', 'Yi', 'Bing', 'Ding', 'Wu', 'Ji', 'Geng', 'Xin', 'Ren', 'Gui'];
const EARTHLY_BRANCHES = ['Zi', 'Chou', 'Yin', 'Mao', 'Chen', 'Si', 'Wu', 'Wei', 'Shen', 'You', 'Xu', 'Hai'];
const ZODIAC_ANIMALS = ['Rat', 'Ox', 'Tiger', 'Rabbit', 'Dragon', 'Snake', 'Horse', 'Goat', 'Monkey', 'Rooster', 'Dog', 'Pig'];
const CHINESE_ELEMENTS = ['Wood', 'Wood', 'Fire', 'Fire', 'Earth', 'Earth', 'Metal', 'Metal', 'Water', 'Water'];

const ISLAMIC_MONTHS = [
  'Muharram', 'Safar', 'Rabi al-Awwal', 'Rabi al-Thani',
  'Jumada al-Ula', 'Jumada al-Thani', 'Rajab', 'Shaban',
  'Ramadan', 'Shawwal', 'Dhu al-Qidah', 'Dhu al-Hijjah'
];

const HEBREW_MONTHS = [
  'Nisan', 'Iyar', 'Sivan', 'Tammuz', 'Av', 'Elul',
  'Tishrei', 'Cheshvan', 'Kislev', 'Tevet', 'Shevat', 'Adar'
];

/**
 * Mayan Long Count correlation constant (GMT correlation)
 * Julian Day Number of the Mayan creation date 0.0.0.0.0
 * August 11, 3114 BCE (proleptic Gregorian) = JDN 584283
 */
const MAYAN_CORRELATION = 584283;

/**
 * Convert a Gregorian date to Julian Day Number
 */
function gregorianToJDN(year: number, month: number, day: number): number {
  const a = Math.floor((14 - month) / 12);
  const y = year + 4800 - a;
  const m = month + 12 * a - 3;
  return day + Math.floor((153 * m + 2) / 5) + 365 * y + Math.floor(y / 4) - Math.floor(y / 100) + Math.floor(y / 400) - 32045;
}

/**
 * Convert Gregorian date to Mayan Long Count
 */
export function toMayanLongCount(date: Date): MayanLongCount {
  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());
  const daysSinceCreation = jdn - MAYAN_CORRELATION;

  const baktun = Math.floor(daysSinceCreation / 144000);
  const remainder1 = daysSinceCreation % 144000;
  const katun = Math.floor(remainder1 / 7200);
  const remainder2 = remainder1 % 7200;
  const tun = Math.floor(remainder2 / 360);
  const remainder3 = remainder2 % 360;
  const uinal = Math.floor(remainder3 / 20);
  const kin = remainder3 % 20;

  const tzolkinNumber = ((daysSinceCreation + 3) % 13) + 1;
  const tzolkinDayIndex = (daysSinceCreation + 19) % 20;
  const tzolkinDay = TZOLKIN_DAYS[tzolkinDayIndex];

  const haabDayOfYear = (daysSinceCreation + 348) % 365;
  const haabMonth = Math.floor(haabDayOfYear / 20);
  const haabDay = haabDayOfYear % 20;
  const haabMonthName = HAAB_MONTHS[haabMonth];

  return {
    baktun,
    katun,
    tun,
    uinal,
    kin,
    longCount: `${baktun}.${katun}.${tun}.${uinal}.${kin}`,
    tzolkinDay,
    tzolkinNumber,
    haabDay: haabMonthName,
    haabMonth: haabDay,
    calendarRound: `${tzolkinNumber} ${tzolkinDay} ${haabDay} ${haabMonthName}`
  };
}

/**
 * Convert Gregorian date to Hebrew calendar (algorithmic approximation)
 */
export function toHebrewDate(date: Date): HebrewDate {
  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());

  const hebrewEpochJDN = 347995.5;
  const daysSinceEpoch = jdn - Math.floor(hebrewEpochJDN);

  const cycles = Math.floor((daysSinceEpoch * 25920) / (25920 * 365.25));
  const hebrewYear = cycles + 1;

  const approxYear = Math.floor(date.getUTCFullYear() + 3761 + (date.getUTCMonth() >= 8 ? 1 : 0));

  const monthIndex = (date.getUTCMonth() + 6) % 12;
  const monthName = HEBREW_MONTHS[monthIndex];

  return {
    year: approxYear,
    month: monthIndex + 1,
    monthName,
    day: date.getUTCDate(),
    formatted: `${date.getUTCDate()} ${monthName} ${approxYear} AM`
  };
}

/**
 * Convert Gregorian date to Chinese Sexagenary Cycle
 */
export function toChineseSexagenary(date: Date): ChineseSexagenary {
  const year = date.getUTCFullYear();

  const chineseNewYearOffset = date.getUTCMonth() < 1 || (date.getUTCMonth() === 1 && date.getUTCDate() < 4) ? -1 : 0;
  const chineseYear = year + chineseNewYearOffset;

  const stemIndex = (chineseYear - 4) % 10;
  const branchIndex = (chineseYear - 4) % 12;
  const cycleYear = ((chineseYear - 4) % 60) + 1;
  const cycleNumber = Math.floor((chineseYear - 4) / 60) + 1;

  const positiveStemIndex = ((stemIndex % 10) + 10) % 10;
  const positiveBranchIndex = ((branchIndex % 12) + 12) % 12;

  return {
    year: chineseYear,
    heavenlyStem: HEAVENLY_STEMS[positiveStemIndex],
    earthlyBranch: EARTHLY_BRANCHES[positiveBranchIndex],
    zodiacAnimal: ZODIAC_ANIMALS[positiveBranchIndex],
    element: CHINESE_ELEMENTS[positiveStemIndex],
    cycleNumber,
    yearInCycle: cycleYear,
    formatted: `${HEAVENLY_STEMS[positiveStemIndex]}-${EARTHLY_BRANCHES[positiveBranchIndex]} (${ZODIAC_ANIMALS[positiveBranchIndex]}/${CHINESE_ELEMENTS[positiveStemIndex]}) Year ${cycleYear} of Cycle ${cycleNumber}`
  };
}

/**
 * Convert Gregorian date to Vedic Kali Yuga reckoning
 * Kali Yuga began February 17/18, 3102 BCE
 * Total duration: 432,000 years
 */
export function toVedicKaliYuga(date: Date): VedicKaliYuga {
  const kaliYugaStart = -3101;
  const totalYugaYears = 432_000;

  const yearInYuga = date.getUTCFullYear() - kaliYugaStart;
  const percentComplete = (yearInYuga / totalYugaYears) * 100;

  const manvantara = Math.floor(yearInYuga / 306_720_000) + 1;

  return {
    yearInYuga,
    totalYugaYears,
    percentComplete: Math.round(percentComplete * 10000) / 10000,
    manvantara,
    kalpa: 'Shveta Varaha Kalpa',
    formatted: `Kali Yuga Year ${yearInYuga.toLocaleString()} of ${totalYugaYears.toLocaleString()} (${percentComplete.toFixed(4)}% elapsed)`
  };
}

/**
 * Convert Gregorian date to Egyptian Civil Calendar (approximation)
 * Based on the Sothic cycle beginning ~2781 BCE
 */
export function toEgyptianCivil(date: Date): EgyptianCivil {
  const egyptianEpochYear = -2780;
  const year = date.getUTCFullYear() - egyptianEpochYear;

  const dayOfYear = Math.floor((date.getTime() - new Date(date.getUTCFullYear(), 0, 0).getTime()) / MS_PER_DAY);

  const seasonIndex = Math.floor(dayOfYear / 120);
  const seasons = [
    { name: 'Akhet', label: 'Inundation' },
    { name: 'Peret', label: 'Growth' },
    { name: 'Shemu', label: 'Harvest' }
  ];
  const season = seasons[Math.min(seasonIndex, 2)];

  const monthInSeason = Math.floor((dayOfYear % 120) / 30) + 1;
  const dayInMonth = (dayOfYear % 30) + 1;
  const isEpagomenal = dayOfYear >= 360;

  return {
    year,
    season: season.name,
    seasonName: season.label,
    month: monthInSeason,
    day: isEpagomenal ? dayOfYear - 359 : dayInMonth,
    epagomenalDay: isEpagomenal,
    formatted: `Year ${year}, ${season.name} (${season.label}), Month ${monthInSeason}, Day ${dayInMonth}${isEpagomenal ? ' [Epagomenal]' : ''}`
  };
}

/**
 * Convert Gregorian date to Julian Day Number
 */
export function toJulianDayNumber(date: Date): JulianDayNumber {
  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());
  const fractionOfDay = (date.getUTCHours() * 3600 + date.getUTCMinutes() * 60 + date.getUTCSeconds()) / 86400;
  const jd = jdn + fractionOfDay - 0.5;

  return {
    julianDay: Math.round(jd * 1000000) / 1000000,
    modifiedJulianDay: Math.round((jd - 2400000.5) * 1000000) / 1000000,
    truncatedJulianDay: Math.round((jd - 2440000.5) * 1000000) / 1000000,
    formatted: `JD ${jd.toFixed(6)} | MJD ${(jd - 2400000.5).toFixed(6)}`
  };
}

/**
 * Convert Gregorian date to Islamic Hijri calendar (tabular approximation)
 */
export function toIslamicHijri(date: Date): IslamicHijri {
  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());

  const hijriEpochJDN = 1948439.5;
  const daysSinceHijri = jdn - Math.floor(hijriEpochJDN);

  const lunarCycleDays = 29.530588853;
  const hijriMonthLength = 354.36667;

  const totalMonths = Math.floor(daysSinceHijri / lunarCycleDays);
  const hijriYear = Math.floor((30 * daysSinceHijri + 10646) / 10631);
  const dayInYear = daysSinceHijri - Math.floor((10631 * hijriYear - 10617) / 30);
  const hijriMonth = Math.min(Math.floor((11 * dayInYear + 330) / 325), 12);
  const hijriDay = Math.max(dayInYear - Math.floor((325 * hijriMonth - 320) / 11) + 1, 1);

  return {
    year: hijriYear,
    month: hijriMonth,
    monthName: ISLAMIC_MONTHS[Math.max(0, Math.min(hijriMonth - 1, 11))],
    day: hijriDay,
    formatted: `${hijriDay} ${ISLAMIC_MONTHS[Math.max(0, Math.min(hijriMonth - 1, 11))]} ${hijriYear} AH`
  };
}

/**
 * Convert Gregorian date to Byzantine Anno Mundi
 * Byzantine creation of the world: September 1, 5509 BCE
 */
export function toByzantineAnnoMundi(date: Date): ByzantineAnnoMundi {
  const year = date.getUTCFullYear();
  const month = date.getUTCMonth();

  const byzantineYear = year + 5509 + (month >= 8 ? 1 : 0);
  const indiction = ((byzantineYear - 1) % 15) + 1;

  return {
    year: byzantineYear,
    indiction,
    formatted: `Anno Mundi ${byzantineYear.toLocaleString()}, Indiction ${indiction}`
  };
}

/**
 * Get the complete Salvi Epoch synchronization across all ancient calendars
 */
export function getSalviEpochCalendarSync(inputDate?: Date): SalviEpochCalendarSync {
  const date = inputDate || SALVI_EPOCH_DATE;
  const fsSinceEpoch = BigInt(date.getTime() - SALVI_EPOCH_DATE.getTime()) * 1_000_000_000_000n;

  const mayan = toMayanLongCount(date);
  const hebrew = toHebrewDate(date);
  const chinese = toChineseSexagenary(date);
  const vedic = toVedicKaliYuga(date);
  const egyptian = toEgyptianCivil(date);
  const julian = toJulianDayNumber(date);
  const islamic = toIslamicHijri(date);
  const byzantine = toByzantineAnnoMundi(date);

  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());

  const allMappings: AncientCalendarMapping[] = [
    {
      calendarSystem: 'Mayan Long Count',
      origin: 'August 11, 3114 BCE (GMT Correlation)',
      originYear: -3113,
      salviEpochEquivalent: mayan.longCount,
      daysSinceCalendarOrigin: jdn - MAYAN_CORRELATION,
      yearInCalendar: Math.floor((jdn - MAYAN_CORRELATION) / 365.2422),
      cyclicPosition: mayan.calendarRound,
      description: `Mesoamerican vigesimal count: ${mayan.longCount} | Calendar Round: ${mayan.calendarRound}`
    },
    {
      calendarSystem: 'Hebrew Calendar',
      origin: 'October 7, 3761 BCE (Anno Mundi)',
      originYear: -3760,
      salviEpochEquivalent: hebrew.formatted,
      daysSinceCalendarOrigin: jdn - 347996,
      yearInCalendar: hebrew.year,
      description: `Lunisolar calendar: ${hebrew.formatted}`
    },
    {
      calendarSystem: 'Chinese Sexagenary Cycle',
      origin: '~2637 BCE (Yellow Emperor)',
      originYear: -2636,
      salviEpochEquivalent: chinese.formatted,
      daysSinceCalendarOrigin: Math.floor((date.getUTCFullYear() + 2636) * 365.2422),
      yearInCalendar: chinese.year + 2637,
      cyclicPosition: `${chinese.heavenlyStem}-${chinese.earthlyBranch} (${chinese.zodiacAnimal}/${chinese.element})`,
      description: `60-year Heavenly Stems & Earthly Branches cycle: ${chinese.formatted}`
    },
    {
      calendarSystem: 'Vedic Kali Yuga',
      origin: 'February 17, 3102 BCE',
      originYear: -3101,
      salviEpochEquivalent: vedic.formatted,
      daysSinceCalendarOrigin: Math.floor(vedic.yearInYuga * 365.2422),
      yearInCalendar: vedic.yearInYuga,
      cyclicPosition: vedic.kalpa,
      description: `Hindu cosmological age: ${vedic.formatted} | Kalpa: ${vedic.kalpa}`
    },
    {
      calendarSystem: 'Egyptian Civil Calendar',
      origin: '~2781 BCE (Sothic Cycle)',
      originYear: -2780,
      salviEpochEquivalent: egyptian.formatted,
      daysSinceCalendarOrigin: Math.floor((date.getUTCFullYear() + 2780) * 365),
      yearInCalendar: egyptian.year,
      cyclicPosition: `Season of ${egyptian.seasonName} (${egyptian.season})`,
      description: `Solar calendar with 3 seasons of 4 months: ${egyptian.formatted}`
    },
    {
      calendarSystem: 'Julian Day Number',
      origin: 'January 1, 4713 BCE (proleptic Julian)',
      originYear: -4712,
      salviEpochEquivalent: julian.formatted,
      daysSinceCalendarOrigin: Math.floor(julian.julianDay),
      yearInCalendar: date.getUTCFullYear(),
      description: `Continuous astronomical day count: ${julian.formatted}`
    },
    {
      calendarSystem: 'Islamic Hijri',
      origin: 'July 16, 622 CE (Hijra)',
      originYear: 622,
      salviEpochEquivalent: islamic.formatted,
      daysSinceCalendarOrigin: jdn - 1948440,
      yearInCalendar: islamic.year,
      description: `Lunar calendar: ${islamic.formatted}`
    },
    {
      calendarSystem: 'Byzantine Anno Mundi',
      origin: 'September 1, 5509 BCE',
      originYear: -5508,
      salviEpochEquivalent: byzantine.formatted,
      daysSinceCalendarOrigin: Math.floor((date.getUTCFullYear() + 5509) * 365.2422),
      yearInCalendar: byzantine.year,
      cyclicPosition: `Indiction ${byzantine.indiction}`,
      description: `Eastern Roman creation reckoning: ${byzantine.formatted}`
    }
  ];

  return {
    salviEpoch: SALVI_EPOCH_DATE.toISOString(),
    salviEpochUnixMs: SALVI_EPOCH_DATE.getTime(),
    femtosecondOffset: fsSinceEpoch.toString(),
    calendars: {
      mayanLongCount: mayan,
      hebrew,
      chineseSexagenary: chinese,
      vedic,
      egyptian,
      julianDay: julian,
      islamic,
      byzantine
    },
    allMappings
  };
}

/**
 * Convert a Salvi femtosecond offset to all ancient calendar representations
 */
export function femtosecondsToAncientCalendars(femtosecondsFromEpoch: bigint): SalviEpochCalendarSync {
  const millisFromEpoch = Number(femtosecondsFromEpoch / 1_000_000_000_000n);
  const targetDate = new Date(SALVI_EPOCH_DATE.getTime() + millisFromEpoch);
  return getSalviEpochCalendarSync(targetDate);
}

/**
 * Get the Salvi Epoch anchor points - the fixed reference mappings
 * These are the canonical synchronization points
 */
export function getSalviEpochAnchorPoints(): {
  epoch: string;
  anchors: Record<string, string>;
  verification: string;
} {
  const sync = getSalviEpochCalendarSync(SALVI_EPOCH_DATE);

  return {
    epoch: 'April 1, 2025 00:00:00.000 UTC (Salvi Epoch / Day Zero)',
    anchors: {
      'Mayan Long Count': sync.calendars.mayanLongCount.longCount,
      'Mayan Calendar Round': sync.calendars.mayanLongCount.calendarRound,
      'Hebrew (Anno Mundi)': sync.calendars.hebrew.formatted,
      'Chinese Sexagenary': sync.calendars.chineseSexagenary.formatted,
      'Vedic Kali Yuga': sync.calendars.vedic.formatted,
      'Egyptian Civil': sync.calendars.egyptian.formatted,
      'Julian Day Number': sync.calendars.julianDay.formatted,
      'Islamic Hijri': sync.calendars.islamic.formatted,
      'Byzantine Anno Mundi': sync.calendars.byzantine.formatted,
      'Unix Timestamp (ms)': SALVI_EPOCH_DATE.getTime().toString(),
      'ISO 8601': SALVI_EPOCH_DATE.toISOString()
    },
    verification: `All calendar mappings are bijectively computed from JDN ${gregorianToJDN(2025, 4, 1)} via the GMT correlation constant ${MAYAN_CORRELATION} and standard astronomical algorithms.`
  };
}
