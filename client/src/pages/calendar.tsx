import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import {
  ArrowRight,
  Clock,
  Globe,
  Shield,
  Zap,
  Check,
  AlertTriangle,
  Calendar,
  RefreshCw,
  Copy,
  Layers,
  Network,
  Building2,
  Database,
  Server,
} from "lucide-react";
import { useState } from "react";
import { motion } from "framer-motion";
import { Link } from "wouter";
import { useQuery } from "@tanstack/react-query";
import { useToast } from "@/hooks/use-toast";

interface CalendarData {
  success: boolean;
  salviEpoch: string;
  calendars: {
    mayanLongCount: { longCount: string; calendarRound: string };
    hebrew: { formatted: string };
    chineseSexagenary: { formatted: string };
    vedic: { formatted: string };
    egyptian: { formatted: string };
    julianDay: { formatted: string };
    islamic: { formatted: string };
    byzantine: { formatted: string };
    thirteenMoon: {
      formatted: string;
      moonName: string;
      day: number;
      galacticSignature: string;
      harmonicTone: number | string;
      arc: string;
      dayOutOfTime: boolean;
    };
  };
}


function HeroSection() {
  return (
    <section className="py-16 md:py-24 bg-gradient-to-b from-primary/5 to-transparent" data-testid="section-calendar-hero">
      <div className="max-w-7xl mx-auto px-5">
        <div className="max-w-4xl mx-auto text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-6">
              12 Calendar Systems / 1 API Call
            </Badge>
          </motion.div>
          <motion.h1
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-4xl md:text-6xl font-bold mb-6 leading-tight"
            data-testid="text-calendar-title"
          >
            One Timestamp.{" "}
            <span className="text-primary">Every Calendar.</span>
          </motion.h1>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-lg md:text-xl text-muted-foreground mb-10 max-w-3xl mx-auto leading-relaxed"
          >
            Convert any date across 12 major calendar systems with femtosecond precision.
            From Mayan Long Count to Islamic Hijri, from Julian Day Numbers to the 13-Moon Harmonic Calendar --
            all routed through Kong Konnect with deterministic caching and a single JDN intermediary for guaranteed bijective accuracy.
          </motion.p>
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="flex flex-col sm:flex-row items-center justify-center gap-4"
          >
            <Button asChild data-testid="button-try-api">
              <Link href="/api-demo">
                <Zap className="w-4 h-4 mr-2" />
                Try the Live API
                <ArrowRight className="w-4 h-4 ml-2" />
              </Link>
            </Button>
            <Button variant="outline" asChild data-testid="button-view-docs">
              <Link href="/docs">View Documentation</Link>
            </Button>
          </motion.div>
        </div>
      </div>
    </section>
  );
}

