use js::ToJsValue;

use phala_allocator::StatSizeAllocator;
use std::alloc::System;

#[global_allocator]
static ALLOCATOR: StatSizeAllocator<System> = StatSizeAllocator::new(System);

#[derive(Debug, Clone, ToJsValue)]
pub struct Stats {
    pub current: usize,
    pub spike: usize,
    pub peak: usize,
}

impl From<phala_allocator::Stats> for Stats {
    fn from(stats: phala_allocator::Stats) -> Self {
        Self {
            current: stats.current,
            spike: stats.spike,
            peak: stats.peak,
        }
    }
}

pub(crate) fn setup(ns: &js::Value) -> anyhow::Result<()> {
    ns.define_property_fn("memoryStats", mem_stats)?;
    Ok(())
}

#[js::host_call]
fn mem_stats() -> Stats {
    ALLOCATOR.stats().into()
}
