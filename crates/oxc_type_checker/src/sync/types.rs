use oxc_semantic::SymbolId;
use oxc_span::CompactStr;
use oxc_syntax::types::{ObjectFlags, TypeFlags, TypeId};
use std::sync::{Arc, OnceLock};

// TODO: make this sync + send?
#[derive(Debug)]
pub struct Type {
    id: TypeId,
    flags: TypeFlags,
    // pattern?: DestructuringPattern,
    alias_symbol: Option<SymbolId>,
    // should we store Rc<Vec<Type>> or Rc<Vec<TypeId>>?
    alias_type_arguments: Option<Arc<Vec<Type>>>,
    /// Cached widend form of the type
    // widened: //Cell<Option<Rc<Type<'a>>>>,
    // widened: OnceCell<Rc<Type<'a>>>,
    widened: OnceLock<Arc<Type>>,
    kind: TypeKind,
}

#[derive(Debug)]
pub enum TypeKind {
    Intrinsic(Box<IntrinsicType>),
    Literal(Box<LiteralType>),
    Union(Box<UnionType>),
}

#[derive(Debug)]
pub struct IntrinsicType {
    pub name: &'static str,
    // TODO: optimize size by removing debug_name in release builds?
    // #[cfg(debug_assertions)]
    pub(crate) debug_name: Option<&'static str>,
    pub object_flags: ObjectFlags,
    // TODO: freshability
}
impl IntrinsicType {
    pub fn debug_name(&self) -> &'static str {
        self.debug_name.unwrap_or(self.name)
    }
}

#[derive(Debug)]
pub struct UnionType {
    pub types: Vec<Arc<Type>>,
    pub object_flags: ObjectFlags,
}

#[derive(Debug)]
pub enum LiteralType {
    String(StringLiteralType),
    Number(NumberLiteralType),
    BigInt(BigIntLiteralType),
}

#[derive(Debug)]
pub struct StringLiteralType {
    pub value: CompactStr,
}

#[derive(Debug)]
pub struct NumberLiteralType {
    pub value: f64,
}

#[derive(Debug)]
pub struct BigIntLiteralType {
    /// base-10 string representation of the BigInt
    pub raw: CompactStr,
}
