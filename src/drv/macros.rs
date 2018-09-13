#[macro_export]
macro_rules! res_reg_decl {
  ($(#[$attr:meta])* $ty:ident, $get:ident, $get_mut:ident) => {
    $(#[$attr])*
    fn $get(&self) -> &Self::$ty;

    $(#[$attr])*
    fn $get_mut(&mut self) -> &mut Self::$ty;
  }
}

#[macro_export]
macro_rules! res_reg_impl {
  ($(#[$attr:meta])* $ty:ident, $get:ident, $get_mut:ident, $reg:ident) => {
    $(#[$attr])*
    #[inline(always)]
    fn $get(&self) -> &Self::$ty { &self.$reg }

    $(#[$attr])*
    #[inline(always)]
    fn $get_mut(&mut self) -> &mut Self::$ty { &mut self.$reg }
  }
}

#[macro_export]
macro_rules! res_reg_field_impl {
  (
    $(#[$attr:meta])*
    $ty:ident,
    $get:ident,
    $get_mut:ident,
    $reg:ident,
    $field:ident
  ) => {
    $(#[$attr])*
    #[inline(always)]
    fn $get(&self) -> &Self::$ty { &self.$reg.$field }

    $(#[$attr])*
    #[inline(always)]
    fn $get_mut(&mut self) -> &mut Self::$ty { &mut self.$reg.$field }
  }
}
