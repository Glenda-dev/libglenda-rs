mod linked_list;
#[cfg(feature = "slab-allocator")]
mod slab;
#[cfg(feature = "talc-allocator")]
mod talc;

pub use linked_list::LockedLinkedListAllocator as LinkedListAllocator;
#[cfg(feature = "slab-allocator")]
pub use slab::SlabAllocator;
#[cfg(feature = "talc-allocator")]
pub use talc::TalcAllocator;

#[cfg(feature = "linked-list-allocator")]
pub use linked_list::LockedLinkedListAllocator as Allocator;
#[cfg(feature = "slab-allocator")]
pub use slab::SlabAllocator as Allocator;
#[cfg(feature = "talc-allocator")]
pub use talc::TalcAllocator as Allocator;
