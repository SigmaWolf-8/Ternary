// TDNS v2.3 — CLI Scanner
// Capomastro Holdings Ltd. — Applied Physics Division
//
// Usage:
//   tdns-scan <url> [url2] [url3] ...
//   tdns-scan --compare <url1> <url2>
//   tdns-scan --describe WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313
//
// Scans live targets and derives their 27-trit ontological addresses.
// No simulations. Real network inspection.

use std::env;
use std::process;

use tdns_v2::addr::CubeAddr;
use tdns_v2::scanner::{format_scan_report, scan};
use tdns_v2::schema::SCHEMA;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            print_usage();
        }
        "--compare" => {
            if args.len() < 4 {
                eprintln!("ERROR: --compare requires two URLs");
                process::exit(1);
            }
            cmd_compare(&args[2], &args[3]);
        }
        "--describe" => {
            if args.len() < 3 {
                eprintln!("ERROR: --describe requires an address");
                process::exit(1);
            }
            let addr_str = args[2..].join(" ");
            cmd_describe(&addr_str);
        }
        _ => {
            let urls: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
            cmd_scan(&urls);
        }
    }
}

// ─── Commands ────────────────────────────────────────────────────────────────

fn cmd_scan(urls: &[&str]) {
    let mut results = Vec::new();

    for url in urls {
        eprintln!("Scanning {}...", url);
        match scan(url) {
            Ok(result) => {
                println!("{}", format_scan_report(&result));
                results.push(result);
            }
            Err(e) => {
                eprintln!("FAILED {}: {}", url, e);
            }
        }
    }

    if results.len() > 1 {
        println!("═══ DISTANCE MATRIX ═══");
        for i in 0..results.len() {
            for j in (i + 1)..results.len() {
                let dist = results[i].address.distance(&results[j].address);
                println!(
                    "  {} ↔ {} = {} hops",
                    results[i].target.domain, results[j].target.domain, dist
                );
            }
        }
        println!("═══════════════════════");
    }
}

fn cmd_compare(url1: &str, url2: &str) {
    eprintln!("Scanning {}...", url1);
    let r1 = match scan(url1) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("FAILED {}: {}", url1, e);
            process::exit(1);
        }
    };

    eprintln!("Scanning {}...", url2);
    let r2 = match scan(url2) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("FAILED {}: {}", url2, e);
            process::exit(1);
        }
    };

    println!("═══ CRS COMPARISON REPORT ═══");
    println!("{}: {}", r1.target.domain, r1.address);
    println!("{}: {}", r2.target.domain, r2.address);
    println!();

    let distance = r1.address.distance(&r2.address);
    let diffs = r1.address.differing_dims(&r2.address);

    println!("Ontological distance: {} hops ({}/27 dimensions differ)", distance, diffs.len());
    println!();

    if diffs.is_empty() {
        println!("  These entities are ontologically identical.");
    } else {
        println!("  Differing dimensions:");
        for &dim_idx in &diffs {
            let dim = &SCHEMA[dim_idx];
            let t1 = r1.address.trit(dim_idx);
            let t2 = r2.address.trit(dim_idx);
            println!(
                "  {:2}. {} : {} ({}) vs {} ({})",
                dim.number,
                dim.question,
                dim.label(t1),
                t1,
                dim.label(t2),
                t2,
            );
        }
        println!();
        println!("  Matching dimensions:");
        for dim_idx in 0..27 {
            if !diffs.contains(&dim_idx) {
                let dim = &SCHEMA[dim_idx];
                let t = r1.address.trit(dim_idx);
                println!(
                    "  {:2}. {} : {} ({})",
                    dim.number,
                    dim.question,
                    dim.label(t),
                    t,
                );
            }
        }
    }

    println!();
    println!("HPTP-mandatory: {} = {}, {} = {}",
        r1.target.domain, r1.address.is_hptp_mandatory(),
        r2.target.domain, r2.address.is_hptp_mandatory(),
    );
    println!("═════════════════════════════");
}

fn cmd_describe(addr_str: &str) {
    let addr = match addr_str.parse::<CubeAddr>() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("ERROR: invalid address: {}", e);
            process::exit(1);
        }
    };

    println!("═══ ADDRESS DESCRIPTION ═══");
    println!("Address: {}", addr);
    println!("HPTP-mandatory: {}", addr.is_hptp_mandatory());
    println!();

    let mut current_category = "";
    for (i, dim) in SCHEMA.iter().enumerate() {
        let cat_prefix = dim.category.prefix();
        if cat_prefix != current_category {
            println!("\n  {} — {}", cat_prefix, dim.category.root_question());
            current_category = cat_prefix;
        }
        let trit = addr.trit(i);
        println!(
            "  {:2}. {} → {} ({})",
            dim.number,
            dim.question,
            dim.label(trit),
            trit,
        );
    }
    println!();
    println!("═══════════════════════════");
}

fn print_usage() {
    println!("TDNS v2.3 — CRS Scanner");
    println!("Capomastro Holdings Ltd. — Applied Physics Division");
    println!();
    println!("USAGE:");
    println!("  tdns-scan <url>                           Scan a live target");
    println!("  tdns-scan <url1> <url2> [url3] ...        Scan multiple targets + distance matrix");
    println!("  tdns-scan --compare <url1> <url2>         Detailed side-by-side comparison");
    println!("  tdns-scan --describe <address>            Describe a known address");
    println!();
    println!("EXAMPLES:");
    println!("  tdns-scan https://github.com");
    println!("  tdns-scan github.com google.com wikipedia.org");
    println!("  tdns-scan --compare github.com google.com");
    println!("  tdns-scan --describe \"WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313\"");
}
