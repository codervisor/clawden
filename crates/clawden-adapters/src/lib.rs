mod docker_runtime;
#[cfg(feature = "nanoclaw")]
mod nanoclaw;
#[cfg(feature = "openclaw")]
mod openclaw;
#[cfg(feature = "openfang")]
mod openfang;
#[cfg(feature = "picoclaw")]
mod picoclaw;
mod registry;
#[cfg(feature = "zeroclaw")]
mod zeroclaw;

use std::sync::Arc;

#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

use clawden_core::ClawRuntime;

#[cfg(feature = "nanoclaw")]
pub use nanoclaw::NanoClawAdapter;
#[cfg(feature = "openclaw")]
pub use openclaw::OpenClawAdapter;
#[cfg(feature = "openfang")]
pub use openfang::OpenFangAdapter;
#[cfg(feature = "picoclaw")]
pub use picoclaw::PicoClawAdapter;
pub use registry::AdapterRegistry;
#[cfg(feature = "zeroclaw")]
pub use zeroclaw::ZeroClawAdapter;

/// Creates a registry pre-populated with all compile-time enabled adapters.
pub fn builtin_registry() -> AdapterRegistry {
    let mut registry = AdapterRegistry::new();

    #[cfg(feature = "openclaw")]
    registry.register(ClawRuntime::OpenClaw, Arc::new(OpenClawAdapter));

    #[cfg(feature = "openfang")]
    registry.register(ClawRuntime::OpenFang, Arc::new(OpenFangAdapter));

    #[cfg(feature = "zeroclaw")]
    registry.register(ClawRuntime::ZeroClaw, Arc::new(ZeroClawAdapter));

    #[cfg(feature = "picoclaw")]
    registry.register(ClawRuntime::PicoClaw, Arc::new(PicoClawAdapter));

    #[cfg(feature = "nanoclaw")]
    registry.register(ClawRuntime::NanoClaw, Arc::new(NanoClawAdapter));

    tracing::info!(
        adapter_count = registry.list().len(),
        "built-in adapter registry initialized"
    );
    registry
}

#[cfg(test)]
pub(crate) fn adapter_test_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("adapter test lock should not be poisoned")
}