function ProblemsSection() {
  const problems = [
    {
      icon: AlertTriangle,
      problem: "Calendar Fragmentation",
      severity: "Critical",
      description: "12+ different epoch dates, each with unique rules. Converting between Hebrew lunisolar, Islamic lunar, and Mayan vigesimal systems requires 144 separate conversion functions.",
      solution: "Single JDN intermediary reduces O(n\u00B2) conversions to O(n). Just 24 functions cover all 12 calendars bidirectionally.",
    },
    {
      icon: Clock,
      problem: "Unix Y2038 Overflow",
      severity: "Critical",
      description: "32-bit signed integer timestamps overflow on January 19, 2038 at 03:14:07 UTC. Billions of devices and systems will fail.",
      solution: "128-bit femtosecond timestamps using BigInt arithmetic. No rollover until approximately year 3.9 x 10\u00B2\u2079.",
    },
    {
      icon: AlertTriangle,
      problem: "JavaScript Date Precision",
      severity: "High",
      description: "JavaScript Date objects are limited to millisecond precision with a 2\u2075\u00B3 integer limit. Financial, scientific, and regulatory applications need better.",
      solution: "BigInt femtosecond timestamps (10\u207B\u00B9\u2075s precision) via process.hrtime.bigint() with Salvi Epoch offset.",
    },
    {
      icon: Shield,
      problem: "Leap Second Ambiguity",
      severity: "High",
      description: "Irregular UTC leap second insertions create unpredictable time gaps. POSIX timestamps cannot represent leap seconds at all.",
      solution: "HPTP protocol with leap_second_info field in every timestamp, ensuring continuous and unambiguous time representation.",
    },
    {
      icon: AlertTriangle,
      problem: "Lunisolar Intercalation",
      severity: "Extreme",
      description: "Hebrew and Chinese calendars insert leap months based on complex astronomical rules. Hebrew uses a 19-year Metonic cycle with 4 postponement rules (dehiyot).",
      solution: "Complete algorithmic implementations: Metonic cycle + astronomical calculations, no lookup tables required.",
    },
    {
      icon: Clock,
      problem: "Floating-Point Accumulation",
      severity: "High",
      description: "IEEE 754 floating-point errors accumulate over time. A 1ms error per day becomes 365ms per year -- unacceptable for regulatory compliance.",
      solution: "Integer-only day counts for calendar math. All calculations use integer days or BigInt femtoseconds. Zero accumulation error.",
    },
  ];

  return (
    <section className="py-20 md:py-28" data-testid="section-problems">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Problems We Solve
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-problems-title"
          >
            Temporal Computing Is Broken
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Every system that touches time faces these problems. We solved all of them.
          </motion.p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {problems.map((item, index) => (
            <motion.div
              key={item.problem}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.08 }}
            >
              <Card className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm" data-testid={`card-problem-${index}`}>
                <div className="flex items-start justify-between mb-4 gap-3">
                  <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-destructive/10 text-destructive flex-shrink-0">
                    <item.icon className="w-6 h-6" />
                  </div>
                  <Badge variant="outline" className="border-destructive/30 bg-destructive/5 text-destructive text-xs flex-shrink-0">
                    {item.severity}
                  </Badge>
                </div>
                <h3 className="text-lg font-semibold mb-2">{item.problem}</h3>
                <p className="text-muted-foreground text-sm leading-relaxed mb-4">{item.description}</p>
                <div className="pt-4 border-t border-primary/10">
                  <div className="flex items-start gap-2">
                    <Check className="w-4 h-4 text-primary flex-shrink-0 mt-0.5" />
                    <p className="text-sm text-foreground font-medium">{item.solution}</p>
                  </div>
                </div>
              </Card>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

function CalendarSystemsSection() {
  const calendars = [
    { name: "Gregorian", origin: "1582 CE", depth: "~440 years", type: "Solar", key: "Direct anchor" },
    { name: "Julian Day Number", origin: "4713 BCE", depth: "~6,700 years", type: "Continuous", key: "Universal intermediary" },
    { name: "Mayan Long Count", origin: "3114 BCE", depth: "~5,100 years", type: "Vigesimal", key: "GMT correlation" },
    { name: "Hebrew (Anno Mundi)", origin: "3761 BCE", depth: "~5,800 years", type: "Lunisolar", key: "Metonic cycle" },
    { name: "Chinese Sexagenary", origin: "2637 BCE", depth: "~4,600 years", type: "Lunisolar", key: "60-year cycle" },
    { name: "Vedic Kali Yuga", origin: "3102 BCE", depth: "~5,100 years", type: "Solar", key: "432,000-year age" },
    { name: "Egyptian Civil", origin: "2781 BCE", depth: "~4,800 years", type: "Solar (365d)", key: "Sothic cycle" },
    { name: "Islamic Hijri", origin: "622 CE", depth: "~1,400 years", type: "Lunar", key: "30-year cycle" },
    { name: "Byzantine", origin: "5509 BCE", depth: "~7,500 years", type: "Julian-based", key: "Offset calc" },
    { name: "13-Moon Harmonic", origin: "~28,000 BCE", depth: "~30,000 years", type: "364-day", key: "Golden ratio" },
    { name: "Mayan Tzolkin", origin: "3114 BCE", depth: "~5,100 years", type: "260-day sacred", key: "Modular arithmetic" },
    { name: "Mayan Haab", origin: "3114 BCE", depth: "~5,100 years", type: "365-day civil", key: "Positional" },
  ];

  return (
    <section className="py-20 md:py-28 bg-secondary/30" data-testid="section-calendar-systems">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              12 Calendar Systems
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-calendars-title"
          >
            30,000 Years of Human Timekeeping
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Every calendar system synchronized through the Julian Day Number intermediary.
            One API call returns all 12 simultaneously.
          </motion.p>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.2 }}
        >
          <Card className="max-w-5xl mx-auto border-primary/10 bg-card/80 backdrop-blur-sm overflow-x-auto">
            <div className="p-6 md:p-8">
              <div className="grid grid-cols-5 gap-4 pb-4 border-b border-foreground/10 mb-4 text-sm font-semibold min-w-[600px]">
                <div>Calendar</div>
                <div>Origin</div>
                <div>Time Depth</div>
                <div>Type</div>
                <div>Salvi Method</div>
              </div>
              {calendars.map((cal, index) => (
                <motion.div
                  key={cal.name}
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.3, delay: index * 0.04 }}
                  className="grid grid-cols-5 gap-4 py-3 border-b border-foreground/5 last:border-b-0 text-sm min-w-[600px]"
                  data-testid={`row-calendar-${index}`}
                >
                  <div className="font-medium">{cal.name}</div>
                  <div className="text-muted-foreground">{cal.origin}</div>
                  <div className="text-muted-foreground">{cal.depth}</div>
                  <div className="text-muted-foreground">{cal.type}</div>
                  <div className="text-primary font-medium">{cal.key}</div>
                </motion.div>
              ))}
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function GoldenRatioSection() {
  const moons = [
    { num: 1, name: "Magnetic", sig: "Red Dragon", tone: "Purpose \u2013 Unify", arc: "Pre-\u03C6" },
    { num: 2, name: "Lunar", sig: "White Wind", tone: "Challenge \u2013 Flow", arc: "Pre-\u03C6" },
    { num: 3, name: "Electric", sig: "Blue Night", tone: "Service \u2013 Activate", arc: "Pre-\u03C6" },
    { num: 4, name: "Self-Existing", sig: "Yellow Seed", tone: "Form \u2013 Measure", arc: "Pre-\u03C6" },
    { num: 5, name: "Overtone", sig: "Red Serpent", tone: "Radiance \u2013 Empower", arc: "Pre-\u03C6" },
    { num: 6, name: "Rhythmic", sig: "White World-Bridger", tone: "Equality \u2013 Organize", arc: "Pre-\u03C6" },
    { num: 7, name: "Resonant", sig: "Blue Hand", tone: "Channel \u2013 Inspire", arc: "Pre-\u03C6" },
    { num: 8, name: "Galactic", sig: "Yellow Star", tone: "Integrity \u2013 Harmonize", arc: "Pre-\u03C6" },
    { num: "\u2605", name: "Day Out of Time", sig: "Green Central Sun", tone: "Forgiveness \u2013 Release [\u221E]", arc: "\u03C6-point" },
    { num: 9, name: "Solar", sig: "Red Moon", tone: "Intention \u2013 Pulse", arc: "Post-\u03C6" },
    { num: 10, name: "Planetary", sig: "White Dog", tone: "Manifestation \u2013 Perfect", arc: "Post-\u03C6" },
    { num: 11, name: "Spectral", sig: "Blue Monkey", tone: "Liberation \u2013 Dissolve", arc: "Post-\u03C6" },
    { num: 12, name: "Crystal", sig: "Yellow Human", tone: "Cooperation \u2013 Dedicate", arc: "Post-\u03C6" },
    { num: 13, name: "Cosmic", sig: "Red Skywalker", tone: "Presence \u2013 Endure", arc: "Post-\u03C6" },
  ];

  return (
    <section className="py-20 md:py-28" data-testid="section-golden-ratio">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              13-Moon Harmonic Calendar
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-golden-ratio-title"
          >
            The Golden Ratio Calendar
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-3xl mx-auto"
          >
            13 Moons of 28 days = 364 regular days. The Day Out of Time falls at day 225 (November 11),
            the precise golden ratio point: 364/\u03C6 = 224.96. This creates an 8/5 Fibonacci moon split --
            consecutive Fibonacci numbers whose ratio approximates \u03C6 within 1.1%.
          </motion.p>
        </div>

        <div className="grid lg:grid-cols-2 gap-8 max-w-6xl mx-auto">
          <motion.div
            initial={{ opacity: 0, x: -30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.2 }}
          >
            <Card className="p-6 md:p-8 border-primary/10 bg-card/70 backdrop-blur-sm h-full">
              <h3 className="text-xl font-semibold mb-6">Fibonacci Architecture</h3>
              <div className="space-y-6">
                <div className="p-4 rounded-md bg-primary/5 border border-primary/10">
                  <div className="text-sm font-semibold text-primary mb-2">Pre-\u03C6 Arc: 8 Moons (224 days)</div>
                  <div className="text-sm text-muted-foreground">Moons 1-4: Magnetic to Self-Existing (112 days)</div>
                  <div className="text-sm text-muted-foreground">Moons 5-8: Overtone to Galactic (112 days)</div>
                </div>
                <div className="p-4 rounded-md bg-primary/10 border border-primary/20 text-center">
                  <div className="text-lg font-bold text-primary">\u03C6 Fracture Point: Day 225</div>
                  <div className="text-sm text-muted-foreground">November 11 (11/11) -- Green Central Sun</div>
                  <div className="text-xs text-muted-foreground mt-1">364 / 1.6180339... = 224.96</div>
                </div>
                <div className="p-4 rounded-md bg-primary/5 border border-primary/10">
                  <div className="text-sm font-semibold text-primary mb-2">Post-\u03C6 Arc: 5 Moons (140 days)</div>
                  <div className="text-sm text-muted-foreground">Moons 9-11: Solar to Spectral (84 days)</div>
                  <div className="text-sm text-muted-foreground">Moons 12-13: Crystal to Cosmic (56 days)</div>
                </div>
                <div className="text-center pt-4 border-t border-foreground/10">
                  <div className="text-sm font-mono text-muted-foreground">224 + 1 + 140 = <span className="text-primary font-bold">365 days</span></div>
                  <div className="text-sm font-mono text-muted-foreground">Ratio: 8:5 = 1.600 <span className="text-primary">\u2248 \u03C6 \u2248 1.618</span></div>
                </div>
              </div>
            </Card>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.3 }}
          >
            <Card className="p-6 md:p-8 border-primary/10 bg-card/70 backdrop-blur-sm h-full overflow-x-auto">
              <h3 className="text-xl font-semibold mb-6">Moon Schedule</h3>
              <div className="space-y-0 min-w-[500px]">
                <div className="grid grid-cols-5 gap-2 pb-3 border-b border-foreground/10 text-xs font-semibold text-muted-foreground">
                  <div>#</div>
                  <div>Moon</div>
                  <div>Galactic Signature</div>
                  <div>Harmonic Tone</div>
                  <div>Arc</div>
                </div>
                {moons.map((moon, index) => (
                  <div
                    key={index}
                    className={`grid grid-cols-5 gap-2 py-2 border-b border-foreground/5 last:border-b-0 text-sm ${
                      moon.arc === "\u03C6-point" ? "bg-primary/10 rounded-md px-2 font-semibold text-primary" : ""
                    }`}
                    data-testid={`row-moon-${index}`}
                  >
                    <div className="font-mono">{moon.num}</div>
                    <div className="font-medium">{moon.name}</div>
                    <div className="text-muted-foreground">{moon.sig}</div>
                    <div className="text-muted-foreground text-xs">{moon.tone}</div>
                    <div className={moon.arc === "\u03C6-point" ? "text-primary" : "text-muted-foreground"}>{moon.arc}</div>
                  </div>
                ))}
              </div>
            </Card>
          </motion.div>
        </div>
      </div>
    </section>
  );
}

