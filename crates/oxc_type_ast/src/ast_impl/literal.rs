use crate::{BigIntLiteralType, LiteralType, NumberLiteralType, StringLiteralType};
use oxc_syntax::types::{GetTypeId, TypeId};

impl GetTypeId for StringLiteralType<'_> {
    #[inline]
    fn type_id(&self) -> TypeId {
        self.id
    }
}

impl GetTypeId for NumberLiteralType {
    #[inline]
    fn type_id(&self) -> TypeId {
        self.id
    }
}

impl GetTypeId for BigIntLiteralType<'_> {
    #[inline]
    fn type_id(&self) -> TypeId {
        self.id
    }
}

impl GetTypeId for LiteralType<'_> {
    fn type_id(&self) -> TypeId {
        match self {
            Self::String(s) => s.id,
            Self::Number(n) => n.id,
            Self::BigInt(b) => b.id,
        }
    }
}
