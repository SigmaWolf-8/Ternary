// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
//
// repx-cli — stub binary. Wires up the RepX engine façade for
// `repx describe / read / find / convert` once the engine lands.
// Tracked by task-133 G7. Current build target only.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    match cmd {
        "describe" | "read" | "find" | "convert" => {
            eprintln!(
                "repx-cli: subcommand `{}` is not yet wired (task-133 G7 follow-up).",
                cmd
            );
            ExitCode::from(2)
        }
        "" | "-h" | "--help" => {
            println!("repx — Salvi Framework physics-engine CLI (stub)");
            println!("Usage: repx <describe|read|find|convert> [args...]");
            println!("Subcommand implementations land in task-133 G7 follow-up.");
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("repx-cli: unknown subcommand `{}`", other);
            ExitCode::from(2)
        }
    }
}