function LiveDemoSection() {
  const [dateInput, setDateInput] = useState("2025-04-01");
  const [queryDate, setQueryDate] = useState("2025-04-01");
  const { toast } = useToast();

  const { data: calendarData, isLoading, error } = useQuery<CalendarData>({
    queryKey: ["/api/salvi/timing/epoch/calendars", queryDate],
    queryFn: async () => {
      const res = await fetch(`/api/salvi/timing/epoch/calendars?date=${queryDate}T00:00:00Z`);
      if (!res.ok) throw new Error("Failed to fetch");
      return res.json();
    },
  });

  const handleLookup = () => {
    if (dateInput) {
      setQueryDate(dateInput);
    }
  };

  const copyEndpoint = () => {
    navigator.clipboard.writeText(`/api/salvi/timing/epoch/calendars?date=${queryDate}T00:00:00Z`);
    toast({ title: "Copied", description: "API endpoint copied to clipboard." });
  };

  return (
    <section className="py-20 md:py-28 bg-secondary/30" data-testid="section-live-demo">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-12">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Live API
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-demo-title"
          >
            Try It Right Now
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Enter any date and see it converted across all 12 calendar systems instantly.
          </motion.p>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.2 }}
        >
          <Card className="max-w-4xl mx-auto p-6 md:p-8 border-primary/10 bg-card/80 backdrop-blur-sm">
            <div className="flex flex-col sm:flex-row items-stretch gap-3 mb-8">
              <input
                type="date"
                value={dateInput}
                onChange={(e) => setDateInput(e.target.value)}
                className="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm"
                data-testid="input-date"
              />
              <Button onClick={handleLookup} disabled={isLoading} data-testid="button-lookup">
                {isLoading ? <RefreshCw className="w-4 h-4 mr-2 animate-spin" /> : <Calendar className="w-4 h-4 mr-2" />}
                Convert Date
              </Button>
              <Button variant="outline" onClick={copyEndpoint} data-testid="button-copy-endpoint">
                <Copy className="w-4 h-4 mr-2" />
                Copy Endpoint
              </Button>
            </div>

            {error && (
              <div className="text-destructive text-sm mb-4" data-testid="text-error">Failed to load calendar data.</div>
            )}

            {calendarData && calendarData.calendars && (
              <div className="space-y-3">
                {[
                  { label: "Julian Day", value: calendarData.calendars.julianDay?.formatted },
                  { label: "Mayan Long Count", value: `${calendarData.calendars.mayanLongCount?.longCount} | ${calendarData.calendars.mayanLongCount?.calendarRound}` },
                  { label: "Hebrew", value: calendarData.calendars.hebrew?.formatted },
                  { label: "Chinese", value: calendarData.calendars.chineseSexagenary?.formatted },
                  { label: "Vedic (Kali Yuga)", value: calendarData.calendars.vedic?.formatted },
                  { label: "Egyptian Civil", value: calendarData.calendars.egyptian?.formatted },
                  { label: "Islamic Hijri", value: calendarData.calendars.islamic?.formatted },
                  { label: "Byzantine", value: calendarData.calendars.byzantine?.formatted },
                  { label: "13-Moon", value: calendarData.calendars.thirteenMoon?.formatted },
                ].filter(item => item.value).map((item, index) => (
                  <div
                    key={item.label}
                    className="flex flex-col sm:flex-row sm:items-center justify-between py-3 border-b border-foreground/5 last:border-b-0 gap-1"
                    data-testid={`result-calendar-${index}`}
                  >
                    <span className="text-sm font-medium">{item.label}</span>
                    <span className="text-sm text-primary font-mono">{item.value}</span>
                  </div>
                ))}

                {calendarData.calendars.thirteenMoon && !calendarData.calendars.thirteenMoon.dayOutOfTime && (
                  <div className="mt-4 pt-4 border-t border-primary/10">
                    <div className="flex flex-wrap items-center gap-3">
                      <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">
                        {calendarData.calendars.thirteenMoon.galacticSignature}
                      </Badge>
                      <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">
                        Tone {calendarData.calendars.thirteenMoon.harmonicTone}
                      </Badge>
                      <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary">
                        {calendarData.calendars.thirteenMoon.arc}
                      </Badge>
                    </div>
                  </div>
                )}
              </div>
            )}
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function KongGatewaySection() {
  const routes = [
    { path: "/api/salvi/timing/epoch/calendars", service: "salvi-calendar", plugins: "rate-limit, jwt-auth, cors", cache: "\u221E (historical) / 24h (future)" },
    { path: "/api/salvi/timing/epoch/calendars/:system", service: "salvi-calendar", plugins: "rate-limit, jwt-auth, cors", cache: "\u221E (historical) / 24h (future)" },
    { path: "/api/salvi/timing/dot/:year", service: "salvi-calendar", plugins: "rate-limit, cors", cache: "\u221E (deterministic)" },
    { path: "/api/salvi/timing/timestamp", service: "salvi-timing", plugins: "rate-limit, jwt-auth, cors", cache: "none (live)" },
    { path: "/api/salvi/timing/sync", service: "salvi-timing", plugins: "jwt-auth, ip-restrict", cache: "none (stateful)" },
    { path: "/api/salvi/timing/batch/:count", service: "salvi-timing", plugins: "rate-limit(strict), jwt-auth", cache: "none (live)" },
    { path: "/api/salvi/timing/metrics", service: "salvi-timing", plugins: "rate-limit, jwt-auth", cache: "5s TTL" },
  ];

  const cacheStrategies = [
    { icon: Database, label: "Historical Dates", ttl: "\u221E TTL", detail: "Calendar conversions for past dates are pure functions -- deterministic, immutable, infinitely cacheable." },
    { icon: Clock, label: "Today's Date", ttl: "24h TTL", detail: "Sunset-based calendars (Hebrew, Islamic) may shift at day boundary. Refreshed daily." },
    { icon: RefreshCw, label: "Future Dates", ttl: "24h TTL", detail: "Pending leap second announcements or Islamic observational determinations may alter results." },
    { icon: Zap, label: "Live Timing", ttl: "No Cache", detail: "Femtosecond timestamps require real-time freshness. Every request generates a new timestamp." },
  ];

  return (
    <section className="py-20 md:py-28 bg-secondary/30" data-testid="section-kong-gateway">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Kong Konnect Gateway
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-kong-title"
          >
            API Gateway Architecture
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-3xl mx-auto"
          >
            Every calendar endpoint routes through Kong Konnect with deterministic caching,
            rate limiting, JWT authentication, and health-check degradation.
          </motion.p>
        </div>

        <div className="grid lg:grid-cols-2 gap-8 max-w-6xl mx-auto mb-12">
          <motion.div
            initial={{ opacity: 0, x: -30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.2 }}
          >
            <Card className="p-6 md:p-8 border-primary/10 bg-card/70 backdrop-blur-sm h-full">
              <div className="flex items-center gap-3 mb-6">
                <div className="inline-flex items-center justify-center w-10 h-10 rounded-full bg-primary/10 text-primary flex-shrink-0">
                  <Server className="w-5 h-5" />
                </div>
                <h3 className="text-xl font-semibold">Route Configuration</h3>
              </div>
              <div className="space-y-0 overflow-x-auto">
                <div className="grid grid-cols-3 gap-2 pb-3 border-b border-foreground/10 text-xs font-semibold text-muted-foreground min-w-[450px]">
                  <div>Route</div>
                  <div>Plugins</div>
                  <div>Cache</div>
                </div>
                {routes.map((route, index) => (
                  <div
                    key={index}
                    className="grid grid-cols-3 gap-2 py-2.5 border-b border-foreground/5 last:border-b-0 text-xs min-w-[450px]"
                    data-testid={`row-kong-route-${index}`}
                  >
                    <div className="font-mono text-primary truncate">{route.path}</div>
                    <div className="text-muted-foreground">{route.plugins}</div>
                    <div className="text-muted-foreground">{route.cache}</div>
                  </div>
                ))}
              </div>
            </Card>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.3 }}
          >
            <Card className="p-6 md:p-8 border-primary/10 bg-card/70 backdrop-blur-sm h-full">
              <div className="flex items-center gap-3 mb-6">
                <div className="inline-flex items-center justify-center w-10 h-10 rounded-full bg-primary/10 text-primary flex-shrink-0">
                  <Database className="w-5 h-5" />
                </div>
                <h3 className="text-xl font-semibold">Deterministic Cache Strategy</h3>
              </div>
              <div className="space-y-4">
                {cacheStrategies.map((strategy, index) => (
                  <div key={index} className="flex items-start gap-3" data-testid={`cache-strategy-${index}`}>
                    <div className="inline-flex items-center justify-center w-8 h-8 rounded-full bg-primary/5 text-primary flex-shrink-0 mt-0.5">
                      <strategy.icon className="w-4 h-4" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex flex-wrap items-center gap-2 mb-1">
                        <span className="text-sm font-medium">{strategy.label}</span>
                        <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary text-xs">
                          {strategy.ttl}
                        </Badge>
                      </div>
                      <p className="text-xs text-muted-foreground leading-relaxed">{strategy.detail}</p>
                    </div>
                  </div>
                ))}
              </div>
            </Card>
          </motion.div>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.35 }}
          className="max-w-6xl mx-auto"
        >
          <Card className="p-6 md:p-8 border-primary/10 bg-card/70 backdrop-blur-sm">
            <div className="flex items-center gap-3 mb-6">
              <div className="inline-flex items-center justify-center w-10 h-10 rounded-full bg-primary/10 text-primary flex-shrink-0">
                <Shield className="w-5 h-5" />
              </div>
              <h3 className="text-xl font-semibold">Gateway Features</h3>
            </div>
            <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6">
              <div data-testid="kong-feature-ratelimit">
                <div className="text-sm font-semibold mb-1">Rate Limiting</div>
                <p className="text-xs text-muted-foreground">100 req/min standard, 30 req/min for bulk conversions. Redis-backed with burst allowance.</p>
              </div>
              <div data-testid="kong-feature-auth">
                <div className="text-sm font-semibold mb-1">JWT Authentication</div>
                <p className="text-xs text-muted-foreground">Bearer tokens via Salvi auth service. Public endpoints support anonymous access with reduced limits.</p>
              </div>
              <div data-testid="kong-feature-health">
                <div className="text-sm font-semibold mb-1">Health Check Degradation</div>
                <p className="text-xs text-muted-foreground">If calendar service goes down, Kong serves cached historical conversions while returning 503 for live timing.</p>
              </div>
              <div data-testid="kong-feature-transform">
                <div className="text-sm font-semibold mb-1">Response Transform</div>
                <p className="text-xs text-muted-foreground">Automatic Cache-Control headers: immutable for historical dates, max-age=86400 for current/future.</p>
              </div>
            </div>
          </Card>
        </motion.div>
      </div>
    </section>
  );
}

