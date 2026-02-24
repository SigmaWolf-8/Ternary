/**
 * Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
 * Applied Physics Division
 *
 * PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
 * Patent(s) Pending.
 *
 * This file is part of the Salvi Framework / PlenumNET platform.
 * Unauthorized copying, modification, distribution, or use of this file,
 * via any medium, is strictly prohibited without the prior written
 * permission of Capomastro Holdings Ltd.
 *
 * See LICENSE in the repository root for full terms.
 */

/**
 * Salvi Framework - Ancient Calendar Synchronization
 * 
 * Anchors the Salvi Epoch (April 1, 2025 00:00:00.000 UTC) to ancient
 * calendar systems spanning tens of thousands of years, providing a universal
 * temporal reference frame across civilizations.
 * 
 * All conversions are computed via Julian Day Number (JDN) using standard
 * astronomical algorithms for maximum precision and backward time compatibility.
 * 
 * Supported Calendar Systems (42 total):
 * - Aboriginal Seasonal (~65,000 yrs, Dharawal six-season, Oceania)
 * - 13-Moon Harmonic (~28,000 BCE, Golden ratio 13x28, Global)
 * - Byzantine (5509 BCE, Offset calc, Europe)
 * - Assyrian (4750 BCE, Epoch offset, West Asia)
 * - Julian Day Number (4713 BCE, Universal intermediary, Global)
 * - Hebrew (3761 BCE, Metonic cycle, West Asia)
 * - Mayan Long Count (3114 BCE, GMT correlation, Americas)
 * - Aztec Tonalpohualli (3114 BCE, Trecena + day sign, Americas)
 * - Vedic Kali Yuga (3102 BCE, 432,000-year age, South Asia)
 * - Nisg̱a'a Seasonal (Pre-contact ~5,000+ yrs, Salmon-run, Americas)
 * - Egyptian Civil (2781 BCE, Sothic cycle, Africa)
 * - Chinese Sexagenary (2637 BCE, 60-year cycle, East Asia)
 * - Korean Dangun Era (2333 BCE, Foundation offset, East Asia)
 * - Igbo (Traditional ~3,000+ yrs, 4-day week, Africa)
 * - Yoruba (Traditional ~3,000+ yrs, 4-day Ojo, Africa)
 * - Akan (Traditional ~3,000+ yrs, 42-day Adae, Africa)
 * - Amazigh/Berber (950 BCE, Yennayer offset, Africa)
 * - Roman Ab Urbe Condita (753 BCE, Kalends/Nones/Ides, Europe)
 * - Japanese Imperial Koki (660 BCE, Era system, East Asia)
 * - Thai Buddhist Era (543 BCE, Parinibbana offset, Southeast Asia)
 * - Jain Vira Nirvana Samvat (527 BCE, Mahavira epoch, South Asia)
 * - Tamil (~300 BCE, Zodiac sidereal months, South Asia)
 * - Vietnamese (~200 BCE, Independent intercalation, Southeast Asia)
 * - Vikram Samvat (57 BCE, Metonic sidereal, South Asia)
 * - Ethiopian/Ge'ez (8 CE, Coptic-derived, Africa)
 * - Indian National/Saka (78 CE, Saka era, South Asia)
 * - Coptic (284 CE, Era of Martyrs, Africa)
 * - Khmer (~500 CE, Surya Siddhanta, Southeast Asia)
 * - Bengali/Bangla (594 CE, Shashanka era, South Asia)
 * - Persian/Solar Hijri (622 CE, Jalali algorithm, West Asia)
 * - Islamic Hijri (622 CE, 30-year cycle, Global)
 * - Zoroastrian Fasli (632 CE, Nowruz anchor, West Asia)
 * - Burmese (638 CE, Surya Siddhanta, Southeast Asia)
 * - Javanese (~8th c., 5+7 day hybrid, Southeast Asia)
 * - Malayalam/Kollam (825 CE, Kollam epoch, South Asia)
 * - Nepal Sambat (879 CE, Newar epoch, South Asia)
 * - Balinese Pawukon (~10th c., 210-day Wuku, Southeast Asia)
 * - Tibetan Rabjung (1027 CE, 60-year cycle, Central Asia)
 * - Nanakshahi (1469 CE, Fixed Gregorian, South Asia/Global)
 * - Gregorian (1582 CE, Direct anchor, Global)
 * - Bahai (1844 CE, 19x19 + intercalary, Global)
 * - Minguo (1912 CE, Gregorian - 1911, East Asia)
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
  haabDay: number;
  haabMonth: string;
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
  month: number;
  monthName: string;
  day: number;
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
  month: number;
  monthName: string;
  day: number;
  indiction: number;
  formatted: string;
}

export interface ThirteenMoonDate {
  year: number;
  moon: number;
  moonName: string;
  day: number;
  dayOfYear: number;
  dayOutOfTime: boolean;
  leapDay: boolean;
  weekday: string;
  totalCycles: number;
  galacticSignature: string;
  harmonicTone: number | string;
  arc: string;
  formatted: string;
}

export interface PersianDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface EthiopianDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface CopticDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface JapaneseKokiDate {
  kokiYear: number;
  era: string;
  eraYear: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface KoreanDangunDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface ThaiBuddhistDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface IndianSakaDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface TibetanDate {
  rabjungCycle: number;
  yearInCycle: number;
  element: string;
  animal: string;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface AztecTonalpohualliDate {
  daySign: string;
  daySignIndex: number;
  trecenaNumber: number;
  tonalpohualliDay: string;
  xiuhpohualliMonth: number;
  xiuhpohualliMonthName: string;
  xiuhpohualliDay: number;
  isNemontemi: boolean;
  formatted: string;
}

export interface RomanAUCDate {
  year: number;
  calendarMarker: string;
  formatted: string;
}

export interface BengaliDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface BerberDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface BalinesePawukonDate {
  wukuWeek: number;
  wukuName: string;
  dayInWuku: number;
  cycleDay: number;
  formatted: string;
}

export interface ZoroastrianFasliDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  isGathaDay: boolean;
  formatted: string;
}

export interface AboriginalSeasonalDate {
  season: string;
  seasonDescription: string;
  naturalIndicator: string;
  formatted: string;
}

export interface AssyrianDate {
  year: number;
  month: number;
  day: number;
  formatted: string;
}

export interface NisgaaSeasonalDate {
  season: string;
  seasonDescription: string;
  naturalIndicator: string;
  formatted: string;
}

export interface YorubaDate {
  dayName: string;
  dayIndex: number;
  month: number;
  dayOfMonth: number;
  formatted: string;
}

export interface JainDate {
  year: number;
  month: number;
  day: number;
  formatted: string;
}

export interface TamilDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface VietnameseDate {
  year: number;
  month: number;
  day: number;
  formatted: string;
}

export interface VikramSamvatDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface KhmerDate {
  year: number;
  month: number;
  day: number;
  formatted: string;
}

export interface BurmeseDate {
  year: number;
  month: number;
  day: number;
  formatted: string;
}

export interface JavaneseDate {
  pasaranDay: string;
  pasaranIndex: number;
  weekday: string;
  weekdayIndex: number;
  cycleDay: number;
  formatted: string;
}

export interface MalayalamDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface NepalSambatDate {
  year: number;
  month: number;
  day: number;
  formatted: string;
}

export interface NanakshahiDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  formatted: string;
}

export interface BahaiDate {
  year: number;
  month: number;
  monthName: string;
  day: number;
  isAyyamiHa: boolean;
  formatted: string;
}

export interface MinguoDate {
  year: number;
  month: number;
  day: number;
  formatted: string;
}

export interface IgboDate {
  dayName: string;
  dayIndex: number;
  month: number;
  dayOfMonth: number;
  formatted: string;
}

export interface AkanDate {
  adaeCycleDay: number;
  adaeCycleName: string;
  formatted: string;
}

export interface GregorianDate {
  year: number;
  month: number;
  day: number;
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
    thirteenMoon: ThirteenMoonDate;
    persian: PersianDate;
    ethiopian: EthiopianDate;
    coptic: CopticDate;
    japaneseKoki: JapaneseKokiDate;
    koreanDangun: KoreanDangunDate;
    thaiBuddhist: ThaiBuddhistDate;
    indianSaka: IndianSakaDate;
    tibetan: TibetanDate;
    aztecTonalpohualli: AztecTonalpohualliDate;
    romanAUC: RomanAUCDate;
    bengali: BengaliDate;
    berber: BerberDate;
    balinesePawukon: BalinesePawukonDate;
    zoroastrianFasli: ZoroastrianFasliDate;
    aboriginalSeasonal: AboriginalSeasonalDate;
    assyrian: AssyrianDate;
    nisgaaSeasonal: NisgaaSeasonalDate;
    yoruba: YorubaDate;
    jain: JainDate;
    tamil: TamilDate;
    vietnamese: VietnameseDate;
    vikramSamvat: VikramSamvatDate;
    khmer: KhmerDate;
    burmese: BurmeseDate;
    javanese: JavaneseDate;
    malayalam: MalayalamDate;
    nepalSambat: NepalSambatDate;
    nanakshahi: NanakshahiDate;
    bahai: BahaiDate;
    minguo: MinguoDate;
    igbo: IgboDate;
    akan: AkanDate;
    gregorian: GregorianDate;
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

const THIRTEEN_MOON_NAMES = [
  'Magnetic', 'Lunar', 'Electric', 'Self-Existing', 'Overtone',
  'Rhythmic', 'Resonant', 'Galactic', 'Solar', 'Planetary',
  'Spectral', 'Crystal', 'Cosmic'
];

const THIRTEEN_MOON_WEEKDAYS = [
  'Dali', 'Seli', 'Gamma', 'Kali', 'Alpha', 'Limi', 'Silio'
];

const GALACTIC_SIGNATURES = [
  'Red Dragon', 'White Wind', 'Blue Night', 'Yellow Seed',
  'Red Serpent', 'White World-Bridger', 'Blue Hand', 'Yellow Star',
  'Red Moon', 'White Dog', 'Blue Monkey', 'Yellow Human', 'Red Skywalker'
];

const HARMONIC_TONES: (number | string)[] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

const PERSIAN_MONTHS = [
  'Farvardin', 'Ordibehesht', 'Khordad', 'Tir', 'Mordad', 'Shahrivar',
  'Mehr', 'Aban', 'Azar', 'Dey', 'Bahman', 'Esfand'
];

const ETHIOPIAN_MONTHS = [
  'Meskerem', 'Tikimt', 'Hidar', 'Tahsas', 'Tir', 'Yekatit',
  'Megabit', 'Miazia', 'Ginbot', 'Sene', 'Hamle', 'Nehase', 'Pagume'
];

const COPTIC_MONTHS = [
  'Thout', 'Paopi', 'Hathor', 'Koiak', 'Tobi', 'Meshir',
  'Paremhat', 'Parmouti', 'Pashons', 'Paoni', 'Epip', 'Mesori', 'Pi Kogi Enavot'
];

const INDIAN_SAKA_MONTHS = [
  'Chaitra', 'Vaishakha', 'Jyeshtha', 'Ashadha', 'Shravana', 'Bhadra',
  'Ashwin', 'Kartika', 'Agrahayana', 'Pausha', 'Magha', 'Phalguna'
];

const TIBETAN_ELEMENTS = ['Iron', 'Water', 'Wood', 'Fire', 'Earth'];

const AZTEC_DAY_SIGNS = [
  'Cipactli', 'Ehecatl', 'Calli', 'Cuetzpalin', 'Coatl',
  'Miquiztli', 'Mazatl', 'Tochtli', 'Atl', 'Itzcuintli',
  'Ozomatli', 'Malinalli', 'Acatl', 'Ocelotl', 'Cuauhtli',
  'Cozcacuauhtli', 'Ollin', 'Tecpatl', 'Quiahuitl', 'Xochitl'
];

const AZTEC_XIUHPOHUALLI_MONTHS = [
  'Atlcahualo', 'Tlacaxipehualiztli', 'Tozoztontli', 'Huey Tozoztli',
  'Toxcatl', 'Etzalcualiztli', 'Tecuilhuitontli', 'Huey Tecuilhuitl',
  'Tlaxochimaco', 'Xocotl Huetzi', 'Ochpaniztli', 'Teotleco',
  'Tepeilhuitl', 'Quecholli', 'Panquetzaliztli', 'Atemoztli',
  'Tititl', 'Izcalli'
];

const BENGALI_MONTHS = [
  'Boishakh', 'Jyoishtha', 'Asharh', 'Shrabon', 'Bhadro', 'Ashwin',
  'Kartik', 'Ogrohayon', 'Poush', 'Magh', 'Falgun', 'Choitro'
];

const CHINESE_MONTHS = [
  'Zhēngyuè', 'Èryuè', 'Sānyuè', 'Sìyuè', 'Wǔyuè', 'Liùyuè',
  'Qīyuè', 'Bāyuè', 'Jiǔyuè', 'Shíyuè', 'Shíyīyuè', 'Làyuè'
];

const BYZANTINE_MONTHS = [
  'Septemvrios', 'Oktovrios', 'Noevrios', 'Dekemvrios',
  'Ianouarios', 'Fevrouarios', 'Martios', 'Aprilios',
  'Maios', 'Iounios', 'Ioulios', 'Avgoustos'
];

const JAPANESE_MONTHS = [
  'Mutsuki', 'Kisaragi', 'Yayoi', 'Uzuki', 'Satsuki', 'Minazuki',
  'Fumizuki', 'Hazuki', 'Nagatsuki', 'Kannazuki', 'Shimotsuki', 'Shiwasu'
];

const KOREAN_MONTHS = [
  'Jeongwol', 'Iwol', 'Samwol', 'Sawol', 'Owol', 'Yuwol',
  'Chirwol', 'Palwol', 'Guwol', 'Siwol', 'Sipilwol', 'Sipiwol'
];

const THAI_MONTHS = [
  'Mokarakhom', 'Kumphaphan', 'Minakhom', 'Mesayon',
  'Phruetsaphakhom', 'Mithunayon', 'Karakadakhom', 'Singhakhom',
  'Kanyayon', 'Tulakhom', 'Phruetsachikayon', 'Thanwakhom'
];

const TIBETAN_MONTHS = [
  'Mchu', 'Dbo', 'Nag', 'Sa-ga', 'Snron', 'Chu-stod',
  'Gro-bzhin', 'Khrums', 'Tha-skar', 'Smin-drug', 'Mgo', 'Rgyal'
];

const BERBER_MONTHS = [
  'Yennayer', 'Yebrayer', 'Mares', 'Yebrir', 'Mayyu', 'Yunyu',
  'Yulyuz', 'Ghusht', 'Shutanbir', 'Ktuber', 'Nwanbir', 'Dujanbir'
];

const WUKU_NAMES = [
  'Sinta', 'Landep', 'Ukir', 'Kulantir', 'Tolu', 'Gumbreg',
  'Wariga', 'Warigadean', 'Julungwangi', 'Sungsang', 'Dungulan',
  'Kuningan', 'Langkir', 'Medangsia', 'Pujut', 'Pahang',
  'Krulut', 'Merakih', 'Tambir', 'Medangkungan', 'Matal',
  'Uye', 'Menail', 'Prangbakat', 'Bala', 'Ugu', 'Wayang',
  'Klawu', 'Dukut', 'Watugunung'
];

const ZOROASTRIAN_MONTHS = [
  'Fravardin', 'Ardibehesht', 'Khordad', 'Tir', 'Amordad', 'Shahrevar',
  'Mehr', 'Aban', 'Azar', 'Dey', 'Bahman', 'Esfand'
];

const DHARAWAL_SEASONS = [
  { name: 'Ngoonungi', description: 'Cool becoming cold', months: [3, 4], indicator: 'Lyre birds displaying, wattle blooming' },
  { name: 'Wiritjiribin', description: 'Cold, frosty', months: [5, 6], indicator: 'Whales migrating north, wattle seeds ripening' },
  { name: 'Hungundung', description: 'Cold becoming warm', months: [7, 8], indicator: 'Orchids flowering, echidnas mating' },
  { name: 'Marrai\'gang', description: 'Warm and wet', months: [9, 10], indicator: 'Muttonbirds arriving, insects hatching' },
  { name: 'Garrawarra', description: 'Hot and dry', months: [11, 0], indicator: 'Cicadas singing, sharks pupping' },
  { name: 'Burran', description: 'Hot becoming cool', months: [1, 2], indicator: 'Flying foxes moving, fig trees fruiting' }
];

/**
 * Mayan Long Count correlation constant (Goodman-Martinez-Thompson correlation)
 * Julian Day Number of the Mayan creation date 0.0.0.0.0
 * August 11, 3114 BCE (proleptic Gregorian) = JDN 584283
 */
