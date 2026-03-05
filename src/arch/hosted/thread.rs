use std::cell::Cell;

thread_local! {
    static TP: Cell<usize> = Cell::new(0);
}

#[inline(always)]
pub fn get_thread_pointer() -> usize {
    TP.with(|tp| tp.get())
}

#[inline(always)]
pub unsafe fn set_thread_pointer(tp: usize) {
    TP.with(|val| val.set(tp));
}
