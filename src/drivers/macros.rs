#[macro_export]
macro_rules! res_reg_decl {
  ($ty:ident, $get:ident, $get_mut:ident) => {
    fn $get(&self) -> &Self::$ty;
    fn $get_mut(&mut self) -> &mut Self::$ty;
  }
}

#[macro_export]
macro_rules! res_reg_impl {
  ($ty:ident, $get:ident, $get_mut:ident, $reg:ident) => {
    #[inline(always)]
    fn $get(&self) -> &Self::$ty { &self.$reg }
    #[inline(always)]
    fn $get_mut(&mut self) -> &mut Self::$ty { &mut self.$reg }
  }
}

#[macro_export]
macro_rules! res_reg_field_impl {
  ($ty:ident, $get:ident, $get_mut:ident, $reg:ident, $field:ident) => {
    #[inline(always)]
    fn $get(&self) -> &Self::$ty { &self.$reg.$field }
    #[inline(always)]
    fn $get_mut(&mut self) -> &mut Self::$ty { &mut self.$reg.$field }
  }
}
