use crate::cap::{CNODE_SLOTS, CNode, CSPACE_CAP, CapPtr};
use crate::error::Error;
use crate::interface::{CSpaceProvider, CSpaceService};

const BITS_PER_WORD: usize = usize::BITS as usize;
const MAX_MANAGED_INDICES: usize = CNODE_SLOTS * (CNODE_SLOTS - 1);
const MAX_BITMAP_WORDS: usize = MAX_MANAGED_INDICES.div_ceil(BITS_PER_WORD);
const MAX_SUMMARY_WORDS: usize = MAX_BITMAP_WORDS.div_ceil(BITS_PER_WORD);

/// CSpaceManager manages the allocation of capability slots in a process's CSpace.
/// It uses a pure 2-level CNode hierarchy (Root CNode -> Level 1 CNodes).
/// Slots `l1_start_slot` to 255 in the Root CNode are used for indices of Level 1 CNodes.
/// Each Level 1 CNode provides 256 slots (0-255), except slot 0 which is reserved.
pub struct CSpaceManager {
    root_cnode: CNode,
    l1_start_slot: usize,
    next_index: usize,
    l1_cnodes: [bool; CNODE_SLOTS],
    free_bitmap: [usize; MAX_BITMAP_WORDS],
    free_summary: [usize; MAX_SUMMARY_WORDS],
    free_words: usize,
    free_summary_words: usize,
    free_count: usize,
    free_hint_word: usize,
}

impl CSpaceManager {
    pub fn new(root: CNode, l1_start_slot: usize) -> Self {
        // next_index is now relative to l1_start_slot.
        // next_index = 1 means l1_start_slot * 256 + 1.
        let next_index = 1;
        let l1_cnodes = [false; CNODE_SLOTS];
        let max_indices = if l1_start_slot < CNODE_SLOTS {
            (CNODE_SLOTS - l1_start_slot) * (CNODE_SLOTS - 1)
        } else {
            0
        };
        let free_words = if max_indices == 0 { 0 } else { max_indices.div_ceil(BITS_PER_WORD) };
        let free_summary_words =
            if free_words == 0 { 0 } else { free_words.div_ceil(BITS_PER_WORD) };
        Self {
            root_cnode: root,
            l1_start_slot,
            next_index,
            l1_cnodes,
            free_bitmap: [0; MAX_BITMAP_WORDS],
            free_summary: [0; MAX_SUMMARY_WORDS],
            free_words,
            free_summary_words,
            free_count: 0,
            free_hint_word: 0,
        }
    }

    fn max_indices(&self) -> usize {
        if self.l1_start_slot >= CNODE_SLOTS {
            0
        } else {
            (CNODE_SLOTS - self.l1_start_slot) * (CNODE_SLOTS - 1)
        }
    }

    fn bitmap_pos(&self, index: usize) -> Option<(usize, usize)> {
        if index == 0 || index > self.max_indices() {
            return None;
        }
        let bit = index - 1;
        Some((bit / BITS_PER_WORD, bit % BITS_PER_WORD))
    }

    fn summary_pos(&self, word_idx: usize) -> Option<(usize, usize)> {
        if word_idx >= self.free_words {
            return None;
        }
        Some((word_idx / BITS_PER_WORD, word_idx % BITS_PER_WORD))
    }

    fn set_summary_present(&mut self, word_idx: usize) {
        let Some((summary_word_idx, summary_bit_idx)) = self.summary_pos(word_idx) else {
            return;
        };
        self.free_summary[summary_word_idx] |= 1usize << summary_bit_idx;
    }

    fn clear_summary_present(&mut self, word_idx: usize) {
        let Some((summary_word_idx, summary_bit_idx)) = self.summary_pos(word_idx) else {
            return;
        };
        self.free_summary[summary_word_idx] &= !(1usize << summary_bit_idx);
    }

    fn mark_free(&mut self, index: usize) {
        let Some((word_idx, bit_idx)) = self.bitmap_pos(index) else {
            return;
        };
        let mask = 1usize << bit_idx;
        if (self.free_bitmap[word_idx] & mask) == 0 {
            self.free_bitmap[word_idx] |= mask;
            self.set_summary_present(word_idx);
            self.free_count += 1;
            if word_idx < self.free_hint_word {
                self.free_hint_word = word_idx;
            }
        }
    }

