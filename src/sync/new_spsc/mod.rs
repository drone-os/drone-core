//! Single-producer, single-consumer communication primitives.

macro_rules! load_atomic {
    ($atomic:expr, $ordering:ident) => {{
        #[cfg(not(any(feature = "_atomics", loom)))]
        {
            $atomic.load()
        }
        #[cfg(any(feature = "_atomics", loom))]
        {
            $atomic.load(core::sync::atomic::Ordering::$ordering)
        }
    }};
}

macro_rules! modify_atomic {
    ($atomic:expr, $ordering_read:ident, $ordering_cas:ident, | $old:ident | $new:expr) => {{
        #[cfg(not(any(feature = "_atomics", loom)))]
        {
            $atomic.modify(|$old| $new)
        }
        #[cfg(any(feature = "_atomics", loom))]
        loop {
            match $atomic.compare_exchange_weak(
                $old,
                $new,
                core::sync::atomic::Ordering::$ordering_cas,
                core::sync::atomic::Ordering::$ordering_read,
            ) {
                Ok(state) => break state,
                Err(state) => $old = state,
            }
        }
    }};
}

macro_rules! load_modify_atomic {
    ($atomic:expr, $ordering_read:ident, $ordering_cas:ident, | $old:ident | $new:expr) => {{
        #[cfg(not(any(feature = "_atomics", loom)))]
        {
            $atomic.modify(|$old| $new)
        }
        #[cfg(any(feature = "_atomics", loom))]
        {
            let mut $old = $atomic.load(core::sync::atomic::Ordering::$ordering_read);
            modify_atomic!($atomic, $ordering_read, $ordering_cas, |$old| $new)
        }
    }};
}

pub mod oneshot;
pub mod pulse;
pub mod ring;