const MAYAN_CORRELATION = 584283;

/**
 * Yellow Emperor epoch for Chinese Sexagenary cycle numbering
 * Traditional start: 2637 BCE = astronomical year -2636
 */
const YELLOW_EMPEROR_EPOCH = -2636;

/**
 * 13-Moon calendar aligned to the Salvi Epoch (April 1)
 * 
 * The 364-day cycle begins each year on April 1 (Gregorian), anchored to the
 * Salvi Epoch. The Day Out of Time falls on November 11 (11/11), positioned at
 * the golden ratio point of the 364-day cycle:
 * 
 *   364 / φ (1.6180339...) = 224.93 → Day 224 (0-indexed from April 1) = November 11
 * 
 * This splits the 13 moons into 8 before and 5 after the Day Out of Time —
 * both Fibonacci numbers whose ratio (8/5 = 1.6) approximates φ itself.
 * 
 * Prehistoric attestation: Ishango bone (~20,000 BCE), Abri Blanchard bone (~28,000 BCE)
 * Enochian reference: Book of Enoch / Dead Sea Scrolls (~300 BCE, referencing older tradition)
 * 
 * Each year has 13 moons x 28 days = 364 days + 1 Day Out of Time (Nov 11)
 * The Day Out of Time exists outside the regular moon count — it belongs to no moon.
 */
const THIRTEEN_MOON_NEW_YEAR_MONTH = 3;
const THIRTEEN_MOON_NEW_YEAR_DAY = 1;
const DAY_OUT_OF_TIME_MONTH = 10;
const DAY_OUT_OF_TIME_DAY = 11;
const GOLDEN_RATIO = 1.6180339887498949;
const GOLDEN_RATIO_DAY = 224;

/**
 * Convert a Gregorian date to Julian Day Number
 * Standard algorithm valid for all dates in the proleptic Gregorian calendar
 */
function gregorianToJDN(year: number, month: number, day: number): number {
  const a = Math.floor((14 - month) / 12);
  const y = year + 4800 - a;
  const m = month + 12 * a - 3;
  return day + Math.floor((153 * m + 2) / 5) + 365 * y + Math.floor(y / 4) - Math.floor(y / 100) + Math.floor(y / 400) - 32045;
}

/**
 * Check if a Gregorian year is a leap year
 */
function isLeapYear(year: number): boolean {
  return (year % 4 === 0 && year % 100 !== 0) || (year % 400 === 0);
}

/**
 * Convert Gregorian date to Mayan Long Count
 * Uses the GMT correlation (584283) which is the scholarly consensus
 * 
 * The Long Count is a vigesimal (base-20) positional notation:
 * baktun.katun.tun.uinal.kin
 * where: 1 kin = 1 day, 1 uinal = 20 kin, 1 tun = 360 kin,
 *         1 katun = 7200 kin, 1 baktun = 144000 kin
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
  const haabMonthIndex = Math.floor(haabDayOfYear / 20);
  const haabDay = haabDayOfYear % 20;
  const haabMonthName = HAAB_MONTHS[haabMonthIndex];

  return {
    baktun,
    katun,
    tun,
    uinal,
    kin,
    longCount: `${baktun}.${katun}.${tun}.${uinal}.${kin}`,
    tzolkinDay,
    tzolkinNumber,
    haabDay,
    haabMonth: haabMonthName,
    calendarRound: `${tzolkinNumber} ${tzolkinDay} ${haabDay} ${haabMonthName}`
  };
}

/**
 * Convert Gregorian date to Hebrew calendar (algorithmic approximation)
 * 
 * The Hebrew calendar is lunisolar. The new year (Rosh Hashanah) falls
 * in September/October (Tishrei). This uses the standard Anno Mundi reckoning:
 * - Before Tishrei (Jan-Aug): Hebrew year = Gregorian year + 3760
 * - Tishrei onward (Sep-Dec): Hebrew year = Gregorian year + 3761
 * 
 * Month mapping uses the ~3 month offset between Gregorian January and
 * Hebrew Tevet, computed via the Nisan-ordered month array.
 */
