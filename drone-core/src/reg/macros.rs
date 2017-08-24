/// Defines and implements a register type.
#[macro_export]
macro_rules! define_reg {
  (name => $name:ty, addr => $addr:expr) => {
    impl<A> $crate::reg::Delegate<$name, A> for ::reg::Reg<$name, A> {
      type Pointer = ::reg::Pointer<$name, A>;
      const ADDRESS: usize = $addr;
    }

    impl<A> $crate::reg::ValuePointer<$name, A> for ::reg::Pointer<$name, A> {
      type Value = ::reg::Value<$name>;
    }
  };

  (name => $name:ty, alias => $alias:expr) => {
    impl $crate::reg::RegionAlias<$name> for ::reg::Alias<$name> {
      const BASE: usize = $alias;

      unsafe fn new(address: usize) -> ::reg::Alias<$name> {
        ::reg::Alias {
          address: Self::alias_base(address),
          register: ::core::marker::PhantomData,
        }
      }
    }

    impl<A> $crate::reg::AliasPointer<$name, A> for ::reg::Pointer<$name, A> {
      type Alias = ::reg::Alias<$name>;
    }
  };

  (name => $name:ty, bits => $bits:ident<$($t:ty),*>) => {
    impl $bits<$($t,)* $crate::reg::marker::Value> for ::reg::Value<$name> {}
    impl $bits<$($t,)* $crate::reg::marker::Alias> for ::reg::Alias<$name> {}
  };

  (name => $name:ty, bits => $bits:ident) => {
    impl $bits<$crate::reg::marker::Value> for ::reg::Value<$name> {}
    impl $bits<$crate::reg::marker::Alias> for ::reg::Alias<$name> {}
  };

  (name => $name:ident, desc => $desc:expr) => {
    #[doc = $desc]
    pub struct $name;
  };

  (
    name => $name:ident$( => $bits:ident)*,
    $($key:ident => $value:expr,)+
  ) => {
    $(define_reg!(name => $name, $key => $value);)+
    $(define_reg!(name => $name, bits => $bits);)*
  };

  (
    type => $type:ty$( => $bits:ident<$($t:ty),*>)*,
    $($key:ident => $value:expr,)+
  ) => {
    $(define_reg!(name => $type, $key => $value);)+
    $(define_reg!(name => $type, bits => $bits<$($t),*>);)*
  };
}

/// Defines and implements concrete register types.
#[macro_export]
macro_rules! define_reg_structs {
  () => {
    /// Register delegate type.
    ///
    /// It holds nothing, but can be converted to a concrete pointer.
    pub struct Reg<R, A> {
      register: PhantomData<R>,
      atomicity: PhantomData<A>,
    }

    /// Register pointer type.
    pub struct Pointer<R, A> {
      address: usize,
      register: PhantomData<R>,
      atomicity: PhantomData<A>,
    }

    /// Register value type.
    pub struct Value<R> {
      value: u32,
      register: PhantomData<R>,
    }

    /// Register bit-band alias type.
    pub struct Alias<R> {
      address: usize,
      register: PhantomData<R>,
    }

    /// Thread-unsafe register delegate type.
    pub type Sreg<R> = Reg<R, $crate::reg::marker::Single>;

    /// Thread-safe register delegate type.
    pub type Areg<R> = Reg<R, $crate::reg::marker::Atomic>;

    impl<R, A> Reg<R, A> {
      pub const fn new() -> Reg<R, A> {
        Reg {
          register: PhantomData,
          atomicity: PhantomData,
        }
      }
    }

    impl<R, A> RawPointer<R, A> for Pointer<R, A> {
      unsafe fn new(address: usize) -> Pointer<R, A> {
        Pointer {
          address,
          register: PhantomData,
          atomicity: PhantomData,
        }
      }

      fn get(&self) -> usize {
        self.address
      }
    }

    impl<R> RawValue<R> for Value<R> {
      fn new(value: u32) -> Value<R> {
        Value {
          value,
          register: PhantomData,
        }
      }

      fn get(&self) -> u32 {
        self.value
      }

      fn set(&mut self, value: u32) -> &mut Value<R> {
        self.value = value;
        self
      }
    }

    impl<R> RawAlias<R> for Alias<R>
    where
      Alias<R>: RegionAlias<R>,
    {
      fn get(&self) -> usize {
        self.address
      }
    }
  };
}
