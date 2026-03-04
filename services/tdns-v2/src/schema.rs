// TDNS v2.3 — Schema
// Capomastro Holdings Ltd. — Applied Physics Division
//
// The 27-dimensional ontological schema. Every dimension answers a
// plain-language question. Every trit value has a human label and a
// machine-deterministic scan method.
//
// WHO · WHAT · WHERE · WHEN · WHY · HOW · PEACE

use serde::{Deserialize, Serialize};

use crate::trit::Trit;

// ─── Category ────────────────────────────────────────────────────────────────

/// The seven root categories of the ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Who,
    What,
    Where,
    When,
    Why,
    How,
    Peace,
}

impl Category {
    pub const ALL: [Category; 7] = [
        Category::Who,
        Category::What,
        Category::Where,
        Category::When,
        Category::Why,
        Category::How,
        Category::Peace,
    ];

    /// Two-letter prefix for display format.
    pub const fn prefix(&self) -> &'static str {
        match self {
            Category::Who => "WO",
            Category::What => "WA",
            Category::Where => "WR",
            Category::When => "WN",
            Category::Why => "WY",
            Category::How => "HO",
            Category::Peace => "PE",
        }
    }

    /// The root question this category answers.
    pub const fn root_question(&self) -> &'static str {
        match self {
            Category::Who => "Who is behind it?",
            Category::What => "What is it?",
            Category::Where => "Where can I find it?",
            Category::When => "When does it operate?",
            Category::Why => "Why does it exist?",
            Category::How => "How does it work?",
            Category::Peace => "Can I sleep at night?",
        }
    }

    /// Dimension index range (0-based, inclusive).
    pub const fn dim_range(&self) -> (usize, usize) {
        match self {
            Category::Who => (0, 3),
            Category::What => (4, 7),
            Category::Where => (8, 11),
            Category::When => (12, 15),
            Category::Why => (16, 19),
            Category::How => (20, 23),
            Category::Peace => (24, 26),
        }
    }
}

// ─── Dimension ───────────────────────────────────────────────────────────────

/// Metadata for a single dimension of the ontology.
#[derive(Debug, Clone)]
pub struct Dimension {
    /// 1-based dimension number (matches spec).
    pub number: usize,
    /// Which category this dimension belongs to.
    pub category: Category,
    /// The question this dimension answers.
    pub question: &'static str,
    /// Human labels for each trit value [V1, V2, V3].
    pub labels: [&'static str; 3],
    /// What CRS scans to determine this value.
    pub scan_method: &'static str,
}

impl Dimension {
    /// Get the label for a given trit value.
    pub fn label(&self, trit: Trit) -> &'static str {
        self.labels[trit.index()]
    }
}

// ─── The Schema ──────────────────────────────────────────────────────────────