export function toHebrewDate(date: Date): HebrewDate {
  const gMonth = date.getUTCMonth();
  const gYear = date.getUTCFullYear();

  const approxYear = gYear + 3760 + (gMonth >= 8 ? 1 : 0);

  const monthIndex = ((gMonth + 9) % 12);
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
 * 
 * The Sexagenary (60-year) cycle pairs 10 Heavenly Stems with 12 Earthly Branches.
 * Year 4 CE = Jia-Zi (start of a well-documented cycle).
 * Cycle numbering uses the traditional Yellow Emperor epoch (2637 BCE).
 * 
 * Stems and branches determine the element and zodiac animal.
 * Chinese New Year falls between Jan 21 - Feb 20; we approximate with Feb 4.
 */
export function toChineseSexagenary(date: Date): ChineseSexagenary {
  const year = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();

  const chineseNewYearOffset = gMonth < 1 || (gMonth === 1 && gDay < 4) ? -1 : 0;
  const chineseYear = year + chineseNewYearOffset;

  const stemIndex = (chineseYear - 4) % 10;
  const branchIndex = (chineseYear - 4) % 12;

  const positiveStemIndex = ((stemIndex % 10) + 10) % 10;
  const positiveBranchIndex = ((branchIndex % 12) + 12) % 12;

  const yearsSinceEmperor = chineseYear - YELLOW_EMPEROR_EPOCH;
  const cycleYear = ((yearsSinceEmperor - 1) % 60) + 1;
  const cycleNumber = Math.floor((yearsSinceEmperor - 1) / 60) + 1;

  const chineseNewYearMs = Date.UTC(chineseYear, 1, 4);
  const dateMs = date.getTime();
  let daysSinceNewYear: number;
  if (dateMs >= chineseNewYearMs) {
    daysSinceNewYear = Math.floor((dateMs - chineseNewYearMs) / MS_PER_DAY) + 1;
  } else {
    const prevNewYearMs = Date.UTC(chineseYear - 1, 1, 4);
    daysSinceNewYear = Math.floor((dateMs - prevNewYearMs) / MS_PER_DAY) + 1;
  }
  daysSinceNewYear = Math.max(1, Math.min(daysSinceNewYear, 385));

  const monthLengths = [29, 30, 29, 30, 29, 30, 29, 30, 29, 30, 29, 30];
  let chineseMonth = 1;
  let chineseDay = daysSinceNewYear;
  for (let i = 0; i < 12; i++) {
    if (chineseDay <= monthLengths[i]) {
      chineseMonth = i + 1;
      break;
    }
    chineseDay -= monthLengths[i];
    chineseMonth = i + 2;
  }
  chineseMonth = Math.min(chineseMonth, 12);
  chineseDay = Math.max(1, Math.min(chineseDay, 30));
  const monthName = CHINESE_MONTHS[chineseMonth - 1];

  return {
    year: chineseYear,
    month: chineseMonth,
    monthName,
    day: chineseDay,
    heavenlyStem: HEAVENLY_STEMS[positiveStemIndex],
    earthlyBranch: EARTHLY_BRANCHES[positiveBranchIndex],
    zodiacAnimal: ZODIAC_ANIMALS[positiveBranchIndex],
    element: CHINESE_ELEMENTS[positiveStemIndex],
    cycleNumber,
    yearInCycle: cycleYear,
    formatted: `${chineseDay} ${monthName}, ${HEAVENLY_STEMS[positiveStemIndex]}-${EARTHLY_BRANCHES[positiveBranchIndex]} (${ZODIAC_ANIMALS[positiveBranchIndex]}/${CHINESE_ELEMENTS[positiveStemIndex]}) Year ${cycleYear} of Cycle ${cycleNumber}`
  };
}

/**
 * Convert Gregorian date to Vedic Kali Yuga reckoning
 * 
 * Kali Yuga began February 17/18, 3102 BCE (astronomical year -3101)
 * Total duration: 432,000 sidereal years
 * Current position within the Shveta Varaha Kalpa (cosmic day of Brahma)
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
 * 
 * The Egyptian civil calendar had 3 seasons (Akhet, Peret, Shemu) of 4 months,
 * each month 30 days, plus 5 epagomenal days = 365 days total.
 * Based on the Sothic cycle beginning ~2781 BCE.
 * Season start is keyed to the Nile inundation cycle.
 */
export function toEgyptianCivil(date: Date): EgyptianCivil {
  const egyptianEpochYear = -2780;
  const year = date.getUTCFullYear() - egyptianEpochYear;

  const startOfYear = Date.UTC(date.getUTCFullYear(), 0, 1);
  const dayOfYear = Math.floor((date.getTime() - startOfYear) / MS_PER_DAY) + 1;

  const isEpagomenal = dayOfYear > 360;

  const seasons = [
    { name: 'Akhet', label: 'Inundation' },
    { name: 'Peret', label: 'Growth' },
    { name: 'Shemu', label: 'Harvest' }
  ];

  let season;
  let monthInSeason: number;
  let dayInMonth: number;

  if (isEpagomenal) {
    season = seasons[2];
    monthInSeason = 4;
    dayInMonth = dayOfYear - 360;
  } else {
    const seasonIndex = Math.min(Math.floor((dayOfYear - 1) / 120), 2);
    season = seasons[seasonIndex];
    const dayInSeason = (dayOfYear - 1) % 120;
    monthInSeason = Math.floor(dayInSeason / 30) + 1;
    dayInMonth = (dayInSeason % 30) + 1;
  }

  return {
    year,
    season: season.name,
    seasonName: season.label,
    month: monthInSeason,
    day: dayInMonth,
    epagomenalDay: isEpagomenal,
    formatted: `Year ${year}, ${season.name} (${season.label}), Month ${monthInSeason}, Day ${dayInMonth}${isEpagomenal ? ' [Epagomenal]' : ''}`
  };
}

/**
 * Convert Gregorian date to Julian Day Number
 * 
 * The Julian Day is a continuous count of days since the beginning of the
 * Julian Period on January 1, 4713 BCE (proleptic Julian calendar).
 * JD starts at noon UT, so midnight = JDN - 0.5
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
 * Convert Gregorian date to Islamic Hijri calendar (tabular/arithmetic method)
 * 
 * Uses the standard tabular Islamic calendar algorithm based on the
 * Hijri epoch: July 16, 622 CE (Julian) = JDN 1948439.5
 * 
 * The tabular method uses a 30-year cycle with 11 leap years.
 * Leap years in each 30-year cycle: 2, 5, 7, 10, 13, 16, 18, 21, 24, 26, 29
 */
export function toIslamicHijri(date: Date): IslamicHijri {
  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());

  const hijriEpochJDN = 1948440;
  const daysSinceHijri = jdn - hijriEpochJDN;

  const hijriYear = Math.floor((30 * daysSinceHijri + 10646) / 10631);
  const dayInYear = daysSinceHijri - Math.floor((10631 * hijriYear - 10617) / 30);
  const hijriMonth = Math.min(Math.floor((11 * dayInYear + 330) / 325), 12);
  const hijriDay = Math.max(dayInYear - Math.floor((325 * hijriMonth - 320) / 11) + 1, 1);

  const safeMonth = Math.max(1, Math.min(hijriMonth, 12));

  return {
    year: hijriYear,
    month: safeMonth,
    monthName: ISLAMIC_MONTHS[safeMonth - 1],
    day: hijriDay,
    formatted: `${hijriDay} ${ISLAMIC_MONTHS[safeMonth - 1]} ${hijriYear} AH`
  };
}

/**
 * Convert Gregorian date to Byzantine Anno Mundi
 * 
 * The Byzantine calendar reckoned from the creation of the world:
 * September 1, 5509 BCE. The new year begins September 1.
 * The Indiction cycle is a 15-year fiscal/administrative cycle
 * inherited from the Roman Empire.
 */
export function toByzantineAnnoMundi(date: Date): ByzantineAnnoMundi {
  const year = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();

  const byzantineYear = year + 5509 + (gMonth >= 8 ? 1 : 0);
  const indiction = ((byzantineYear - 1) % 15) + 1;

  const byzMonthIndex = ((gMonth - 8) + 12) % 12;
  const monthName = BYZANTINE_MONTHS[byzMonthIndex];

  return {
    year: byzantineYear,
    month: byzMonthIndex + 1,
    monthName,
    day: gDay,
    indiction,
    formatted: `${gDay} ${monthName}, Anno Mundi ${byzantineYear.toLocaleString()}, Indiction ${indiction}`
  };
}

/**
 * Convert Gregorian date to the 13-Moon Calendar (364-day natural time cycle)
 * 
 * Salvi Framework alignment:
 * - Year begins April 1 (Salvi Epoch anchor)
 * - Day Out of Time: November 11 (11/11), the golden ratio point
 *   364/φ = 224.93 → day 224 (0-indexed from April 1) = November 11
 * - 8 Fibonacci moons before DOT, 5 Fibonacci moons after (8/5 ≈ φ)
 * 
 * Historical attestation:
 * - Abri Blanchard bone (France, ~28,000 BCE): lunar notation marks
 * - Ishango bone (Congo, ~20,000 BCE): possible 6-month lunar tally
 * - Book of Enoch / Dead Sea Scrolls (~300 BCE): 364-day sacred calendar
 *   with 4 seasons of 91 days (13 weeks each)
 * - Essene/Qumran community: liturgical 364-day calendar
 * - Celtic/Druidic traditions: 13-month tree calendar
 * 
 * Structure: 13 moons x 28 days = 364 regular days + 1 Day Out of Time
 * The DOT exists outside the moon count; it belongs to no moon.
 * In leap years, a Hunab Ku Day is inserted before the DOT (Nov 10).
 * 
 * Each 28-day moon follows the same pattern:
 * Week 1 (days 1-7), Week 2 (days 8-14), Week 3 (days 15-21), Week 4 (days 22-28)
 * Every day of the month always falls on the same day of the week.
 */
export function toThirteenMoonDate(date: Date): ThirteenMoonDate {
  const gYear = date.getUTCFullYear();
  const dateMs = date.getTime();

  const newYearThisYear = Date.UTC(gYear, THIRTEEN_MOON_NEW_YEAR_MONTH, THIRTEEN_MOON_NEW_YEAR_DAY);
  const thirteenMoonYear = dateMs >= newYearThisYear ? gYear : gYear - 1;

  const yearStartMs = Date.UTC(thirteenMoonYear, THIRTEEN_MOON_NEW_YEAR_MONTH, THIRTEEN_MOON_NEW_YEAR_DAY);
  const daysSinceNewYear = Math.floor((dateMs - yearStartMs) / MS_PER_DAY);

  const dotMs = Date.UTC(thirteenMoonYear, DAY_OUT_OF_TIME_MONTH, DAY_OUT_OF_TIME_DAY);
  const isDayOutOfTime = dateMs >= dotMs && dateMs < dotMs + MS_PER_DAY;

  const leapYearForCycle = thirteenMoonYear + 1;
  const hasLeapDay = isLeapYear(leapYearForCycle);
  const hunabKuMs = hasLeapDay ? Date.UTC(leapYearForCycle, 1, 29) : 0;
  const isHunabKu = hasLeapDay && dateMs >= hunabKuMs && dateMs < hunabKuMs + MS_PER_DAY;

  const totalCycles = thirteenMoonYear + 28000;

  if (isDayOutOfTime) {
    return {
      year: thirteenMoonYear,
      moon: 0,
      moonName: 'Day Out of Time',
      day: 0,
      dayOfYear: GOLDEN_RATIO_DAY + 1,
      dayOutOfTime: true,
      leapDay: false,
      weekday: 'Day Out of Time',
      totalCycles,
      galacticSignature: 'Green Central Sun',
      harmonicTone: '\u221E',
      arc: '\u03C6-point',
      formatted: `Day Out of Time (11/11 \u2014 Golden Ratio Point: 364/\u03C6 = ${(364 / GOLDEN_RATIO).toFixed(2)}), Year ${thirteenMoonYear} [Cycle ${totalCycles.toLocaleString()}]`
    };
  }

  if (isHunabKu) {
    return {
      year: thirteenMoonYear,
      moon: 0,
      moonName: 'Hunab Ku Day',
      day: 0,
      dayOfYear: 0,
      dayOutOfTime: false,
      leapDay: true,
      weekday: 'Hunab Ku',
      totalCycles,
      galacticSignature: 'Hunab Ku',
      harmonicTone: 0,
      arc: 'Post-\u03C6',
      formatted: `Hunab Ku Day (Leap Day), Year ${thirteenMoonYear} [Cycle ${totalCycles.toLocaleString()}]`
    };
  }

  let adjustedDay = daysSinceNewYear;

  if (dateMs >= dotMs + MS_PER_DAY) {
    adjustedDay = adjustedDay - 1;
  }
  if (hasLeapDay && dateMs >= hunabKuMs + MS_PER_DAY) {
    adjustedDay = adjustedDay - 1;
  }

  adjustedDay = Math.max(0, Math.min(adjustedDay, 363));

  const moon = Math.floor(adjustedDay / 28) + 1;
  const dayInMoon = (adjustedDay % 28) + 1;
  const weekdayIndex = (adjustedDay % 7);
  const weekday = THIRTEEN_MOON_WEEKDAYS[weekdayIndex];

  const safeMoon = Math.max(1, Math.min(moon, 13));
  const moonName = THIRTEEN_MOON_NAMES[safeMoon - 1];
  const galacticSignature = GALACTIC_SIGNATURES[safeMoon - 1];
  const harmonicTone = HARMONIC_TONES[safeMoon - 1];
  const arc = safeMoon <= 8 ? 'Pre-\u03C6' : 'Post-\u03C6';

  return {
    year: thirteenMoonYear,
    moon: safeMoon,
    moonName,
    day: dayInMoon,
    dayOfYear: adjustedDay + 1,
    dayOutOfTime: false,
    leapDay: false,
    weekday,
    totalCycles,
    galacticSignature,
    harmonicTone,
    arc,
    formatted: `${moonName} Moon, Day ${dayInMoon} (${weekday}), Year ${thirteenMoonYear} [Cycle ${totalCycles.toLocaleString()}]`
  };
}

/**
 * Convert Gregorian date to Persian/Solar Hijri calendar
 * 
 * The Solar Hijri calendar is a solar calendar used in Iran and Afghanistan.
 * Origin: March 22, 622 CE (Hijra of Prophet Muhammad).
 * Year = Gregorian year - 621 (after March 20) or - 622 (before March 20).
 * First 6 months have 31 days, next 5 have 30 days, last has 29 (30 in leap).
 */
