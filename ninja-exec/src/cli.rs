// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Command {
    Run {
        port: Option<u16>,
        headless: bool,
        data_dir: Option<PathBuf>,
    },
    Init {
        data_dir: Option<PathBuf>,
    },
    Pubkey {
        data_dir: Option<PathBuf>,
    },
    Fingerprint {
        data_dir: Option<PathBuf>,
    },
    ExportOperator {
        data_dir: Option<PathBuf>,
        clipboard: bool,
    },
    Status {
        port: Option<u16>,
    },
    Lock {
        port: Option<u16>,
    },
    Unlock {
        port: Option<u16>,
    },
    SignFile {
        file: String,
        data_dir: Option<PathBuf>,
    },
    VerifyFile {
        file: String,
        signature: String,
        data_dir: Option<PathBuf>,
    },
    Version,
}

pub fn parse_args() -> Command {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Command::Run {
            port: env_port(),
            headless: false,
            data_dir: None,
        };
    }

    let mut port: Option<u16> = env_port();
    let mut headless = false;
    let mut data_dir: Option<PathBuf> = None;
    let mut clipboard = false;

    let subcommand = args[1].as_str();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            s if s.starts_with("--port=") => {
                port = s[7..].parse().ok();
                i += 1;
            }
            "--headless" => {
                headless = true;
                i += 1;
            }
            "--data-dir" => {
                if i + 1 < args.len() {
                    data_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    i += 1;
                }
            }
            s if s.starts_with("--data-dir=") => {
                data_dir = Some(PathBuf::from(&s[11..]));
                i += 1;
            }
            "--clipboard" => {
                clipboard = true;
                i += 1;
            }
            "--version" | "-V" => {
                return Command::Version;
            }
            _ => {
                i += 1;
            }
        }
    }

    match subcommand {
        "init" => Command::Init { data_dir },
        "pubkey" => Command::Pubkey { data_dir },
        "fingerprint" => Command::Fingerprint { data_dir },
        "export-operator" => Command::ExportOperator { data_dir, clipboard },
        "status" => Command::Status { port },
        "lock" => Command::Lock { port },
        "unlock" => Command::Unlock { port },
        "sign" => {
            let file = args.get(2).cloned().unwrap_or_default();
            Command::SignFile { file, data_dir }
        }
        "verify" => {
            let file = args.get(2).cloned().unwrap_or_default();
            let signature = args.get(3).cloned().unwrap_or_default();
            Command::VerifyFile { file, signature, data_dir }
        }
        "--version" | "-V" => Command::Version,
        "--headless" => Command::Run {
            port,
            headless: true,
            data_dir,
        },
        _ => Command::Run {
            port,
            headless,
            data_dir,
        },
    }
}

fn env_port() -> Option<u16> {
    std::env::var("NINJA_EXEC_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
}
