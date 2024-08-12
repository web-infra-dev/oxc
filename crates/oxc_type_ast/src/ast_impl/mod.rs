use oxc_syntax::types::{GetTypeId, TypeId};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

mod literal;

impl GetTypeId for Type<'_> {
    fn type_id(&self) -> TypeId {
        match self {
            Self::Literal(ty) => ty.type_id(),
            Self::Freshable(ty) => ty.type_id(),
        }
    }
}

impl GetTypeId for FreshableType<'_> {
    fn type_id(&self) -> TypeId {
        match self {
            Self::Fresh(f) => f.type_id(),
            Self::Regular(r) => r.type_id(),
        }
    }
}
impl GetTypeId for FreshFreshableType<'_> {
    fn type_id(&self) -> TypeId {
        self.id
    }
}

impl GetTypeId for RegularFreshableType<'_> {
    fn type_id(&self) -> TypeId {
        self.id
    }
}