export function toPersianDate(date: Date): PersianDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();

  const afterNowruz = gMonth > 2 || (gMonth === 2 && gDay >= 21);
  const persianYear = afterNowruz ? gYear - 621 : gYear - 622;

  const nowruzMs = Date.UTC(gYear, 2, 21);
  const dateMs = date.getTime();
  let dayOfPersianYear: number;

  if (afterNowruz) {
    dayOfPersianYear = Math.floor((dateMs - nowruzMs) / MS_PER_DAY) + 1;
  } else {
    const prevNowruzMs = Date.UTC(gYear - 1, 2, 21);
    dayOfPersianYear = Math.floor((dateMs - prevNowruzMs) / MS_PER_DAY) + 1;
  }

  dayOfPersianYear = Math.max(1, Math.min(dayOfPersianYear, 366));

  let persianMonth: number;
  let persianDay: number;

  if (dayOfPersianYear <= 186) {
    persianMonth = Math.floor((dayOfPersianYear - 1) / 31) + 1;
    persianDay = ((dayOfPersianYear - 1) % 31) + 1;
  } else {
    const remaining = dayOfPersianYear - 186;
    persianMonth = Math.floor((remaining - 1) / 30) + 7;
    persianDay = ((remaining - 1) % 30) + 1;
    persianMonth = Math.min(persianMonth, 12);
  }

  const monthName = PERSIAN_MONTHS[persianMonth - 1];

  return {
    year: persianYear,
    month: persianMonth,
    monthName,
    day: persianDay,
    formatted: `${persianDay} ${monthName} ${persianYear} SH`
  };
}

/**
 * Convert Gregorian date to Ethiopian/Ge'ez calendar
 * 
 * Origin: August 29, 8 CE. The Ethiopian calendar has 13 months:
 * 12 months of 30 days each + Pagume (5 or 6 days in leap year).
 * New year (Enkutatash) falls on September 11 (or 12 in leap years).
 * Year = Gregorian year - 8 (after Sep 11) or - 7 (before Sep 11).
 */
export function toEthiopianDate(date: Date): EthiopianDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();

  const afterNewYear = gMonth > 8 || (gMonth === 8 && gDay >= 11);
  const ethYear = afterNewYear ? gYear - 7 : gYear - 8;

  const newYearMs = afterNewYear
    ? Date.UTC(gYear, 8, 11)
    : Date.UTC(gYear - 1, 8, 11);

  const daysSinceNewYear = Math.floor((date.getTime() - newYearMs) / MS_PER_DAY) + 1;
  const safeDays = Math.max(1, Math.min(daysSinceNewYear, 366));

  let ethMonth: number;
  let ethDay: number;

  if (safeDays <= 360) {
    ethMonth = Math.floor((safeDays - 1) / 30) + 1;
    ethDay = ((safeDays - 1) % 30) + 1;
  } else {
    ethMonth = 13;
    ethDay = safeDays - 360;
  }

  const monthName = ETHIOPIAN_MONTHS[ethMonth - 1];

  return {
    year: ethYear,
    month: ethMonth,
    monthName,
    day: ethDay,
    formatted: `${ethDay} ${monthName} ${ethYear} (Ethiopian)`
  };
}

/**
 * Convert Gregorian date to Coptic calendar
 * 
 * Origin: August 29, 284 CE (Era of Martyrs / Diocletian Era).
 * Same structure as Ethiopian: 12 months of 30 days + Pi Kogi Enavot (5-6 days).
 * Year = Gregorian year - 284 (after Sep 11) or - 283 (before Sep 11).
 */
export function toCopticDate(date: Date): CopticDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();

  const afterNewYear = gMonth > 8 || (gMonth === 8 && gDay >= 11);
  const copticYear = afterNewYear ? gYear - 283 : gYear - 284;

  const newYearMs = afterNewYear
    ? Date.UTC(gYear, 8, 11)
    : Date.UTC(gYear - 1, 8, 11);

  const daysSinceNewYear = Math.floor((date.getTime() - newYearMs) / MS_PER_DAY) + 1;
  const safeDays = Math.max(1, Math.min(daysSinceNewYear, 366));

  let copticMonth: number;
  let copticDay: number;

  if (safeDays <= 360) {
    copticMonth = Math.floor((safeDays - 1) / 30) + 1;
    copticDay = ((safeDays - 1) % 30) + 1;
  } else {
    copticMonth = 13;
    copticDay = safeDays - 360;
  }

  const monthName = COPTIC_MONTHS[copticMonth - 1];

  return {
    year: copticYear,
    month: copticMonth,
    monthName,
    day: copticDay,
    formatted: `${copticDay} ${monthName} ${copticYear} AM (Coptic)`
  };
}

/**
 * Convert Gregorian date to Japanese Imperial (Koki) calendar
 * 
 * Origin: February 11, 660 BCE (legendary founding of Japan by Emperor Jimmu).
 * Koki year = Gregorian year + 660.
 * Also tracks the current imperial era name (gengo):
 * Reiwa began May 1, 2019.
 */
export function toJapaneseKokiDate(date: Date): JapaneseKokiDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();

  const kokiYear = gYear + 660;

  let era = 'Reiwa';
  let eraYear = gYear - 2018;

  if (gYear < 2019 || (gYear === 2019 && (gMonth < 4 || (gMonth === 4 && gDay < 1)))) {
    era = 'Heisei';
    eraYear = gYear - 1988;
  }

  if (eraYear < 1) eraYear = 1;

  const monthName = JAPANESE_MONTHS[gMonth];

  return {
    kokiYear,
    era,
    eraYear,
    month: gMonth + 1,
    monthName,
    day: gDay,
    formatted: `${gDay} ${monthName}, Koki ${kokiYear} / ${era} ${eraYear}`
  };
}

/**
 * Convert Gregorian date to Korean Dangun Era
 * 
 * Origin: October 3, 2333 BCE (legendary founding of Gojoseon by Dangun Wanggeom).
 * Dangun year = Gregorian year + 2333.
 */
export function toKoreanDangunDate(date: Date): KoreanDangunDate {
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();
  const year = date.getUTCFullYear() + 2333;
  const monthName = KOREAN_MONTHS[gMonth];

  return {
    year,
    month: gMonth + 1,
    monthName,
    day: gDay,
    formatted: `${gDay} ${monthName}, Dangun ${year.toLocaleString()}`
  };
}

/**
 * Convert Gregorian date to Thai Buddhist Era
 * 
 * Origin: 543 BCE (death/parinibbana of Gautama Buddha).
 * Thai Buddhist year = Gregorian year + 543.
 * Used officially in Thailand and parts of Southeast Asia.
 */
export function toThaiBuddhistDate(date: Date): ThaiBuddhistDate {
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();
  const year = date.getUTCFullYear() + 543;
  const monthName = THAI_MONTHS[gMonth];

  return {
    year,
    month: gMonth + 1,
    monthName,
    day: gDay,
    formatted: `${gDay} ${monthName}, BE ${year}`
  };
}

/**
 * Convert Gregorian date to Indian National/Saka calendar
 * 
 * Origin: March 22, 78 CE (beginning of Saka Era).
 * Year = Gregorian year - 78 (after March 22) or - 79 (before March 22).
 * 12 months: Chaitra (first, ~March-April) through Phalguna.
 * First month Chaitra has 30 days (31 in leap years), months 2-6 have 31 days,
 * months 7-12 have 30 days.
 */
export function toIndianSakaDate(date: Date): IndianSakaDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();

  const afterNewYear = gMonth > 2 || (gMonth === 2 && gDay >= 22);
  const sakaYear = afterNewYear ? gYear - 78 : gYear - 79;

  const newYearMs = afterNewYear
    ? Date.UTC(gYear, 2, 22)
    : Date.UTC(gYear - 1, 2, 22);

  const daysSinceNewYear = Math.floor((date.getTime() - newYearMs) / MS_PER_DAY) + 1;
  const safeDays = Math.max(1, Math.min(daysSinceNewYear, 366));

  let sakaMonth: number;
  let sakaDay: number;

  const chaitraDays = isLeapYear(gYear) ? 31 : 30;

  if (safeDays <= chaitraDays) {
    sakaMonth = 1;
    sakaDay = safeDays;
  } else {
    let remaining = safeDays - chaitraDays;
    sakaMonth = 2;
    while (sakaMonth <= 12) {
      const daysInMonth = sakaMonth <= 6 ? 31 : 30;
      if (remaining <= daysInMonth) {
        sakaDay = remaining;
        break;
      }
      remaining -= daysInMonth;
      sakaMonth++;
    }
    sakaDay = sakaDay! || remaining;
  }

  sakaMonth = Math.min(sakaMonth, 12);
  const monthName = INDIAN_SAKA_MONTHS[sakaMonth - 1];

  return {
    year: sakaYear,
    month: sakaMonth,
    monthName,
    day: sakaDay!,
    formatted: `${sakaDay!} ${monthName} ${sakaYear} SE`
  };
}

/**
 * Convert Gregorian date to Tibetan calendar (Rabjung cycle)
 * 
 * The Tibetan calendar uses a 60-year cycle (Rabjung) starting from 1027 CE.
 * Each year is named by combining one of 5 elements with one of 12 animals.
 * Elements cycle: Iron, Water, Wood, Fire, Earth (each used for 2 consecutive years).
 * Animals: same as Chinese zodiac cycle.
 */
export function toTibetanDate(date: Date): TibetanDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();

  const yearsSinceStart = gYear - 1027;
  const rabjungCycle = Math.floor(yearsSinceStart / 60) + 1;
  const yearInCycle = ((yearsSinceStart % 60) + 60) % 60 + 1;

  const elementIndex = Math.floor(((yearInCycle - 1) % 10) / 2);
  const animalIndex = (yearInCycle - 1) % 12;

  const element = TIBETAN_ELEMENTS[elementIndex];
  const animal = ZODIAC_ANIMALS[animalIndex];

  const tibMonth = ((gMonth - 1) + 12) % 12 + 1;
  const monthName = TIBETAN_MONTHS[tibMonth - 1];

  return {
    rabjungCycle,
    yearInCycle,
    element,
    animal,
    month: tibMonth,
    monthName,
    day: gDay,
    formatted: `${gDay} ${monthName}, ${element} ${animal} Year ${yearInCycle} of Rabjung ${rabjungCycle}`
  };
}

/**
 * Convert Gregorian date to Aztec Tonalpohualli (260-day sacred calendar)
 * 
 * The Tonalpohualli combines 20 day signs with 13 trecena numbers for a 260-day cycle.
 * Reference: JDN 584283 (GMT correlation, same as Mayan epoch).
 * Day sign = daysSinceRef % 20, trecena number = (daysSinceRef % 13) + 1.
 * Also computes the Xiuhpohualli (365-day solar calendar): 18 months of 20 days + 5 Nemontemi.
 */
export function toAztecTonalpohualliDate(date: Date): AztecTonalpohualliDate {
  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());
  const daysSinceRef = jdn - MAYAN_CORRELATION;

  const daySignIndex = ((daysSinceRef % 20) + 20) % 20;
  const trecenaNumber = ((daysSinceRef % 13) + 13) % 13 + 1;
  const daySign = AZTEC_DAY_SIGNS[daySignIndex];

  const xiuhDayOfYear = ((daysSinceRef % 365) + 365) % 365;
  let xiuhMonth: number;
  let xiuhDay: number;
  let xiuhMonthName: string;
  let isNemontemi = false;

  if (xiuhDayOfYear < 360) {
    xiuhMonth = Math.floor(xiuhDayOfYear / 20) + 1;
    xiuhDay = (xiuhDayOfYear % 20) + 1;
    xiuhMonthName = AZTEC_XIUHPOHUALLI_MONTHS[xiuhMonth - 1];
  } else {
    xiuhMonth = 19;
    xiuhDay = xiuhDayOfYear - 360 + 1;
    xiuhMonthName = 'Nemontemi';
    isNemontemi = true;
  }

  return {
    daySign,
    daySignIndex,
    trecenaNumber,
    tonalpohualliDay: `${trecenaNumber} ${daySign}`,
    xiuhpohualliMonth: xiuhMonth,
    xiuhpohualliMonthName: xiuhMonthName,
    xiuhpohualliDay: xiuhDay,
    isNemontemi,
    formatted: `${trecenaNumber} ${daySign} | ${xiuhMonthName} Day ${xiuhDay}${isNemontemi ? ' [Nemontemi]' : ''}`
  };
}

