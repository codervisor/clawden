use anyhow::Result;
use clawden_core::ProcessManager;

use crate::commands::config_gen::cleanup_project_config_dir;
use crate::commands::up::{load_config, runtimes_from_config};
use crate::util::{append_audit_file, project_hash};

pub fn exec_down(
    process_manager: &ProcessManager,
    runtimes: Vec<String>,
    timeout: u64,
    remove_orphans: bool,
) -> Result<()> {
    let current_hash = project_hash()?;
    let owned = process_manager
        .list_processes()?
        .into_iter()
        .filter(|p| p.project_hash.as_deref() == Some(current_hash.as_str()))
        .map(|p| p.runtime)
        .collect::<Vec<_>>();

    if owned.is_empty() {
        println!("No project-owned runtimes found");
        return Ok(());
    }

    let declared = load_config()?
        .map(|cfg| runtimes_from_config(&cfg))
        .unwrap_or_default();

    let mut targets = if !runtimes.is_empty() {
        runtimes
            .into_iter()
            .filter(|name| owned.contains(name))
            .collect::<Vec<_>>()
    } else if declared.is_empty() {
        owned.clone()
    } else {
        declared
            .iter()
            .filter(|name| owned.contains(*name))
            .cloned()
            .collect::<Vec<_>>()
    };

    if remove_orphans {
        for runtime in &owned {
            if !targets.contains(runtime) && !declared.contains(runtime) {
                targets.push(runtime.clone());
            }
        }
    }

    targets.sort();
    targets.dedup();

    if targets.is_empty() {
        println!("No matching project runtimes to stop");
        return Ok(());
    }

    for runtime in &targets {
        let outcome = process_manager.stop_with_timeout(runtime, timeout)?;
        if outcome.forced {
            append_audit_file("runtime.force_kill", runtime, "ok")?;
        }
        append_audit_file("runtime.down", runtime, "ok")?;
        println!("Stopped {runtime}");
    }
    cleanup_project_config_dir(&current_hash)?;

    Ok(())
}