function UseCasesSection() {
  const useCases = [
    {
      icon: Building2,
      title: "Financial Services & Compliance",
      description: "FINRA 613 and MiFID II require sub-millisecond timestamp accuracy. Our femtosecond timing with calendar synchronization provides regulatory-grade timestamps across any calendar system used in global markets.",
      benefit: "Built-in compliance",
    },
    {
      icon: Globe,
      title: "International Business",
      description: "Global operations span Islamic, Hebrew, Chinese, and Gregorian calendars. Convert scheduling, contracts, and reporting dates across all systems with a single API call -- no custom conversion logic needed.",
      benefit: "12 calendars, 1 API",
    },
    {
      icon: Layers,
      title: "Historical & Academic Research",
      description: "Cross-reference events across Mayan Long Count, Egyptian Civil, Byzantine, and modern calendars. Archaeologists, historians, and anthropologists get bijective precision back to 28,000 BCE.",
      benefit: "30,000+ years",
    },
    {
      icon: Network,
      title: "Blockchain & Smart Contracts",
      description: "Immutable temporal anchoring for smart contracts, payment witnessing, and audit trails. Every transaction timestamped with femtosecond precision and convertible across any calendar system.",
      benefit: "Verifiable time",
    },
    {
      icon: Shield,
      title: "Defense & Intelligence",
      description: "Quantum-resistant timing with phase encryption and timing-window enforcement. Multi-calendar coordination for international operations and intelligence fusion across allied forces.",
      benefit: "Post-quantum secure",
    },
    {
      icon: Clock,
      title: "Scientific Computing",
      description: "128-bit femtosecond timestamps eliminate floating-point accumulation errors. Perfect for particle physics, astronomical observations, and any domain where nanosecond precision matters.",
      benefit: "Zero drift",
    },
  ];

  return (
    <section className="py-20 md:py-28" data-testid="section-use-cases">
      <div className="max-w-7xl mx-auto px-5">
        <div className="text-center mb-16">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
          >
            <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary px-4 py-1.5 mb-4">
              Real-World Applications
            </Badge>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="text-3xl md:text-4xl font-bold mb-4"
            data-testid="text-use-cases-title"
          >
            Who Needs This
          </motion.h2>
          <motion.p
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="text-muted-foreground text-lg max-w-2xl mx-auto"
          >
            Any system that touches time across cultures, regulations, or precision requirements.
          </motion.p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {useCases.map((item, index) => (
            <motion.div
              key={item.title}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: index * 0.08 }}
            >
              <Card className="p-6 md:p-8 h-full border-primary/10 bg-card/70 backdrop-blur-sm" data-testid={`card-usecase-${index}`}>
                <div className="flex items-start justify-between mb-4 gap-3">
                  <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-primary/10 text-primary flex-shrink-0">
                    <item.icon className="w-6 h-6" />
                  </div>
                  <Badge variant="outline" className="border-primary/30 bg-primary/5 text-primary text-xs flex-shrink-0">
                    {item.benefit}
                  </Badge>
                </div>
                <h3 className="text-lg font-semibold mb-2">{item.title}</h3>
                <p className="text-muted-foreground text-sm leading-relaxed">{item.description}</p>
              </Card>
            </motion.div>
          ))}
        </div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.5 }}
          className="text-center mt-12"
        >
          <Button asChild data-testid="button-get-started">
            <Link href="/api-demo">
              <Zap className="w-4 h-4 mr-2" />
              Get Started with the API
              <ArrowRight className="w-4 h-4 ml-2" />
            </Link>
          </Button>
        </motion.div>
      </div>
    </section>
  );
}

export default function CalendarPage() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <main>
        <HeroSection />
        <ProblemsSection />
        <CalendarSystemsSection />
        <GoldenRatioSection />
        <KongGatewaySection />
        <LiveDemoSection />
        <UseCasesSection />
      </main>
      <footer className="border-t border-primary/10 py-8">
        <div className="max-w-7xl mx-auto px-5 text-center">
          <p className="text-sm text-muted-foreground">
            Capomastro Holdings Ltd. -- Temporal Unification Architecture
          </p>
        </div>
      </footer>
    </div>
  );
}
