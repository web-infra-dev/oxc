mod composite;
mod get_type_from_type_node;

pub(crate) use get_type_from_type_node::GetTypeFromTypeNode;
use oxc_ast::ast::TSType;
use oxc_syntax::types::TypeId;

use super::Checker;
pub(crate) use composite::UnionReduction;

impl<'a> Checker<'a> {
    #[inline]
    pub fn get_type_from_type_node(&self, type_node: &TSType<'a>) -> TypeId {
        type_node.get_type_from_type_node(self)
    }
}
