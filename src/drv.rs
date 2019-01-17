#[doc(hidden)]
#[macro_export]
macro_rules! res_decl {
  ($(#[$attr:meta])* $ty:ident, $get:ident) => {
    $(#[$attr])*
    fn $get(&self) -> &Self::$ty;
  };
  ($(#[$attr:meta])* $ty:ident, $get:ident, $get_mut:ident) => {
    $(#[$attr])*
    fn $get_mut(&mut self) -> &mut Self::$ty;
    res_decl!($(#[$attr])* $ty, $get);
  };
}

#[doc(hidden)]
#[macro_export]
macro_rules! res_impl {
  ($(#[$attr:meta])* $ty:ident, $get:ident, $($field:tt).*) => {
    $(#[$attr])*
    #[inline(always)]
    fn $get(&self) -> &Self::$ty { &self$(.$field)* }
  };
  ($(#[$attr:meta])* $ty:ident, $get:ident, $get_mut:ident, $($field:tt).*) => {
    $(#[$attr])*
    #[inline(always)]
    fn $get_mut(&mut self) -> &mut Self::$ty { &mut self$(.$field)* }
    res_impl!($(#[$attr])* $ty, $get, $($field).*);
  };
}