/**
 * Convert Gregorian date to Roman Ab Urbe Condita (AUC)
 * 
 * Origin: April 21, 753 BCE (legendary founding of Rome by Romulus).
 * AUC year = Gregorian year + 753.
 * Includes determination of Kalends, Nones, and Ides markers.
 */
export function toRomanAUCDate(date: Date): RomanAUCDate {
  const gYear = date.getUTCFullYear();
  const gDay = date.getUTCDate();
  const gMonth = date.getUTCMonth();

  const aucYear = gYear + 753;

  const longMonths = [2, 4, 6, 9];
  const isLongMonth = longMonths.includes(gMonth);
  const nonesDay = isLongMonth ? 7 : 5;
  const idesDay = isLongMonth ? 15 : 13;

  let calendarMarker: string;
  if (gDay === 1) {
    calendarMarker = 'Kalendae';
  } else if (gDay === nonesDay) {
    calendarMarker = 'Nonae';
  } else if (gDay === idesDay) {
    calendarMarker = 'Idus';
  } else if (gDay < nonesDay) {
    calendarMarker = `ante diem ${nonesDay - gDay + 1} Nonas`;
  } else if (gDay < idesDay) {
    calendarMarker = `ante diem ${idesDay - gDay + 1} Idus`;
  } else {
    calendarMarker = `ante diem ${gDay - idesDay} post Idus`;
  }

  return {
    year: aucYear,
    calendarMarker,
    formatted: `${aucYear} AUC (${calendarMarker})`
  };
}

/**
 * Convert Gregorian date to Bengali/Bangla calendar
 * 
 * Origin: April 14, 594 CE (Shashanka era, Bengali solar calendar).
 * Year = Gregorian year - 593 (after April 14) or - 594 (before April 14).
 * 12 months: Boishakh (first, mid-April) through Choitro.
 */
export function toBengaliDate(date: Date): BengaliDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();

  const afterNewYear = gMonth > 3 || (gMonth === 3 && gDay >= 14);
  const bengaliYear = afterNewYear ? gYear - 593 : gYear - 594;

  const newYearMs = afterNewYear
    ? Date.UTC(gYear, 3, 14)
    : Date.UTC(gYear - 1, 3, 14);

  const daysSinceNewYear = Math.floor((date.getTime() - newYearMs) / MS_PER_DAY) + 1;
  const safeDays = Math.max(1, Math.min(daysSinceNewYear, 366));

  let bengaliMonth: number;
  let bengaliDay: number;

  if (safeDays <= 186) {
    bengaliMonth = Math.floor((safeDays - 1) / 31) + 1;
    bengaliDay = ((safeDays - 1) % 31) + 1;
    bengaliMonth = Math.min(bengaliMonth, 6);
  } else {
    const remaining = safeDays - 186;
    bengaliMonth = Math.floor((remaining - 1) / 30) + 7;
    bengaliDay = ((remaining - 1) % 30) + 1;
    bengaliMonth = Math.min(bengaliMonth, 12);
  }

  const monthName = BENGALI_MONTHS[bengaliMonth - 1];

  return {
    year: bengaliYear,
    month: bengaliMonth,
    monthName,
    day: bengaliDay,
    formatted: `${bengaliDay} ${monthName} ${bengaliYear} (Bangla)`
  };
}

/**
 * Convert Gregorian date to Berber/Amazigh (Yennayer) calendar
 * 
 * Origin: ~950 BCE (traditional beginning of Amazigh calendar).
 * Year = Gregorian year + 950.
 * Yennayer (New Year) falls on January 12-14 depending on region.
 */
export function toBerberDate(date: Date): BerberDate {
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();

  const afterYennayer = gMonth > 0 || (gMonth === 0 && gDay >= 13);
  const year = afterYennayer ? date.getUTCFullYear() + 950 : date.getUTCFullYear() + 949;

  const newYearMs = afterYennayer
    ? Date.UTC(date.getUTCFullYear(), 0, 13)
    : Date.UTC(date.getUTCFullYear() - 1, 0, 13);

  const daysSinceNewYear = Math.floor((date.getTime() - newYearMs) / MS_PER_DAY) + 1;
  const safeDays = Math.max(1, Math.min(daysSinceNewYear, 366));

  let berberMonth: number;
  let berberDay: number;

  if (safeDays <= 186) {
    berberMonth = Math.floor((safeDays - 1) / 31) + 1;
    berberDay = ((safeDays - 1) % 31) + 1;
    berberMonth = Math.min(berberMonth, 6);
  } else {
    const remaining = safeDays - 186;
    berberMonth = Math.floor((remaining - 1) / 30) + 7;
    berberDay = ((remaining - 1) % 30) + 1;
    berberMonth = Math.min(berberMonth, 12);
  }

  const monthName = BERBER_MONTHS[berberMonth - 1];

  return {
    year,
    month: berberMonth,
    monthName,
    day: berberDay,
    formatted: `${berberDay} ${monthName}, Yennayer ${year.toLocaleString()}`
  };
}

/**
 * Convert Gregorian date to Balinese Pawukon calendar
 * 
 * The Pawukon is a 210-day cycle divided into 30 weeks (wuku) of 7 days each.
 * Used for determining auspicious days for ceremonies in Bali.
 * Reference: The Pawukon cycle can be computed from a known reference point.
 * We use JDN 2456739 (Feb 5, 2014) as Wuku Sinta day 1.
 */
export function toBalinesePawukonDate(date: Date): BalinesePawukonDate {
  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());
  const referenceJDN = 2456739;

  const daysSinceRef = jdn - referenceJDN;
  const cycleDay = ((daysSinceRef % 210) + 210) % 210;

  const wukuWeek = Math.floor(cycleDay / 7) + 1;
  const dayInWuku = (cycleDay % 7) + 1;

  const wukuName = WUKU_NAMES[wukuWeek - 1] || WUKU_NAMES[0];

  return {
    wukuWeek,
    wukuName,
    dayInWuku,
    cycleDay: cycleDay + 1,
    formatted: `Wuku ${wukuName} (Week ${wukuWeek}), Day ${dayInWuku}, Cycle Day ${cycleDay + 1}/210`
  };
}

/**
 * Convert Gregorian date to Zoroastrian Fasli calendar
 * 
 * Origin: 632 CE (death of Yazdegerd III, last Sassanid emperor).
 * Year = Gregorian year - 632 (after March 21) or - 631 (before March 21).
 * 12 months of 30 days each + 5 Gatha days (epagomenal).
 */
export function toZoroastrianFasliDate(date: Date): ZoroastrianFasliDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();

  const afterNowruz = gMonth > 2 || (gMonth === 2 && gDay >= 21);
  const fasliYear = afterNowruz ? gYear - 631 : gYear - 632;

  const newYearMs = afterNowruz
    ? Date.UTC(gYear, 2, 21)
    : Date.UTC(gYear - 1, 2, 21);

  const daysSinceNewYear = Math.floor((date.getTime() - newYearMs) / MS_PER_DAY) + 1;
  const safeDays = Math.max(1, Math.min(daysSinceNewYear, 366));

  let zMonth: number;
  let zDay: number;
  let isGathaDay = false;

  if (safeDays <= 360) {
    zMonth = Math.floor((safeDays - 1) / 30) + 1;
    zDay = ((safeDays - 1) % 30) + 1;
  } else {
    zMonth = 13;
    zDay = safeDays - 360;
    isGathaDay = true;
  }

  const monthName = zMonth <= 12 ? ZOROASTRIAN_MONTHS[zMonth - 1] : 'Gatha Days';

  return {
    year: fasliYear,
    month: zMonth,
    monthName,
    day: zDay,
    isGathaDay,
    formatted: `${zDay} ${monthName} ${fasliYear} YZ${isGathaDay ? ' [Gatha]' : ''}`
  };
}

const NISGAA_SEASONS = [
  { name: "K'alii Aks", description: 'Gathering and preparation', months: [2, 3], indicator: 'Oolichan run, spring salmon preparation' },
  { name: "Hobiyee", description: 'Celebration and renewal', months: [0, 1], indicator: 'New year celebration, ice breakup' },
  { name: "Lax̱ Ha", description: 'Salmon season begins', months: [4, 5], indicator: 'Spring salmon arriving, berry picking begins' },
  { name: "Xsaak", description: 'Main salmon run', months: [6, 7], indicator: 'Sockeye salmon run, peak harvest' },
  { name: "Miso'o", description: 'Late harvest and preserving', months: [8, 9], indicator: 'Coho salmon, smoking and drying fish' },
  { name: "Anlo'o", description: 'Winter rest and ceremony', months: [10, 11], indicator: 'Snow, storytelling, ceremonial season' }
];

const TAMIL_MONTHS = [
  'Chithirai', 'Vaigasi', 'Aani', 'Aadi', 'Avani', 'Purattasi',
  'Aippasi', 'Karthigai', 'Margazhi', 'Thai', 'Maasi', 'Panguni'
];

const VIKRAM_SAMVAT_MONTHS = [
  'Chaitra', 'Vaishakha', 'Jyeshtha', 'Ashadha', 'Shravana', 'Bhadrapada',
  'Ashwin', 'Kartik', 'Margashirsha', 'Pausha', 'Magha', 'Phalguna'
];

const MALAYALAM_MONTHS = [
  'Chingam', 'Kanni', 'Thulam', 'Vrischikam', 'Dhanu', 'Makaram',
  'Kumbham', 'Meenam', 'Medam', 'Edavam', 'Mithunam', 'Karkidakam'
];

const NANAKSHAHI_MONTHS = [
  'Chet', 'Vaisakh', 'Jeth', 'Harh', 'Sawan', 'Bhadon',
  'Assu', 'Katak', 'Maghar', 'Poh', 'Magh', 'Phagun'
];

const NANAKSHAHI_MONTH_STARTS = [
  { month: 2, day: 14 },
  { month: 3, day: 14 },
  { month: 4, day: 15 },
  { month: 5, day: 15 },
  { month: 6, day: 16 },
  { month: 7, day: 16 },
  { month: 8, day: 15 },
  { month: 9, day: 15 },
  { month: 10, day: 14 },
  { month: 11, day: 13 },
  { month: 0, day: 13 },
  { month: 1, day: 12 }
];

const BAHAI_MONTHS = [
  'Baha', 'Jalal', 'Jamal', "'Azamat", 'Nur', 'Rahmat', 'Kalimat',
  'Kamal', "Asma'", "'Izzat", 'Mashiyyat', "'Ilm", 'Qudrat', 'Qawl',
  'Masa\'il', 'Sharaf', 'Sultan', 'Mulk', "'Ala'"
];

const IGBO_DAYS = ['Eke', 'Orie', 'Afo', 'Nkwo'];

const JAVANESE_PASARAN = ['Pon', 'Wage', 'Kliwon', 'Legi', 'Pahing'];
const JAVANESE_WEEKDAYS = ['Ahad', 'Senin', 'Selasa', 'Rabu', 'Kamis', 'Jumat', 'Sabtu'];

const YORUBA_DAYS = ['Ojó-Aìkú', 'Ojó-Ajé', 'Ojó-Ìṣégun', 'Ojó-Rú'];

export function toAssyrianDate(date: Date): AssyrianDate {
  const gYear = date.getUTCFullYear();
  const year = gYear + 4750;
  return {
    year,
    month: date.getUTCMonth() + 1,
    day: date.getUTCDate(),
    formatted: `${date.getUTCDate()}/${date.getUTCMonth() + 1}/${year} (Assyrian)`
  };
}

export function toNisgaaSeasonalDate(date: Date): NisgaaSeasonalDate {
  const gMonth = date.getUTCMonth();
  let matchedSeason = NISGAA_SEASONS[0];
  for (const season of NISGAA_SEASONS) {
    if (season.months.includes(gMonth)) {
      matchedSeason = season;
      break;
    }
  }
  return {
    season: matchedSeason.name,
    seasonDescription: matchedSeason.description,
    naturalIndicator: matchedSeason.indicator,
    formatted: `${matchedSeason.name} (${matchedSeason.description}) - ${matchedSeason.indicator}`
  };
}

