macro_rules! load_atomic {
    ($atomic:expr, $ordering:ident) => {{
        #[cfg(not(any(feature = "atomics", loom)))]
        {
            $atomic.load()
        }
        #[cfg(any(feature = "atomics", loom))]
        {
            $atomic.load(core::sync::atomic::Ordering::$ordering)
        }
    }};
}

macro_rules! store_atomic {
    ($atomic:expr, $value:expr, $ordering:ident) => {{
        #[cfg(not(any(feature = "atomics", loom)))]
        {
            $atomic.store($value)
        }
        #[cfg(any(feature = "atomics", loom))]
        {
            $atomic.store($value, core::sync::atomic::Ordering::$ordering)
        }
    }};
}

#[allow(unused_macros)]
macro_rules! modify_atomic {
    ($atomic:expr, $ordering_read:ident, $ordering_cas:ident, | $old:ident | $new:expr) => {{
        #[cfg(not(any(feature = "atomics", loom)))]
        {
            $atomic.modify(|$old| $new)
        }
        #[cfg(any(feature = "atomics", loom))]
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

#[allow(unused_macros)]
macro_rules! try_modify_atomic {
    ($atomic:expr, $ordering_read:ident, $ordering_cas:ident, | $old:ident | $new:expr) => {{
        #[cfg(not(any(feature = "atomics", loom)))]
        {
            $atomic.try_modify(|$old| $new)
        }
        #[cfg(any(feature = "atomics", loom))]
        loop {
            if let Some(new) = $new {
                match $atomic.compare_exchange_weak(
                    $old,
                    new,
                    core::sync::atomic::Ordering::$ordering_cas,
                    core::sync::atomic::Ordering::$ordering_read,
                ) {
                    Ok(state) => break Ok(state),
                    Err(state) => $old = state,
                }
            } else {
                break Err($old);
            }
        }
    }};
}

macro_rules! load_modify_atomic {
    ($atomic:expr, $ordering_read:ident, $ordering_cas:ident, | $old:ident | $new:expr) => {{
        #[cfg(not(any(feature = "atomics", loom)))]
        {
            $atomic.modify(|$old| $new)
        }
        #[cfg(any(feature = "atomics", loom))]
        {
            let mut $old = $atomic.load(core::sync::atomic::Ordering::$ordering_read);
            modify_atomic!($atomic, $ordering_read, $ordering_cas, |$old| $new)
        }
    }};
}

macro_rules! load_try_modify_atomic {
    ($atomic:expr, $ordering_read:ident, $ordering_cas:ident, | $old:ident | $new:expr) => {{
        #[cfg(not(any(feature = "atomics", loom)))]
        {
            $atomic.try_modify(|$old| $new)
        }
        #[cfg(any(feature = "atomics", loom))]
        {
            let mut $old = $atomic.load(core::sync::atomic::Ordering::$ordering_read);
            try_modify_atomic!($atomic, $ordering_read, $ordering_cas, |$old| $new)
        }
    }};
}

macro_rules! maybe_const_fn {
    ($(#[$($attr:tt)*])* $vis:vis const fn $name:ident($($args:tt)*) -> $ret:ty { $($body:tt)* }) => {
        #[cfg(not(loom))]
        $(#[$($attr)*])* $vis const fn $name($($args)*) -> $ret { $($body)* }
        #[cfg(loom)]
        $(#[$($attr)*])* $vis fn $name($($args)*) -> $ret { $($body)* }
    };
}
