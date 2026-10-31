//! Windows: the cores the process may run on, from its affinity mask, for the renderer's worker
//! count (`vectorcraft_render::set_usable_cores`). `available_parallelism` counts every processor
//! in the machine, also when the process may run on fewer (started with `start /affinity`, or
//! pinned by a launcher). The desktop app and `vectorcraft-cli` (exports, MCP) both install it; the
//! CLI shares this file.
//!
//! `GetProcessAffinityMask` is a C API with no safe binding, hence the scoped `unsafe_code`
//! allowance.
#![allow(unsafe_code)]

use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessAffinityMask};

/// Tells the renderer how many cores the process may run on.
pub fn install() {
    vectorcraft_render::set_usable_cores(usable_cores());
}

/// The number of cores in the process's affinity mask, or `None` when Windows reports none (a
/// process with threads in more than one processor group).
fn usable_cores() -> Option<usize> {
    let (mut process, mut system) = (0usize, 0usize);
    // SAFETY: `GetCurrentProcess` returns a pseudo-handle that needs no closing, and both pointers
    // point at live `usize`s for the length of the call.
    unsafe { GetProcessAffinityMask(GetCurrentProcess(), &mut process, &mut system) }.ok()?;
    (process != 0).then(|| process.count_ones() as usize)
}

#[cfg(test)]
mod tests {
    /// A process that may run on one core renders on the calling thread. With workers, the
    /// renderer's flush spins on that core while the workers it waits for get no time, and a render
    /// crawls or stalls.
    #[test]
    fn a_process_limited_to_one_core_starts_no_render_workers() {
        const CHILD: &str = "VECTORCRAFT_TEST_ONE_CORE";
        const NAME: &str = "windows_cores::tests::a_process_limited_to_one_core_starts_no_render_workers";
        if std::env::var_os(CHILD).is_some() {
            assert_eq!(super::usable_cores(), Some(1));
            super::install();
            assert_eq!(vectorcraft_render::default_threads(), 0);
            return;
        }
        // `start /affinity 1` runs this test again in a process allowed only the first core.
        use std::os::windows::process::CommandExt;
        let exe = std::env::current_exe().unwrap();
        let output = std::process::Command::new("cmd")
            .raw_arg(format!("/C start \"\" /affinity 1 /wait /b \"{}\" --exact {NAME} --nocapture", exe.display()))
            .env(CHILD, "1")
            .env_remove("VECTORCRAFT_RENDER_THREADS")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success() && stdout.contains("test result: ok. 1 passed; 0 failed;"),
            "one-core test failed:\n{stdout}\n{}",
            String::from_utf8_lossy(&output.stderr),
        );
    }
}