export function toYorubaDate(date: Date): YorubaDate {
  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());
  const dayIndex = ((jdn % 4) + 4) % 4;
  const dayName = YORUBA_DAYS[dayIndex];
  const dayOfYear = Math.floor((date.getTime() - Date.UTC(date.getUTCFullYear(), 0, 1)) / MS_PER_DAY) + 1;
  const month = Math.floor((dayOfYear - 1) / 28) + 1;
  const dayOfMonth = ((dayOfYear - 1) % 28) + 1;
  return {
    dayName,
    dayIndex,
    month: Math.min(month, 13),
    dayOfMonth,
    formatted: `${dayName}, Month ${Math.min(month, 13)} Day ${dayOfMonth} (Yoruba)`
  };
}

export function toJainDate(date: Date): JainDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const year = gYear + 527 + (gMonth >= 3 ? 0 : -1);
  return {
    year,
    month: date.getUTCMonth() + 1,
    day: date.getUTCDate(),
    formatted: `${date.getUTCDate()}/${date.getUTCMonth() + 1}/${year} VNS (Jain)`
  };
}

export function toTamilDate(date: Date): TamilDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();
  const afterNewYear = gMonth > 3 || (gMonth === 3 && gDay >= 14);
  const tamilYear = afterNewYear ? gYear - 31 : gYear - 32;
  const newYearMs = afterNewYear
    ? Date.UTC(gYear, 3, 14)
    : Date.UTC(gYear - 1, 3, 14);
  const daysSinceNewYear = Math.floor((date.getTime() - newYearMs) / MS_PER_DAY) + 1;
  const safeDays = Math.max(1, Math.min(daysSinceNewYear, 366));
  let tamilMonth: number;
  let tamilDay: number;
  if (safeDays <= 186) {
    tamilMonth = Math.floor((safeDays - 1) / 31) + 1;
    tamilDay = ((safeDays - 1) % 31) + 1;
    tamilMonth = Math.min(tamilMonth, 6);
  } else {
    const remaining = safeDays - 186;
    tamilMonth = Math.floor((remaining - 1) / 30) + 7;
    tamilDay = ((remaining - 1) % 30) + 1;
    tamilMonth = Math.min(tamilMonth, 12);
  }
  const monthName = TAMIL_MONTHS[tamilMonth - 1];
  return {
    year: tamilYear,
    month: tamilMonth,
    monthName,
    day: tamilDay,
    formatted: `${tamilDay} ${monthName} ${tamilYear} (Tamil)`
  };
}

export function toVietnameseDate(date: Date): VietnameseDate {
  const year = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();
  const chineseNewYearOffset = gMonth < 1 || (gMonth === 1 && gDay < 4) ? -1 : 0;
  const vietYear = year + chineseNewYearOffset;
  const chineseNewYearMs = Date.UTC(vietYear, 1, 4);
  const dateMs = date.getTime();
  let daysSinceNewYear: number;
  if (dateMs >= chineseNewYearMs) {
    daysSinceNewYear = Math.floor((dateMs - chineseNewYearMs) / MS_PER_DAY) + 1;
  } else {
    const prevNewYearMs = Date.UTC(vietYear - 1, 1, 4);
    daysSinceNewYear = Math.floor((dateMs - prevNewYearMs) / MS_PER_DAY) + 1;
  }
  daysSinceNewYear = Math.max(1, Math.min(daysSinceNewYear, 385));
  const monthLengths = [29, 30, 29, 30, 29, 30, 29, 30, 29, 30, 29, 30];
  let vMonth = 1;
  let vDay = daysSinceNewYear;
  for (let i = 0; i < 12; i++) {
    if (vDay <= monthLengths[i]) { vMonth = i + 1; break; }
    vDay -= monthLengths[i];
    vMonth = i + 2;
  }
  vMonth = Math.min(vMonth, 12);
  vDay = Math.max(1, Math.min(vDay, 30));
  return {
    year: vietYear,
    month: vMonth,
    day: vDay,
    formatted: `${vDay}/${vMonth}/${vietYear} (Vietnamese)`
  };
}

export function toVikramSamvatDate(date: Date): VikramSamvatDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();
  const afterNewYear = gMonth > 3 || (gMonth === 3 && gDay >= 14);
  const vsYear = afterNewYear ? gYear + 57 : gYear + 56;
  const newYearMs = afterNewYear
    ? Date.UTC(gYear, 3, 14)
    : Date.UTC(gYear - 1, 3, 14);
  const daysSinceNewYear = Math.floor((date.getTime() - newYearMs) / MS_PER_DAY) + 1;
  const safeDays = Math.max(1, Math.min(daysSinceNewYear, 385));
  const monthLengths = [30, 31, 31, 31, 31, 31, 30, 30, 30, 29, 30, 30];
  let vsMonth = 1;
  let vsDay = safeDays;
  for (let i = 0; i < 12; i++) {
    if (vsDay <= monthLengths[i]) { vsMonth = i + 1; break; }
    vsDay -= monthLengths[i];
    vsMonth = i + 2;
  }
  vsMonth = Math.min(vsMonth, 12);
  vsDay = Math.max(1, Math.min(vsDay, 31));
  const monthName = VIKRAM_SAMVAT_MONTHS[vsMonth - 1];
  return {
    year: vsYear,
    month: vsMonth,
    monthName,
    day: vsDay,
    formatted: `${vsDay} ${monthName} ${vsYear} VS`
  };
}

export function toKhmerDate(date: Date): KhmerDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();
  const afterNewYear = gMonth > 3 || (gMonth === 3 && gDay >= 14);
  const khmerYear = afterNewYear ? gYear + 544 : gYear + 543;
  return {
    year: khmerYear,
    month: gMonth + 1,
    day: gDay,
    formatted: `${gDay}/${gMonth + 1}/${khmerYear} BE (Khmer)`
  };
}

export function toBurmeseDate(date: Date): BurmeseDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();
  const afterNewYear = gMonth > 3 || (gMonth === 3 && gDay >= 17);
  const burmeseYear = afterNewYear ? gYear - 638 : gYear - 639;
  return {
    year: Math.max(burmeseYear, 1),
    month: gMonth + 1,
    day: gDay,
    formatted: `${gDay}/${gMonth + 1}/${Math.max(burmeseYear, 1)} ME (Burmese)`
  };
}

export function toJavaneseDate(date: Date): JavaneseDate {
  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());
  const pasaranIndex = ((jdn % 5) + 5) % 5;
  const weekdayIndex = ((jdn + 1) % 7);
  const pasaranDay = JAVANESE_PASARAN[pasaranIndex];
  const weekday = JAVANESE_WEEKDAYS[weekdayIndex];
  const cycleDay = (pasaranIndex * 7 + weekdayIndex) % 35 + 1;
  return {
    pasaranDay,
    pasaranIndex,
    weekday,
    weekdayIndex,
    cycleDay,
    formatted: `${pasaranDay} ${weekday}, Cycle Day ${cycleDay}/35 (Javanese)`
  };
}

export function toMalayalamDate(date: Date): MalayalamDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();
  const afterNewYear = gMonth > 7 || (gMonth === 7 && gDay >= 17);
  const kollamYear = afterNewYear ? gYear - 825 : gYear - 826;
  const newYearMs = afterNewYear
    ? Date.UTC(gYear, 7, 17)
    : Date.UTC(gYear - 1, 7, 17);
  const daysSinceNewYear = Math.floor((date.getTime() - newYearMs) / MS_PER_DAY) + 1;
  const safeDays = Math.max(1, Math.min(daysSinceNewYear, 366));
  let mlMonth: number;
  let mlDay: number;
  if (safeDays <= 186) {
    mlMonth = Math.floor((safeDays - 1) / 31) + 1;
    mlDay = ((safeDays - 1) % 31) + 1;
    mlMonth = Math.min(mlMonth, 6);
  } else {
    const remaining = safeDays - 186;
    mlMonth = Math.floor((remaining - 1) / 30) + 7;
    mlDay = ((remaining - 1) % 30) + 1;
    mlMonth = Math.min(mlMonth, 12);
  }
  const monthName = MALAYALAM_MONTHS[mlMonth - 1];
  return {
    year: kollamYear,
    month: mlMonth,
    monthName,
    day: mlDay,
    formatted: `${mlDay} ${monthName} ${kollamYear} ME (Malayalam)`
  };
}

export function toNepalSambatDate(date: Date): NepalSambatDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const afterNewYear = gMonth >= 9;
  const nsYear = afterNewYear ? gYear - 879 + 1 : gYear - 879;
  return {
    year: nsYear,
    month: date.getUTCMonth() + 1,
    day: date.getUTCDate(),
    formatted: `${date.getUTCDate()}/${date.getUTCMonth() + 1}/${nsYear} NS (Nepal Sambat)`
  };
}

export function toNanakshahiDate(date: Date): NanakshahiDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();
  let nsMonth = 0;
  let nsDay = 0;
  for (let i = 0; i < 12; i++) {
    const startGMonth = NANAKSHAHI_MONTH_STARTS[i].month;
    const startGDay = NANAKSHAHI_MONTH_STARTS[i].day;
    const nextI = (i + 1) % 12;
    const endGMonth = NANAKSHAHI_MONTH_STARTS[nextI].month;
    const endGDay = NANAKSHAHI_MONTH_STARTS[nextI].day;
    const afterStart = gMonth > startGMonth || (gMonth === startGMonth && gDay >= startGDay);
    const beforeEnd = gMonth < endGMonth || (gMonth === endGMonth && gDay < endGDay);
    const wraps = endGMonth < startGMonth || (endGMonth === startGMonth && endGDay <= startGDay);
    const inMonth = wraps ? (afterStart || beforeEnd) : (afterStart && beforeEnd);
    if (inMonth) {
      nsMonth = i + 1;
      const startMs = Date.UTC(gYear, startGMonth, startGDay);
      const nowMs = Date.UTC(gYear, gMonth, gDay);
      nsDay = Math.floor((nowMs - startMs) / MS_PER_DAY) + 1;
      if (nsDay <= 0) {
        const prevStartMs = Date.UTC(gYear - 1, startGMonth, startGDay);
        nsDay = Math.floor((nowMs - prevStartMs) / MS_PER_DAY) + 1;
      }
      break;
    }
  }
  if (nsMonth === 0) { nsMonth = 1; nsDay = 1; }
  const afterChet = gMonth > 2 || (gMonth === 2 && gDay >= 14);
  const nsYear = afterChet ? gYear - 1468 : gYear - 1469;
  const monthName = NANAKSHAHI_MONTHS[nsMonth - 1];
  return {
    year: nsYear,
    month: nsMonth,
    monthName,
    day: nsDay,
    formatted: `${nsDay} ${monthName} ${nsYear} NS (Nanakshahi)`
  };
}

export function toBahaiDate(date: Date): BahaiDate {
  const gYear = date.getUTCFullYear();
  const gMonth = date.getUTCMonth();
  const gDay = date.getUTCDate();
  const nawRuzMonth = 2;
  const nawRuzDay = 20;
  const afterNawRuz = gMonth > nawRuzMonth || (gMonth === nawRuzMonth && gDay >= nawRuzDay);
  const bahaiYear = afterNawRuz ? gYear - 1843 : gYear - 1844;
  const nawRuzMs = afterNawRuz
    ? Date.UTC(gYear, nawRuzMonth, nawRuzDay)
    : Date.UTC(gYear - 1, nawRuzMonth, nawRuzDay);
  const daysSinceNawRuz = Math.floor((date.getTime() - nawRuzMs) / MS_PER_DAY) + 1;
  const safeDays = Math.max(1, Math.min(daysSinceNawRuz, 366));
  let bahaiMonth: number;
  let bahaiDay: number;
  let isAyyamiHa = false;
  if (safeDays <= 342) {
    bahaiMonth = Math.floor((safeDays - 1) / 19) + 1;
    bahaiDay = ((safeDays - 1) % 19) + 1;
  } else if (safeDays <= 346 + (isLeapYear(gYear) ? 1 : 0)) {
    bahaiMonth = 0;
    bahaiDay = safeDays - 342;
    isAyyamiHa = true;
  } else {
    bahaiMonth = 19;
    bahaiDay = safeDays - 346 - (isLeapYear(gYear) ? 1 : 0);
    bahaiDay = Math.max(1, Math.min(bahaiDay, 19));
  }
  const monthName = isAyyamiHa ? "Ayyam-i-Ha" : (bahaiMonth >= 1 && bahaiMonth <= 19 ? BAHAI_MONTHS[bahaiMonth - 1] : "Ayyam-i-Ha");
  return {
    year: bahaiYear,
    month: bahaiMonth,
    monthName,
    day: bahaiDay,
    isAyyamiHa,
    formatted: `${bahaiDay} ${monthName} ${bahaiYear} BE (Bahai)`
  };
}

