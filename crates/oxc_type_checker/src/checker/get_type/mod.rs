//! Trying to get a type from something? It's here.
//!
//! Contains logic for constructing types, but not building or checking them.
//! Although `check*` functions create/get types, they live in the
//! [`check` module](crate::checker::check).  Type building logic (i.e.
//! allocating and putting types together) lives in
//! [`TypeBuilder`](crate::subsystem::TypeBuilder)

mod composite;
mod get_type_from_type_node;
mod literal;

pub(crate) use get_type_from_type_node::GetTypeFromTypeNode;
use oxc_ast::ast::TSType;
use oxc_syntax::types::TypeId;

use super::Checker;
pub(crate) use composite::UnionReduction;

impl<'a> Checker<'a> {
    /// ```typescript
    /// function getTypeFromTypeNode(node: TypeNode): Type {
    ///     return getConditionalFlowTypeOfType(getTypeFromTypeNodeWorker(node), node);
    /// }
    /// ```
    #[inline]
    pub fn get_type_from_type_node(&self, type_node: &TSType<'a>) -> TypeId {
        type_node.get_type_from_type_node(self)
    }
}
