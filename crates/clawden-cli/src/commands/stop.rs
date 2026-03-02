use anyhow::Result;
use clawden_core::ProcessManager;

use crate::util::append_audit_file;

pub fn exec_stop(
    process_manager: &ProcessManager,
    runtime: Option<String>,
    timeout: u64,
) -> Result<()> {
    if let Some(rt) = runtime {
        println!("Stopping {}...", rt);
        let outcome = process_manager.stop_with_timeout(&rt, timeout)?;
        if outcome.forced {
            append_audit_file("runtime.force_kill", &rt, "ok")?;
        }
        append_audit_file("runtime.stop", &rt, "ok")?;
        return Ok(());
    }

    println!("Stopping all runtimes...");
    for status in process_manager.list_statuses()? {
        let outcome = process_manager.stop_with_timeout(&status.runtime, timeout)?;
        if outcome.forced {
            append_audit_file("runtime.force_kill", &status.runtime, "ok")?;
        }
        append_audit_file("runtime.stop", &status.runtime, "ok")?;
        println!("Stopped {}", status.runtime);
    }
    Ok(())
}