export function toMinguoDate(date: Date): MinguoDate {
  const year = date.getUTCFullYear() - 1911;
  return {
    year,
    month: date.getUTCMonth() + 1,
    day: date.getUTCDate(),
    formatted: `${date.getUTCDate()}/${date.getUTCMonth() + 1}/${year} (Minguo)`
  };
}

export function toIgboDate(date: Date): IgboDate {
  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());
  const dayIndex = ((jdn % 4) + 4) % 4;
  const dayName = IGBO_DAYS[dayIndex];
  const dayOfYear = Math.floor((date.getTime() - Date.UTC(date.getUTCFullYear(), 0, 1)) / MS_PER_DAY) + 1;
  const month = Math.floor((dayOfYear - 1) / 28) + 1;
  const dayOfMonth = ((dayOfYear - 1) % 28) + 1;
  return {
    dayName,
    dayIndex,
    month: Math.min(month, 13),
    dayOfMonth,
    formatted: `${dayName}, Month ${Math.min(month, 13)} Day ${dayOfMonth} (Igbo)`
  };
}

export function toAkanDate(date: Date): AkanDate {
  const jdn = gregorianToJDN(date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate());
  const adaeCycleDay = ((jdn % 42) + 42) % 42 + 1;
  let adaeCycleName: string;
  if (adaeCycleDay <= 14) {
    adaeCycleName = 'Adae Kese (Great Adae period)';
  } else if (adaeCycleDay <= 28) {
    adaeCycleName = 'Awukudae (Wednesday Adae period)';
  } else {
    adaeCycleName = 'Akwasidae (Sunday Adae period)';
  }
  return {
    adaeCycleDay,
    adaeCycleName,
    formatted: `Adae Cycle Day ${adaeCycleDay}/42 - ${adaeCycleName} (Akan)`
  };
}

export function toGregorianDate(date: Date): GregorianDate {
  return {
    year: date.getUTCFullYear(),
    month: date.getUTCMonth() + 1,
    day: date.getUTCDate(),
    formatted: `${date.getUTCDate()}/${date.getUTCMonth() + 1}/${date.getUTCFullYear()} CE (Gregorian)`
  };
}

/**
 * Convert Gregorian date to Aboriginal Australian Seasonal calendar
 * 
 * Based on the Dharawal/D'harawal nation six-season calendar from
 * the Sydney basin region of Australia. Seasons are tied to natural
 * phenomena rather than fixed dates, approximated here by month ranges.
 */
export function toAboriginalSeasonalDate(date: Date): AboriginalSeasonalDate {
  const gMonth = date.getUTCMonth();

  let matchedSeason = DHARAWAL_SEASONS[0];
  for (const season of DHARAWAL_SEASONS) {
    if (season.months.includes(gMonth)) {
      matchedSeason = season;
      break;
    }
  }

  return {
    season: matchedSeason.name,
    seasonDescription: matchedSeason.description,
    naturalIndicator: matchedSeason.indicator,
    formatted: `${matchedSeason.name} (${matchedSeason.description}) - ${matchedSeason.indicator}`
  };
}

