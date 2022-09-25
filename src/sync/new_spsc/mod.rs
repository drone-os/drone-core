//! Single-producer, single-consumer communication primitives.

macro_rules! load_state {
    ($ptr:expr, $ordering:ident) => {{
        #[cfg(not(any(feature = "_atomics", loom)))]
        {
            $ptr.as_ref().state.load()
        }
        #[cfg(any(feature = "_atomics", loom))]
        {
            $ptr.as_ref().state.load(core::sync::atomic::Ordering::$ordering)
        }
    }};
}

macro_rules! modify_state {
    ($ptr:expr, $ordering_read:ident, $ordering_cas:ident, | $old:ident | $new:expr) => {{
        #[cfg(not(any(feature = "_atomics", loom)))]
        {
            $ptr.as_ref().state.modify(|$old| $new)
        }
        #[cfg(any(feature = "_atomics", loom))]
        loop {
            match $ptr.as_ref().state.compare_exchange_weak(
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

macro_rules! load_modify_state {
    ($ptr:expr, $ordering_read:ident, $ordering_cas:ident, | $old:ident | $new:expr) => {{
        #[cfg(not(any(feature = "_atomics", loom)))]
        {
            $ptr.as_ref().state.modify(|$old| $new)
        }
        #[cfg(any(feature = "_atomics", loom))]
        {
            let mut $old = $ptr.as_ref().state.load(core::sync::atomic::Ordering::$ordering_read);
            modify_state!($ptr, $ordering_read, $ordering_cas, |$old| $new)
        }
    }};
}

pub mod oneshot;
pub mod pulse;
