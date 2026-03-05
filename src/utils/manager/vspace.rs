use crate::arch::mem::{PGSIZE, PT_LEVELS};
use crate::cap::{CapPtr, Frame, PageTable, VSpace};
use crate::error::Error;
use crate::interface::vspace::{VSpaceService, VSpaceProvider};
use crate::interface::cspace::CSpaceService;
use crate::mem::Perms;

pub struct VSpaceManager {
    pub root: VSpace,
}

impl VSpaceManager {
    pub fn new(root: VSpace) -> Self {
        Self { root }
    }

    /// 确保中间页表存在
    fn ensure_tables(
        &mut self,
        vaddr: usize,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        for i in 1..PT_LEVELS {
            if !self.is_mapped(vaddr, i) {
                self.map_pt(i, vaddr, provider, slots)?;
            }
        }
        Ok(())
    }
}

impl VSpaceService for VSpaceManager {
    fn map_frame(
        &mut self,
        frame: Frame,
        vaddr: usize,
        perms: Perms,
        pages: usize,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
    ) -> Result<(), Error> {
        if vaddr % PGSIZE != 0 {
            return Err(Error::InvalidAddress);
        }

        self.ensure_tables(vaddr, provider, slots)?;
        self.root.map(frame, vaddr, perms, pages)
    }

    fn map_pt(
        &mut self,
        level: usize,
        vaddr: usize,
        provider: &mut dyn VSpaceProvider,
        slots: &mut dyn CSpaceService,
    ) -> Result<PageTable, Error> {
        let slot = slots.alloc(provider)?;
        provider.alloc_pagetable(slot)?;
        let pt = PageTable::from(slot);
        self.root.map_table(pt, vaddr, level)?;
        Ok(pt)
    }

    fn unmap(&mut self, vaddr: usize, pages: usize) -> Result<(), Error> {
        self.root.unmap(vaddr, pages * PGSIZE)
    }

    fn map_scratch(
        &mut self,
        _frame: Frame,
        _perms: Perms,
        _pages: usize,
        _provider: &mut dyn VSpaceProvider,
        _slots: &mut dyn CSpaceService,
    ) -> Result<usize, Error> {
        Err(Error::NotImplemented)
    }

    fn is_mapped(&self, _vaddr: usize, _level: usize) -> bool {
        // 在目前的 VSpace cap 接口中，没有直接查询是否已映射的方法。
        // 这里暂时返回 false 以触发 map_pt。
        // 如果底层 map_table 支持幂等或返回 AlreadyExists，则可以工作。
        false
    }
}
