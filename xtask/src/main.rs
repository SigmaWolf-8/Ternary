// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
//
// xtask — workspace task runner.
//
// Final form orchestrates the 14 G-gates of task-133 via
// `cargo xtask repx-zero-error`. This stub records the wall-clock
// budget and lists the gates in execution order; full
// implementation lands in task-133 G11 follow-up.

use std::env;
use std::process::ExitCode;

/// Wall-clock budget for the full G1–G14 sweep, per task-133 §G11.
pub const REPX_MAX_WALL_SECONDS: u64 = 1800;

const GATES: &[&str] = &[
    "G1: build & lint",
    "G2: rename-safety sweep",
    "G3: 1521-pair interchange matrix",
    "G4: symbol-map AST lint",
    "G5: backend bit-identity",
    "G6: worked-example acceptance",
    "G7: doc-test & cookbook",
    "G8: FFI / JSON surface",
    "G9: determinism & reproducibility",
    "G10: downstream consumer smoke",
    "G11: CI wiring (this runner)",
    "G12: pre-merge checklist",
    "G13: crypto byte-identity",
    "G14: workspace wiring",
];

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    match cmd {
        "repx-zero-error" => {
            println!("xtask repx-zero-error (stub) — gates planned:");
            for g in GATES {
                println!("  {g}");
            }
            println!("(implementation lands in task-133 G11 follow-up)");
            ExitCode::SUCCESS
        }
        "" | "-h" | "--help" => {
            println!("xtask — workspace task runner");
            println!("Usage: cargo xtask <repx-zero-error>");
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("xtask: unknown subcommand `{}`", other);
            ExitCode::from(2)
        }
    }
}
