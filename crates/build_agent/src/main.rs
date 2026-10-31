//! noro-build-agent: launcher releases built on our own machines.
//!
//! `run` sits on a Mac or a Windows box and asks the master for work: it gets
//! a source archive and a target, builds it, packages it the way the release
//! workflow does and uploads the result. `submit` packs the working copy of
//! this repository and queues a build of it. No Git hosting and no Actions
//! minutes anywhere in between — the master holds the source and hands it out.

mod agent;
mod args;
mod compile;
mod master;
mod package;
mod source;
mod submit;

use std::process::ExitCode;

const USAGE: &str = "\
noro-build-agent run    [--name NAME] [--targets T1,T2] [--work DIR]
noro-build-agent submit [--repo DIR] [--targets T1,T2] [--deploy core|all] [--no-wait]

Environment: NORO_MASTER_URL, NORO_ADMIN_TOKEN (needs noro.admin.launcher.publish,
plus .deploy for --deploy). Without --targets, `run` builds what this host can
build natively and `submit` asks for every target an online agent offers.";

#[tokio::main]
async fn main() -> ExitCode {
    let mut argv = std::env::args().skip(1);
    let command = argv.next().unwrap_or_default();
    let parsed = match args::Args::parse(argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let result = match command.as_str() {
        "run" => agent::run(parsed).await,
        "submit" => submit::run(parsed).await,
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
