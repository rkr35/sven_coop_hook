pub unsafe trait Yank<T> {
    unsafe fn yank(self) -> T;
}

unsafe impl<T> Yank<T> for Option<T> {
    unsafe fn yank(self) -> T {
        self.unwrap_or_else(|| std::hint::unreachable_unchecked())
    }
}