    /// Mark a L1 CNode as already present. Used during initialization.
    pub fn mark_present(&mut self, l0_idx: usize) {
        if l0_idx < CNODE_SLOTS {
            self.l1_cnodes[l0_idx] = true;
        }
    }

    fn get_absolute_cptr(&self, cptr: CapPtr) -> CapPtr {
        if self.root_cnode == CSPACE_CAP {
            cptr
        } else {
            CapPtr::concat(self.root_cnode.cap(), cptr)
        }
    }

    /// Converts a logical index to a capability pointer.
    /// Index 0 is reserved for metadata and should not be allocated for regular capabilities.
    /// Each Level 1 CNode's slot 0 is also skipped.
    /// The index calculations start from the first L1 CNode pointed to by `l1_start_slot`.
    pub fn index_to_cptr(&self, index: usize) -> CapPtr {
        // Calculate internal slot index within L1 CNodes, skipping slot 0 of each L1.
        // Effective slots per L1 = CNODE_SLOTS - 1
        let slots_per_l1 = CNODE_SLOTS - 1;

        // index 1 maps to (l0 = l1_start_slot, l1 = 1)
        // index 2 maps to (l0 = l1_start_slot, l1 = 2)
        // ...
        // index 255 maps to (l0 = l1_start_slot, l1 = 255)
        // index 256 maps to (l0 = l1_start_slot + 1, l1 = 1)

        let l1_offset = (index - 1) / slots_per_l1;
        let l1_slot = (index - 1) % slots_per_l1 + 1; // Start from slot 1

        let l0_idx = self.l1_start_slot + l1_offset;
        let l1_idx = l1_slot;

        let l0 = CapPtr::from(l0_idx);
        let l1 = CapPtr::from(l1_idx);
        self.get_absolute_cptr(CapPtr::concat(l0, l1))
    }

    /// Converts a capability pointer back to a logical index.
    pub fn cptr_to_index(&self, cptr: CapPtr) -> Option<usize> {
        let rel_ptr = CapPtr::relative(self.root_cnode.cap(), cptr);
        if rel_ptr.is_null() {
            return None;
        }
        // CSpaceManager only manages exact 2-level descendants under its root.
        // Reject deeper paths to avoid cross-manager free-list corruption.
        if rel_ptr.len() != 2 {
            return None;
        }
        let l0 = rel_ptr.level_idx(0);
        let l1 = rel_ptr.level_idx(1);

        if l0 < self.l1_start_slot || l1 == 0 {
            return None;
        }

        let l1_offset = l0 - self.l1_start_slot;
        let slots_per_l1 = CNODE_SLOTS - 1;
        Some(l1_offset * slots_per_l1 + l1)
    }

    pub fn owns_slot(&self, slot: CapPtr) -> bool {
        self.cptr_to_index(slot).is_some()
    }

    /// Proactively ensures that the L1 CNode for a future index is mapped.
    fn ensure_margin(&mut self, provider: &mut dyn CSpaceProvider) {
        let index_to_check = self.next_index + 16;
        let slots_per_l1 = CNODE_SLOTS - 1;
        let l1_offset = (index_to_check - 1) / slots_per_l1;
        if (self.l1_start_slot + l1_offset) < CNODE_SLOTS {
            let _ = self.ensure_l1_cnode(provider, index_to_check);
        }
    }

    fn ensure_l1_cnode(
        &mut self,
        provider: &mut dyn CSpaceProvider,
        index: usize,
    ) -> Result<(), Error> {
        let slots_per_l1 = CNODE_SLOTS - 1;
        let l1_offset = (index - 1) / slots_per_l1;
        let l0_idx = self.l1_start_slot + l1_offset;

        if l0_idx >= CNODE_SLOTS {
            crate::error!("CSpaceManager: index {} out of bounds", index);
            return Err(Error::CNodeFull);
        }

        if !self.l1_cnodes[l0_idx] {
            let dest = self.get_absolute_cptr(CapPtr::from(l0_idx));
            provider.alloc_cnode(dest)?;
            self.l1_cnodes[l0_idx] = true;
        }
        Ok(())
    }

