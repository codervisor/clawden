use anyhow::Result;
use clawden_core::{LifecycleManager, ProcessManager, RuntimeInstaller};

use super::up::{exec_up, UpOptions};

pub async fn exec_start(
    runtimes: Vec<String>,
    no_docker: bool,
    installer: &RuntimeInstaller,
    process_manager: &ProcessManager,
    manager: &mut LifecycleManager,
) -> Result<()> {
    exec_up(
        UpOptions {
            runtimes,
            detach: true,
            no_log_prefix: false,
            timeout: 10,
        },
        no_docker,
        installer,
        process_manager,
        manager,
    )
    .await
}
