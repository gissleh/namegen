/// A WorkingSet is a crucial part of this generator's performance. It is all local state required
/// to generate a name and get the output without performing additional allocations per generation
/// once the WorkingSet's underlying vectors have grown.
/// 
/// This also means that if you have a working set per thread, then generating names is completely
/// thread safe.
pub struct WorkingSet {
    pub result: Vec<usize>,
    pub result_str: String,
    pub stack: Vec<usize>,
    pub stack_pos: Vec<usize>,
}

impl WorkingSet {
    /// Get the results from the last generator call.
    /// If you need to keep it around, copy it to another
    /// string.
    #[inline]
    pub fn get_result(&self) -> &str {
        &self.result_str
    }

    pub fn new() -> WorkingSet {
        WorkingSet{
            result: Vec::with_capacity(16),
            result_str: String::with_capacity(16),
            stack: Vec::with_capacity(128),
            stack_pos: Vec::with_capacity(16),
        }
    }
}