    fn pop_reusable_index(&mut self) -> Option<usize> {
        if self.free_count == 0 || self.free_words == 0 {
            return None;
        }

        let mut word_idx =
            self.find_non_empty_word(self.free_hint_word.min(self.free_words - 1))?;
        for _ in 0..self.free_words {
            let word = self.free_bitmap[word_idx];
            if word != 0 {
                let bit_idx = word.trailing_zeros() as usize;
                let mask = 1usize << bit_idx;
                self.free_bitmap[word_idx] &= !mask;
                if self.free_bitmap[word_idx] == 0 {
                    self.clear_summary_present(word_idx);
                }
                self.free_count = self.free_count.saturating_sub(1);

                let index = word_idx * BITS_PER_WORD + bit_idx + 1;
                if index <= self.max_indices() {
                    self.free_hint_word = word_idx;
                    return Some(index);
                }
            }
            let next_word = (word_idx + 1) % self.free_words;
            if let Some(found) = self.find_non_empty_word(next_word) {
                word_idx = found;
            } else {
                break;
            }
        }
        self.free_hint_word = 0;
        None
    }

    fn find_non_empty_word(&self, start_word: usize) -> Option<usize> {
        if self.free_summary_words == 0 {
            return None;
        }

        let start_summary_word = start_word / BITS_PER_WORD;
        let start_summary_bit = start_word % BITS_PER_WORD;

        // First pass: [start_summary_word, end)
        for summary_word_idx in start_summary_word..self.free_summary_words {
            let mut summary = self.free_summary[summary_word_idx];
            if summary_word_idx == start_summary_word {
                summary &= usize::MAX << start_summary_bit;
            }
            if summary == 0 {
                continue;
            }
            let bit_idx = summary.trailing_zeros() as usize;
            let word_idx = summary_word_idx * BITS_PER_WORD + bit_idx;
            if word_idx < self.free_words {
                return Some(word_idx);
            }
        }

        // Second pass: [0, start_summary_word)
        for summary_word_idx in 0..start_summary_word {
            let summary = self.free_summary[summary_word_idx];
            if summary == 0 {
                continue;
            }
            let bit_idx = summary.trailing_zeros() as usize;
            let word_idx = summary_word_idx * BITS_PER_WORD + bit_idx;
            if word_idx < self.free_words {
                return Some(word_idx);
            }
        }

        None
    }

    /// Allocate a slot directly without calling a provider.
    pub fn alloc_direct(&mut self) -> Result<CapPtr, Error> {
        if let Some(index) = self.pop_reusable_index() {
            return Ok(self.index_to_cptr(index));
        }

        let index = self.next_index;
        self.next_index += 1;

        let slots_per_l1 = CNODE_SLOTS - 1;
        let l1_offset = (index - 1) / slots_per_l1;
        let l0_idx = self.l1_start_slot + l1_offset;

        if !self.l1_cnodes[l0_idx] {
            // Cannot allocate if L1 CNode is missing and no provider is available
            self.next_index = self.next_index.saturating_sub(1);
            crate::error!("CSpaceManager: L1 CNode for index {} not allocated", index);
            return Err(Error::CNodeFull);
        }

        let ptr = self.index_to_cptr(index);
        Ok(ptr)
    }
}

impl CSpaceService for CSpaceManager {
    fn alloc(&mut self, provider: &mut dyn CSpaceProvider) -> Result<CapPtr, Error> {
        self.ensure_margin(provider);

        if let Some(index) = self.pop_reusable_index() {
            self.ensure_l1_cnode(provider, index)?;
            return Ok(self.index_to_cptr(index));
        }

        let index = self.next_index;
        self.next_index += 1;
        if let Err(e) = self.ensure_l1_cnode(provider, index) {
            self.next_index = self.next_index.saturating_sub(1);
            return Err(e);
        }

        let ptr = self.index_to_cptr(index);
        Ok(ptr)
    }

    fn reserve_slots(
        &mut self,
        provider: &mut dyn CSpaceProvider,
        count: usize,
    ) -> Result<(), Error> {
        for i in 0..count {
            let index = self.next_index + i;
            let slots_per_l1 = CNODE_SLOTS - 1;
            let l1_offset = (index - 1) / slots_per_l1;
            if (self.l1_start_slot + l1_offset) >= CNODE_SLOTS {
                crate::error!(
                    "CSpaceManager: cannot reserve {} slots, index {} out of bounds",
                    count,
                    index
                );
                return Err(Error::CNodeFull);
            }
            self.ensure_l1_cnode(provider, index)?;
        }
        Ok(())
    }

    fn free(&mut self, slot: CapPtr) {
        if let Some(index) = self.cptr_to_index(slot) {
            if index >= self.next_index {
                return;
            }
            self.mark_free(index);
        }
    }
}