/// The complete 27-dimensional schema, indexed 0–26.
pub static SCHEMA: [Dimension; 27] = [
    // ── WHO (trits 1–4) ──────────────────────────────────────────────
    Dimension {
        number: 1,
        category: Category::Who,
        question: "What kind?",
        labels: ["Personal", "Corporate", "Governance"],
        scan_method: "WHOIS + legal entity DB",
    },
    Dimension {
        number: 2,
        category: Category::Who,
        question: "Who's it for?",
        labels: ["Just me", "My group", "Everyone"],
        scan_method: "Access patterns, robots.txt",
    },
    Dimension {
        number: 3,
        category: Category::Who,
        question: "Who runs it?",
        labels: ["Anonymous", "Known", "Transparent"],
        scan_method: "About page, WHOIS privacy, business registry",
    },
    Dimension {
        number: 4,
        category: Category::Who,
        question: "Who hosts it?",
        labels: ["Me", "A provider", "The cloud"],
        scan_method: "ASN lookup, IP range, cloud provider fingerprint",
    },
    // ── WHAT (trits 5–8) ─────────────────────────────────────────────
    Dimension {
        number: 5,
        category: Category::What,
        question: "What is it?",
        labels: ["Website", "App", "Device"],
        scan_method: "HTTP headers, content-type, TCP fingerprint",
    },
    Dimension {
        number: 6,
        category: Category::What,
        question: "What's on it?",
        labels: ["Text", "Media", "Live"],
        scan_method: "MIME types served",
    },
    Dimension {
        number: 7,
        category: Category::What,
        question: "Who uses it?",
        labels: ["People", "Software", "Both"],
        scan_method: "UI presence vs API-only patterns",
    },
    Dimension {
        number: 8,
        category: Category::What,
        question: "Does it think?",
        labels: ["No", "Partly", "Yes"],
        scan_method: "ML endpoint detection, inference headers",
    },
    // ── WHERE (trits 9–12) ───────────────────────────────────────────
    Dimension {
        number: 9,
        category: Category::Where,
        question: "Who can see it?",
        labels: ["Just me", "My group", "Everyone"],
        scan_method: "Unauthenticated GET: 200 / 401 / timeout",
    },
    Dimension {
        number: 10,
        category: Category::Where,
        question: "Do I need to log in?",
        labels: ["No", "Password", "ID Check"],
        scan_method: "Challenge detection: none / form / MFA+cert",
    },
    Dimension {
        number: 11,
        category: Category::Where,
        question: "How many servers?",
        labels: ["One", "Several", "Many"],
        scan_method: "DNS A/AAAA record count, CDN detection",
    },
    Dimension {
        number: 12,
        category: Category::Where,
        question: "What connection?",
        labels: ["HTTP", "WebSocket", "Raw TCP"],
        scan_method: "Port scan, protocol handshake",
    },
    // ── WHEN (trits 13–16) ───────────────────────────────────────────
    Dimension {
        number: 13,
        category: Category::When,
        question: "What era?",
        labels: ["Pre-2010", "2010s", "2020s+"],
        scan_method: "Domain registration date, first cert issuance, protocol fingerprint",
    },
    Dimension {
        number: 14,
        category: Category::When,
        question: "When is it available?",
        labels: ["Business hours", "Extended", "24/7"],
        scan_method: "Uptime monitoring over sample window",
    },
    Dimension {
        number: 15,
        category: Category::When,
        question: "What kind of data?",
        labels: ["Historical", "Current", "Live"],
        scan_method: "Content timestamps, streaming protocol detection",
    },
    Dimension {
        number: 16,
        category: Category::When,
        question: "Is it real-time?",
        labels: ["Batch", "Near-time", "Real-time"],
        scan_method: "Latency measurement, WebSocket/gRPC/SSE",
    },
    // ── WHY (trits 17–20) ────────────────────────────────────────────
    Dimension {
        number: 17,
        category: Category::Why,
        question: "Does it handle money?",
        labels: ["No", "Accepts", "Processes"],
        scan_method: "Payment endpoint detection, merchant headers",
    },
    Dimension {
        number: 18,
        category: Category::Why,
        question: "Does it want my data?",
        labels: ["No", "Some", "Lots"],
        scan_method: "Input field count, registration forms, data-sharing scripts",
    },
    Dimension {
        number: 19,
        category: Category::Why,
        question: "Does it have policies?",
        labels: ["No", "Basic", "Detailed"],
        scan_method: "Scan for /privacy, /terms, cookie consent",
    },
    Dimension {
        number: 20,
        category: Category::Why,
        question: "Does it cost money?",
        labels: ["Free", "Pay-per-use", "Subscription"],
        scan_method: "Paywall detection, pricing page",
    },
    // ── HOW (trits 21–24) ────────────────────────────────────────────
    Dimension {
        number: 21,
        category: Category::How,
        question: "Who gets it?",
        labels: ["One person", "A group", "Whoever's closest"],
        scan_method: "Multicast headers, anycast DNS, CDN fanout",
    },
    Dimension {
        number: 22,
        category: Category::How,
        question: "Which way does data go?",
        labels: ["Out", "Through", "In"],
        scan_method: "GET vs POST ratio, data flow analysis",
    },
    Dimension {
        number: 23,
        category: Category::How,
        question: "How do I get updates?",
        labels: ["I ask", "I subscribe", "It tells me"],
        scan_method: "RSS/Atom, WebSocket/SSE, polling detection",
    },
    Dimension {
        number: 24,
        category: Category::How,
        question: "Does it remember me?",
        labels: ["No", "For a bit", "Always"],
        scan_method: "Cookie/session/localStorage analysis",
    },
    // ── PEACE (trits 25–27) ──────────────────────────────────────────
    Dimension {
        number: 25,
        category: Category::Peace,
        question: "Is it encrypted?",
        labels: ["No", "Basic TLS", "Full TLS"],
        scan_method: "TLS version, HSTS, CSP, security.txt",
    },
    Dimension {
        number: 26,
        category: Category::Peace,
        question: "How many trackers?",
        labels: ["Many", "Few", "None"],
        scan_method: "Third-party request count on page load",
    },
    Dimension {
        number: 27,
        category: Category::Peace,
        question: "Has it been audited?",
        labels: ["No", "Self-certified", "Audited"],
        scan_method: "SOC2/ISO badge scan, audit certificates",
    },
];

/// Look up a dimension by 1-based number.
pub fn dimension(number: usize) -> &'static Dimension {
    &SCHEMA[number - 1]
}

/// Describe an address in plain English, dimension by dimension.
pub fn describe(addr: &crate::addr::CubeAddr) -> Vec<(usize, &'static str, &'static str, &'static str)> {
    SCHEMA
        .iter()
        .enumerate()
        .map(|(i, dim)| {
            let trit = addr.trit(i);
            (dim.number, dim.question, dim.label(trit), dim.category.prefix())
        })
        .collect()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::CubeAddr;

    #[test]
    fn schema_has_27_dimensions() {
        assert_eq!(SCHEMA.len(), 27);
    }

    #[test]
    fn dimension_numbers_sequential() {
        for (i, dim) in SCHEMA.iter().enumerate() {
            assert_eq!(dim.number, i + 1);
        }
    }

    #[test]
    fn categories_cover_all_dims() {
        let mut covered = vec![false; 27];
        for cat in Category::ALL {
            let (start, end) = cat.dim_range();
            for i in start..=end {
                covered[i] = true;
            }
        }
        assert!(covered.iter().all(|&b| b));
    }

    #[test]
    fn describe_google() {
        let g = CubeAddr::from_category_string(
            "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313",
        )
        .unwrap();

        let desc = describe(&g);
        assert_eq!(desc.len(), 27);

        // Trit 1: Corporate
        assert_eq!(desc[0].2, "Corporate");
        // Trit 8: Yes (thinks)
        assert_eq!(desc[7].2, "Yes");
        // Trit 25: Full TLS
        assert_eq!(desc[24].2, "Full TLS");
        // Trit 26: Many trackers
        assert_eq!(desc[25].2, "Many");
    }

    #[test]
    fn each_label_array_has_three_entries() {
        for dim in &SCHEMA {
            assert_eq!(dim.labels.len(), 3);
            for label in dim.labels {
                assert!(!label.is_empty());
            }
        }
    }

    #[test]
    fn every_dimension_has_scan_method() {
        for dim in &SCHEMA {
            assert!(!dim.scan_method.is_empty());
        }
    }
}