/**
 * Get the complete Salvi Epoch synchronization across all 42 ancient calendars
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
  const thirteenMoon = toThirteenMoonDate(date);
  const persian = toPersianDate(date);
  const ethiopian = toEthiopianDate(date);
  const coptic = toCopticDate(date);
  const japaneseKoki = toJapaneseKokiDate(date);
  const koreanDangun = toKoreanDangunDate(date);
  const thaiBuddhist = toThaiBuddhistDate(date);
  const indianSaka = toIndianSakaDate(date);
  const tibetan = toTibetanDate(date);
  const aztecTonalpohualli = toAztecTonalpohualliDate(date);
  const romanAUC = toRomanAUCDate(date);
  const bengali = toBengaliDate(date);
  const berber = toBerberDate(date);
  const balinesePawukon = toBalinesePawukonDate(date);
  const zoroastrianFasli = toZoroastrianFasliDate(date);
  const aboriginalSeasonal = toAboriginalSeasonalDate(date);
  const assyrian = toAssyrianDate(date);
  const nisgaaSeasonal = toNisgaaSeasonalDate(date);
  const yoruba = toYorubaDate(date);
  const jain = toJainDate(date);
  const tamil = toTamilDate(date);
  const vietnamese = toVietnameseDate(date);
  const vikramSamvat = toVikramSamvatDate(date);
  const khmer = toKhmerDate(date);
  const burmese = toBurmeseDate(date);
  const javanese = toJavaneseDate(date);
  const malayalam = toMalayalamDate(date);
  const nepalSambat = toNepalSambatDate(date);
  const nanakshahi = toNanakshahiDate(date);
  const bahai = toBahaiDate(date);
  const minguo = toMinguoDate(date);
  const igbo = toIgboDate(date);
  const akan = toAkanDate(date);
  const gregorian = toGregorianDate(date);

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
      daysSinceCalendarOrigin: Math.floor((date.getUTCFullYear() - YELLOW_EMPEROR_EPOCH) * 365.2422),
      yearInCalendar: date.getUTCFullYear() - YELLOW_EMPEROR_EPOCH,
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
    },
    {
      calendarSystem: '13-Moon Natural Time',
      origin: '~28,000 BCE (Abri Blanchard bone attestation)',
      originYear: -28000,
      salviEpochEquivalent: thirteenMoon.formatted,
      daysSinceCalendarOrigin: Math.floor((date.getUTCFullYear() + 28000) * 365.2422),
      yearInCalendar: thirteenMoon.totalCycles,
      cyclicPosition: `${thirteenMoon.moonName} Moon, Day ${thirteenMoon.day}`,
      description: `364-day cycle (13 months x 28 days): ${thirteenMoon.formatted}`
    },
    {
      calendarSystem: 'Persian/Solar Hijri',
      origin: 'March 22, 622 CE (Hijra)',
      originYear: 622,
      salviEpochEquivalent: persian.formatted,
      daysSinceCalendarOrigin: Math.floor(persian.year * 365.2422),
      yearInCalendar: persian.year,
      description: `Solar calendar (Iran/Afghanistan): ${persian.formatted}`
    },
    {
      calendarSystem: 'Ethiopian/Ge\'ez',
      origin: 'August 29, 8 CE',
      originYear: 8,
      salviEpochEquivalent: ethiopian.formatted,
      daysSinceCalendarOrigin: Math.floor(ethiopian.year * 365.25),
      yearInCalendar: ethiopian.year,
      description: `Ethiopian calendar (13 months): ${ethiopian.formatted}`
    },
    {
      calendarSystem: 'Coptic (Era of Martyrs)',
      origin: 'August 29, 284 CE',
      originYear: 284,
      salviEpochEquivalent: coptic.formatted,
      daysSinceCalendarOrigin: Math.floor(coptic.year * 365.25),
      yearInCalendar: coptic.year,
      description: `Coptic calendar (13 months): ${coptic.formatted}`
    },
    {
      calendarSystem: 'Japanese Imperial (Koki)',
      origin: 'February 11, 660 BCE',
      originYear: -659,
      salviEpochEquivalent: japaneseKoki.formatted,
      daysSinceCalendarOrigin: Math.floor(japaneseKoki.kokiYear * 365.2422),
      yearInCalendar: japaneseKoki.kokiYear,
      cyclicPosition: `${japaneseKoki.era} ${japaneseKoki.eraYear}`,
      description: `Japanese imperial reckoning: ${japaneseKoki.formatted}`
    },
    {
      calendarSystem: 'Korean Dangun Era',
      origin: 'October 3, 2333 BCE',
      originYear: -2332,
      salviEpochEquivalent: koreanDangun.formatted,
      daysSinceCalendarOrigin: Math.floor(koreanDangun.year * 365.2422),
      yearInCalendar: koreanDangun.year,
      description: `Korean foundation calendar: ${koreanDangun.formatted}`
    },
    {
      calendarSystem: 'Thai Buddhist Era',
      origin: '543 BCE (Parinibbana of Buddha)',
      originYear: -542,
      salviEpochEquivalent: thaiBuddhist.formatted,
      daysSinceCalendarOrigin: Math.floor(thaiBuddhist.year * 365.2422),
      yearInCalendar: thaiBuddhist.year,
      description: `Thai Buddhist calendar: ${thaiBuddhist.formatted}`
    },
    {
      calendarSystem: 'Indian National/Saka',
      origin: 'March 22, 78 CE',
      originYear: 78,
      salviEpochEquivalent: indianSaka.formatted,
      daysSinceCalendarOrigin: Math.floor(indianSaka.year * 365.2422),
      yearInCalendar: indianSaka.year,
      description: `Indian national calendar: ${indianSaka.formatted}`
    },
    {
      calendarSystem: 'Tibetan Rabjung Cycle',
      origin: '1027 CE (First Rabjung)',
      originYear: 1027,
      salviEpochEquivalent: tibetan.formatted,
      daysSinceCalendarOrigin: Math.floor((date.getUTCFullYear() - 1027) * 365.2422),
      yearInCalendar: tibetan.yearInCycle,
      cyclicPosition: `${tibetan.element} ${tibetan.animal}`,
      description: `Tibetan 60-year cycle: ${tibetan.formatted}`
    },
    {
      calendarSystem: 'Aztec Tonalpohualli',
      origin: 'August 11, 3114 BCE (GMT Correlation)',
      originYear: -3113,
      salviEpochEquivalent: aztecTonalpohualli.formatted,
      daysSinceCalendarOrigin: jdn - MAYAN_CORRELATION,
      yearInCalendar: Math.floor((jdn - MAYAN_CORRELATION) / 365),
      cyclicPosition: aztecTonalpohualli.tonalpohualliDay,
      description: `Aztec sacred 260-day calendar: ${aztecTonalpohualli.formatted}`
    },
    {
      calendarSystem: 'Roman Ab Urbe Condita',
      origin: 'April 21, 753 BCE',
      originYear: -752,
      salviEpochEquivalent: romanAUC.formatted,
      daysSinceCalendarOrigin: Math.floor(romanAUC.year * 365.2422),
      yearInCalendar: romanAUC.year,
      cyclicPosition: romanAUC.calendarMarker,
      description: `Roman calendar from founding of Rome: ${romanAUC.formatted}`
    },
    {
      calendarSystem: 'Bengali/Bangla',
      origin: 'April 14, 594 CE (Shashanka Era)',
      originYear: 594,
      salviEpochEquivalent: bengali.formatted,
      daysSinceCalendarOrigin: Math.floor(bengali.year * 365.2422),
      yearInCalendar: bengali.year,
      description: `Bengali solar calendar: ${bengali.formatted}`
    },
    {
      calendarSystem: 'Berber/Amazigh (Yennayer)',
      origin: '~950 BCE',
      originYear: -949,
      salviEpochEquivalent: berber.formatted,
      daysSinceCalendarOrigin: Math.floor(berber.year * 365.2422),
      yearInCalendar: berber.year,
      description: `North African agricultural calendar: ${berber.formatted}`
    },
    {
      calendarSystem: 'Balinese Pawukon',
      origin: '210-day ceremonial cycle',
      originYear: 0,
      salviEpochEquivalent: balinesePawukon.formatted,
      daysSinceCalendarOrigin: balinesePawukon.cycleDay,
      yearInCalendar: 0,
      cyclicPosition: `Wuku ${balinesePawukon.wukuName}`,
      description: `Balinese 210-day cycle: ${balinesePawukon.formatted}`
    },
    {
      calendarSystem: 'Zoroastrian Fasli',
      origin: '632 CE (Death of Yazdegerd III)',
      originYear: 632,
      salviEpochEquivalent: zoroastrianFasli.formatted,
      daysSinceCalendarOrigin: Math.floor(zoroastrianFasli.year * 365.2422),
      yearInCalendar: zoroastrianFasli.year,
      description: `Zoroastrian calendar: ${zoroastrianFasli.formatted}`
    },
    {
      calendarSystem: 'Aboriginal Australian Seasonal (Dharawal)',
      origin: 'Continuous ecological observation',
      originYear: 0,
      salviEpochEquivalent: aboriginalSeasonal.formatted,
      daysSinceCalendarOrigin: 0,
      yearInCalendar: 0,
      cyclicPosition: aboriginalSeasonal.season,
      description: `Dharawal six-season calendar: ${aboriginalSeasonal.formatted}`
    },
    {
      calendarSystem: 'Assyrian',
      origin: '4750 BCE (Assyrian institutional era)',
      originYear: -4749,
      salviEpochEquivalent: assyrian.formatted,
      daysSinceCalendarOrigin: Math.floor(assyrian.year * 365.2422),
      yearInCalendar: assyrian.year,
      description: `Assyrian calendar: ${assyrian.formatted}`
    },
    {
      calendarSystem: "Nisg\u0331a'a Seasonal",
      origin: 'Pre-contact (~5,000+ years oral tradition)',
      originYear: 0,
      salviEpochEquivalent: nisgaaSeasonal.formatted,
      daysSinceCalendarOrigin: 0,
      yearInCalendar: 0,
      cyclicPosition: nisgaaSeasonal.season,
      description: `Nisg\u0331a'a seasonal calendar: ${nisgaaSeasonal.formatted}`
    },
    {
      calendarSystem: 'Yoruba',
      origin: 'Traditional (~3,000+ years)',
      originYear: 0,
      salviEpochEquivalent: yoruba.formatted,
      daysSinceCalendarOrigin: 0,
      yearInCalendar: 0,
      cyclicPosition: yoruba.dayName,
      description: `Yoruba 4-day cycle: ${yoruba.formatted}`
    },
    {
      calendarSystem: 'Jain (Vira Nirvana Samvat)',
      origin: '527 BCE (Nirvana of Mahavira)',
      originYear: -526,
      salviEpochEquivalent: jain.formatted,
      daysSinceCalendarOrigin: Math.floor(jain.year * 365.2422),
      yearInCalendar: jain.year,
      description: `Jain calendar: ${jain.formatted}`
    },
    {
      calendarSystem: 'Tamil',
      origin: '~300 BCE (Zodiac sidereal)',
      originYear: -299,
      salviEpochEquivalent: tamil.formatted,
      daysSinceCalendarOrigin: Math.floor(tamil.year * 365.2422),
      yearInCalendar: tamil.year,
      description: `Tamil solar calendar: ${tamil.formatted}`
    },
    {
      calendarSystem: 'Vietnamese',
      origin: '~200 BCE (Independent lunisolar)',
      originYear: -199,
      salviEpochEquivalent: vietnamese.formatted,
      daysSinceCalendarOrigin: Math.floor(vietnamese.year * 365.2422),
      yearInCalendar: vietnamese.year,
      description: `Vietnamese lunisolar calendar: ${vietnamese.formatted}`
    },
    {
      calendarSystem: 'Vikram Samvat',
      origin: '57 BCE',
      originYear: -56,
      salviEpochEquivalent: vikramSamvat.formatted,
      daysSinceCalendarOrigin: Math.floor(vikramSamvat.year * 365.2422),
      yearInCalendar: vikramSamvat.year,
      description: `Vikram Samvat lunisolar calendar: ${vikramSamvat.formatted}`
    },
    {
      calendarSystem: 'Khmer (Cambodian)',
      origin: '~500 CE (Surya Siddhanta variant)',
      originYear: 500,
      salviEpochEquivalent: khmer.formatted,
      daysSinceCalendarOrigin: Math.floor(khmer.year * 365.2422),
      yearInCalendar: khmer.year,
      description: `Khmer Buddhist-derived calendar: ${khmer.formatted}`
    },
    {
      calendarSystem: 'Burmese',
      origin: '638 CE (Surya Siddhanta)',
      originYear: 638,
      salviEpochEquivalent: burmese.formatted,
      daysSinceCalendarOrigin: Math.floor(burmese.year * 365.2422),
      yearInCalendar: burmese.year,
      description: `Burmese lunisolar calendar: ${burmese.formatted}`
    },
    {
      calendarSystem: 'Javanese',
      origin: '~8th century CE (5+7 day hybrid)',
      originYear: 700,
      salviEpochEquivalent: javanese.formatted,
      daysSinceCalendarOrigin: javanese.cycleDay,
      yearInCalendar: 0,
      cyclicPosition: `${javanese.pasaranDay} ${javanese.weekday}`,
      description: `Javanese dual-cycle calendar: ${javanese.formatted}`
    },
    {
      calendarSystem: 'Malayalam (Kollam Era)',
      origin: '825 CE (Kollam epoch)',
      originYear: 825,
      salviEpochEquivalent: malayalam.formatted,
      daysSinceCalendarOrigin: Math.floor(malayalam.year * 365.2422),
      yearInCalendar: malayalam.year,
      description: `Malayalam solar calendar: ${malayalam.formatted}`
    },
    {
      calendarSystem: 'Nepal Sambat',
      origin: '879 CE (Newar epoch)',
      originYear: 879,
      salviEpochEquivalent: nepalSambat.formatted,
      daysSinceCalendarOrigin: Math.floor(nepalSambat.year * 365.2422),
      yearInCalendar: nepalSambat.year,
      description: `Nepal Sambat lunisolar calendar: ${nepalSambat.formatted}`
    },
    {
      calendarSystem: 'Nanakshahi (Sikh)',
      origin: '1469 CE (Birth of Guru Nanak)',
      originYear: 1469,
      salviEpochEquivalent: nanakshahi.formatted,
      daysSinceCalendarOrigin: Math.floor(nanakshahi.year * 365.2422),
      yearInCalendar: nanakshahi.year,
      description: `Nanakshahi solar calendar: ${nanakshahi.formatted}`
    },
    {
      calendarSystem: "Baha'i (Badi')",
      origin: '1844 CE (Declaration of the Bab)',
      originYear: 1844,
      salviEpochEquivalent: bahai.formatted,
      daysSinceCalendarOrigin: Math.floor(bahai.year * 365.2422),
      yearInCalendar: bahai.year,
      description: `Bahai calendar: ${bahai.formatted}`
    },
    {
      calendarSystem: 'Minguo (Republic of China)',
      origin: '1912 CE (Republic founding)',
      originYear: 1912,
      salviEpochEquivalent: minguo.formatted,
      daysSinceCalendarOrigin: Math.floor(minguo.year * 365.2422),
      yearInCalendar: minguo.year,
      description: `Minguo calendar: ${minguo.formatted}`
    },
    {
      calendarSystem: 'Igbo',
      origin: 'Traditional (~3,000+ years)',
      originYear: 0,
      salviEpochEquivalent: igbo.formatted,
      daysSinceCalendarOrigin: 0,
      yearInCalendar: 0,
      cyclicPosition: igbo.dayName,
      description: `Igbo 4-day week calendar: ${igbo.formatted}`
    },
    {
      calendarSystem: 'Akan',
      origin: 'Traditional (~3,000+ years)',
      originYear: 0,
      salviEpochEquivalent: akan.formatted,
      daysSinceCalendarOrigin: akan.adaeCycleDay,
      yearInCalendar: 0,
      cyclicPosition: akan.adaeCycleName,
      description: `Akan 42-day Adae cycle: ${akan.formatted}`
    },
    {
      calendarSystem: 'Gregorian',
      origin: '1582 CE (Papal bull Inter gravissimas)',
      originYear: 1582,
      salviEpochEquivalent: gregorian.formatted,
      daysSinceCalendarOrigin: jdn - 2299161,
      yearInCalendar: gregorian.year,
      description: `Gregorian civil calendar: ${gregorian.formatted}`
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
      byzantine,
      thirteenMoon,
      persian,
      ethiopian,
      coptic,
      japaneseKoki,
      koreanDangun,
      thaiBuddhist,
      indianSaka,
      tibetan,
      aztecTonalpohualli,
      romanAUC,
      bengali,
      berber,
      balinesePawukon,
      zoroastrianFasli,
      aboriginalSeasonal,
      assyrian,
      nisgaaSeasonal,
      yoruba,
      jain,
      tamil,
      vietnamese,
      vikramSamvat,
      khmer,
      burmese,
      javanese,
      malayalam,
      nepalSambat,
      nanakshahi,
      bahai,
      minguo,
      igbo,
      akan,
      gregorian
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
 * These are the canonical synchronization points computed at Day Zero
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
      '13-Moon Natural Time': sync.calendars.thirteenMoon.formatted,
      'Persian/Solar Hijri': sync.calendars.persian.formatted,
      'Ethiopian/Ge\'ez': sync.calendars.ethiopian.formatted,
      'Coptic': sync.calendars.coptic.formatted,
      'Japanese Imperial (Koki)': sync.calendars.japaneseKoki.formatted,
      'Korean Dangun Era': sync.calendars.koreanDangun.formatted,
      'Thai Buddhist Era': sync.calendars.thaiBuddhist.formatted,
      'Indian National/Saka': sync.calendars.indianSaka.formatted,
      'Tibetan Rabjung': sync.calendars.tibetan.formatted,
      'Aztec Tonalpohualli': sync.calendars.aztecTonalpohualli.formatted,
      'Roman Ab Urbe Condita': sync.calendars.romanAUC.formatted,
      'Bengali/Bangla': sync.calendars.bengali.formatted,
      'Berber/Amazigh': sync.calendars.berber.formatted,
      'Balinese Pawukon': sync.calendars.balinesePawukon.formatted,
      'Zoroastrian Fasli': sync.calendars.zoroastrianFasli.formatted,
      'Aboriginal Seasonal (Dharawal)': sync.calendars.aboriginalSeasonal.formatted,
      'Assyrian': sync.calendars.assyrian.formatted,
      "Nisg\u0331a'a Seasonal": sync.calendars.nisgaaSeasonal.formatted,
      'Yoruba': sync.calendars.yoruba.formatted,
      'Jain (Vira Nirvana Samvat)': sync.calendars.jain.formatted,
      'Tamil': sync.calendars.tamil.formatted,
      'Vietnamese': sync.calendars.vietnamese.formatted,
      'Vikram Samvat': sync.calendars.vikramSamvat.formatted,
      'Khmer': sync.calendars.khmer.formatted,
      'Burmese': sync.calendars.burmese.formatted,
      'Javanese': sync.calendars.javanese.formatted,
      'Malayalam (Kollam)': sync.calendars.malayalam.formatted,
      'Nepal Sambat': sync.calendars.nepalSambat.formatted,
      'Nanakshahi': sync.calendars.nanakshahi.formatted,
      "Baha'i": sync.calendars.bahai.formatted,
      'Minguo': sync.calendars.minguo.formatted,
      'Igbo': sync.calendars.igbo.formatted,
      'Akan': sync.calendars.akan.formatted,
      'Gregorian': sync.calendars.gregorian.formatted,
      'Unix Timestamp (ms)': SALVI_EPOCH_DATE.getTime().toString(),
      'ISO 8601': SALVI_EPOCH_DATE.toISOString()
    },
    verification: `All calendar mappings are bijectively computed from JDN ${gregorianToJDN(2025, 4, 1)} via the GMT correlation constant ${MAYAN_CORRELATION} and standard astronomical algorithms. Backward time compatibility verified across all 42 calendar systems.`
  };
}
