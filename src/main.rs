use std::io::{self, BufRead, Write};

fn worker_main() -> io::Result<()> {
    // Simple worker loop that reads JSON per line and evaluates using Boa
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Deserialize)]
    struct Job { id: u64, code: String, loop_limit: u64, recursion_limit: usize }

    #[derive(Serialize)]
    struct Res { id: u64, value: String, is_error: bool }

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut ctx: boa_engine::Context = boa_engine::Context::default();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        if let Ok(job) = serde_json::from_str::<Job>(&line) {
            // apply limits
            if job.loop_limit > 0 { ctx.runtime_limits_mut().set_loop_iteration_limit(job.loop_limit); }
            if job.recursion_limit < usize::MAX { ctx.runtime_limits_mut().set_recursion_limit(job.recursion_limit); }
            let res = match ctx.eval(boa_engine::Source::from_bytes(job.code.as_bytes())) {
                Ok(v) => Res { id: job.id, value: format!("{}", v.display()), is_error: false },
                Err(e) => Res { id: job.id, value: format!("Script thrown: {}", e), is_error: true },
            };
            let js = serde_json::to_string(&res).unwrap_or_else(|_| format!("{{\"id\":{},\"value\":\"serialization failed\",\"is_error\":true}}", job.id));
            writeln!(out, "{}", js)?;
            out.flush()?;
        } else {
            // ignore malformed lines
        }
    }
    Ok(())
}

fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    if let Some(a) = args.next() {
        if a == "--worker" {
            if let Err(e) = worker_main() {
                eprintln!("Worker failed: {}", e);
                std::process::exit(1);
            }
            return;
        }
    }
    println!("rfheadless: run with --worker to start a worker process");
}
