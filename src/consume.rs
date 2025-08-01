use std::sync::atomic::Ordering;
// I'm aware of silly issues with dependency tracking and things like
// f = load_consume(...); *a[f - f]; that isn't actually consume
// This project uses it exclusively for things like b = *a, c = *b

#[cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "arm"
))]
mod can_consume {
    use std::sync::atomic::Ordering;
    pub const CONSUME: Ordering = Ordering::Relaxed;
}

#[cfg(not(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "arm"
)))]
mod can_consume {
    use std::sync::atomic::Ordering;
    pub const CONSUME: Ordering = Ordering::Acquire;
}

pub const CONSUME: Ordering = can_consume::CONSUME;
