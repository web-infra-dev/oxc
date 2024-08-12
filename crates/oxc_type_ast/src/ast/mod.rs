mod literal;

use std::cell::Cell;

use oxc_allocator::Box;
pub use literal::*;
use oxc_syntax::types::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EscapedStr<'a>(&'a str);
impl<'a> std::ops::Deref for EscapedStr<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[derive(Debug)]
pub enum Type<'a> {
    Literal(Box<'a, LiteralType<'a>>),
    Freshable(Box<'a, FreshableType<'a>>)
}

#[derive(Debug)]
pub enum FreshableType<'a> {
    Fresh(FreshFreshableType<'a>),
    Regular(RegularFreshableType<'a>)
}


#[derive(Debug)]
pub struct IntrinsicType <'a> {
    pub id: TypeId,
    pub name: &'a str,
    pub(crate) debug_name: Option<&'a str>
}

// TypeScript makes this same nominal distinction, and I'm not entirely sure
// why. I'm guessing it's for code clarity.
pub type NullableType<'a> = IntrinsicType<'a>;

#[derive(Debug)]
pub struct FreshFreshableType<'a> {
    pub ty: Type<'a>,
    pub regular: Cell<TypeId>
}

#[derive(Debug)]
pub struct RegularFreshableType<'a> {
    pub ty: Type<'a>,
    pub fresh: Cell<TypeId>
}
