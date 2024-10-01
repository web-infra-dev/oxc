use oxc_allocator::Vec;
use oxc_semantic::NodeId;
use oxc_syntax::types::{ObjectFlags, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexKind {
    String,
    Number,
}

#[derive(Debug)]
pub struct IndexInfo {
    key_type: TypeId,
    r#type: TypeId,
    is_readonly: bool,
    declaration: Option<NodeId>, // IndexSignatureDeclaration
}

// export interface ObjectType extends Type {
//     objectFlags: ObjectFlags;
//     /** @internal */ members?: SymbolTable;             // Properties by name
//     /** @internal */ properties?: Symbol[];             // Properties
//     /** @internal */ callSignatures?: readonly Signature[];      // Call signatures of type
//     /** @internal */ constructSignatures?: readonly Signature[]; // Construct signatures of type
//     /** @internal */ indexInfos?: readonly IndexInfo[];  // Index signatures
//     /** @internal */ objectTypeWithoutAbstractConstructSignatures?: ObjectType;
// }

#[derive(Debug)]
pub struct ObjectType<'a> {
    pub object_flags: ObjectFlags,
    pub(crate) index_infos: Vec<'a, IndexInfo>,
}

// should interface types go here too?
