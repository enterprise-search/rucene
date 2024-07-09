
/// IOContext holds additional details on the merge/search context and
/// specifies the context in which the Directory is being used for.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum IOContext {
    Merge(MergeInfo),
    Read(bool),
    Flush(FlushInfo),
    Default,
}

impl IOContext {
    pub const READ: IOContext = IOContext::Read(false);
    pub const READ_ONCE: IOContext = IOContext::Read(true);
    pub fn is_merge(&self) -> bool {
        match self {
            IOContext::Merge(_) => true,
            _ => false,
        }
    }
}

/// A FlushInfo provides information required for a FLUSH context.
///
/// It is used as part of an `IOContext` in case of FLUSH context.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct FlushInfo {
    num_docs: u32,
}

impl FlushInfo {
    pub fn new(num_docs: u32) -> Self {
        FlushInfo { num_docs }
    }
}

/// A MergeInfo provides information required for a MERGE context.
///
/// It is used as part of an `IOContext` in case of MERGE context.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct MergeInfo {
    total_max_doc: u32,
    estimated_merge_bytes: u64,
    is_external: bool,
    merge_max_num_segments: Option<u32>,
}

impl MergeInfo {
    pub fn new(
        total_max_doc: u32,
        estimated_merge_bytes: u64,
        is_external: bool,
        merge_max_num_segments: Option<u32>,
    ) -> Self {
        MergeInfo {
            total_max_doc,
            estimated_merge_bytes,
            is_external,
            merge_max_num_segments,
        }
    }
}
