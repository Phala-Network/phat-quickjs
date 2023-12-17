use qjsbind as js;

use js::ToJsValue;

use dlmalloc::GlobalDlmalloc;
use phala_allocator::StatSizeAllocator;

#[global_allocator]
static ALLOCATOR: StatSizeAllocator<GlobalDlmalloc> = StatSizeAllocator::new(GlobalDlmalloc);

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

pub(crate) fn setup(ns: &js::Value) -> js::Result<()> {
    ns.define_property_fn("memoryStats", mem_stats)?;
    Ok(())
}

#[js::host_call]
fn mem_stats() -> Stats {
    ALLOCATOR.stats().into()
}
