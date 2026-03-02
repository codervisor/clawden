use anyhow::Result;
use clawden_core::ProcessManager;
use std::time::Duration;

use crate::commands::up::render_log_line;

pub async fn exec_logs(
    process_manager: &ProcessManager,
    runtimes: Vec<String>,
    tail: usize,
    follow: bool,
    timestamps: bool,
) -> Result<()> {
    let selected = if runtimes.is_empty() {
        process_manager
            .list_statuses()?
            .into_iter()
            .map(|s| s.runtime)
            .collect::<Vec<_>>()
    } else {
        runtimes
    };

    if selected.is_empty() {
        println!("No running runtimes");
        return Ok(());
    }

    for runtime in &selected {
        let logs = process_manager.tail_logs(runtime, tail)?;
        for line in logs.lines() {
            println!("{}", render_log_line(runtime, line, true, timestamps));
        }
    }

    if !follow {
        return Ok(());
    }

    println!("Following logs. Press Ctrl+C to stop.");
    let stream = process_manager.stream_logs(&selected)?;
    let mut tick = tokio::time::interval(Duration::from_millis(150));
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            _ = &mut ctrl_c => break,
            _ = tick.tick() => {
                while let Ok(line) = stream.receiver.try_recv() {
                    println!("{}", render_log_line(&line.runtime, &line.text, true, timestamps));
                }
            }
        }
    }

    Ok(())
}
