//! Hooks stub - simplified for WikiMind

/// Hook abort signal stub
pub struct HookAbortSignal;

/// Hook progress reporter trait
pub trait HookProgressReporter {
    fn report(&self, message: &str);
}

/// Hook run result stub
pub enum HookRunResult {
    Completed,
    Skipped,
    Failed(String),
}

/// Hook runner stub
pub struct HookRunner;

impl HookRunner {
    pub fn new(_name: &str) -> Self {
        Self
    }

    pub fn run<R: HookProgressReporter>(&self, _reporter: &R) -> HookRunResult {
        HookRunResult::Completed
    }
}
