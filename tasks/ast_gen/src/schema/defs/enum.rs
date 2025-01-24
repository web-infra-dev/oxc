use std::iter::FusedIterator;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::utils::create_ident;

use super::{Def, Derives, FieldDef, FileId, Layout, Schema, TypeDef, TypeId};

pub type Discriminant = u8;

/// Type definition for an enum.
#[derive(Debug)]
pub struct EnumDef {
    pub id: TypeId,
    pub name: String,
    pub has_lifetime: bool,
    pub file_id: FileId,
    pub generated_derives: Derives,
    pub variants: Vec<VariantDef>,
    /// For `@inherits` inherited enum variants
    pub inherits: Vec<TypeId>,
    pub is_visited: bool,
    pub has_kind: bool,
    pub layout: Layout,
}

impl EnumDef {
    /// Create new [`EnumDef`].
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        id: TypeId,
        name: String,
        has_lifetime: bool,
        file_id: FileId,
        generated_derives: Derives,
        variants: Vec<VariantDef>,
        inherits: Vec<TypeId>,
        is_visited: bool,
    ) -> Self {
        Self {
            id,
            name,
            has_lifetime,
            file_id,
            generated_derives,
            variants,
            inherits,
            is_visited,
            has_kind: false,
            layout: Layout::default(),
        }
    }

    /// Get reference to [`VariantDef`] by its variant index.
    pub fn variant(&self, variant_index: usize) -> &VariantDef {
        &self.variants[variant_index]
    }

    /// Get mutable reference to [`VariantDef`] by its variant index.
    #[expect(dead_code)]
    pub fn variant_mut(&mut self, variant_index: usize) -> &mut VariantDef {
        &mut self.variants[variant_index]
    }

    /// Get iterator over all enum's variants (including inherited)
    pub fn all_variants<'s>(&'s self, schema: &'s Schema) -> AllVariantsIter<'s> {
        AllVariantsIter::new(self, schema)
    }

    /// Get whether all variants are fieldless.
    pub fn is_fieldless(&self) -> bool {
        // All AST enums are `#[repr(C, u8)]` or `#[repr(u8)]`.
        // Such enums must have at least 1 variant, so only way can have size 1 is if all variants
        // are fieldless.
        self.layout.layout_64.size == 1
    }
}

impl Def for EnumDef {
    /// Get [`TypeId`] for type.
    fn id(&self) -> TypeId {
        self.id
    }

    /// Get type name.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get if type has a lifetime.
    fn has_lifetime(&self, _schema: &Schema) -> bool {
        self.has_lifetime
    }

    /// Get type signature (including lifetime).
    /// Lifetime is anonymous (`'_`) if `anon` is true.
    fn ty_with_lifetime(&self, schema: &Schema, anon: bool) -> TokenStream {
        let ident = self.ident();
        let lifetime = self.lifetime_maybe_anon(schema, anon);
        quote!( #ident #lifetime )
    }

    /// Get inner type.
    ///
    /// Enums don't have a single inner type, so returns `None`.
    fn inner_type<'s>(&self, _schema: &'s Schema) -> Option<&'s TypeDef> {
        None
    }

    /// Get whether type is visited.
    ///
    /// Returns `true` if type is tagged `#[ast(visit)]`.
    fn is_visited(&self) -> bool {
        self.is_visited
    }

    /// Get whether type has `AstKind`.
    fn has_kind(&self) -> bool {
        self.has_kind
    }

    /// Get type's layout.
    fn layout(&self) -> &Layout {
        &self.layout
    }
}

#[derive(Debug)]
pub struct VariantDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub discriminant: Discriminant,
}

impl VariantDef {
    /// Get variant name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get variant name as an `Ident`.
    pub fn ident(&self) -> Ident {
        create_ident(self.name())
    }

    /// Get variant's field.
    ///
    /// Returns `None` if variant is fieldless.
    ///
    /// # Panics
    /// Panics if variant has more than 1 field.
    pub fn field(&self) -> Option<&FieldDef> {
        if self.fields.is_empty() {
            None
        } else {
            assert!(self.fields.len() == 1);
            Some(&self.fields[0])
        }
    }
}

/// Iterator over all variants of an enum (including inherited).
pub struct AllVariantsIter<'s> {
    schema: &'s Schema,
    variants_iter: std::slice::Iter<'s, VariantDef>,
    inherits_iter: std::slice::Iter<'s, TypeId>,
    inner_iter: Option<Box<AllVariantsIter<'s>>>,
}

impl<'s> AllVariantsIter<'s> {
    /// Create new [`AllVariantsIter`].
    fn new(def: &'s EnumDef, schema: &'s Schema) -> Self {
        let variants_iter = def.variants.iter();
        let inherits_iter = def.inherits.iter();
        Self { schema, variants_iter, inherits_iter, inner_iter: None }
    }
}

impl<'s> Iterator for AllVariantsIter<'s> {
    type Item = &'s VariantDef;

    fn next(&mut self) -> Option<Self::Item> {
        // Yield own variants first
        if let Some(variant) = self.variants_iter.next() {
            return Some(variant);
        }

        // Yield from inner iterator (iterating over inherited type's variants)
        if let Some(inner_iter) = &mut self.inner_iter {
            if let Some(variant) = inner_iter.next() {
                return Some(variant);
            }
            self.inner_iter = None;
        }

        // No current inner iterator. Start iterating over next inherited type.
        if let Some(&inherits_type_id) = self.inherits_iter.next() {
            let inherited_type = self.schema.enum_def(inherits_type_id);
            let inner_iter = inherited_type.all_variants(self.schema);
            self.inner_iter = Some(Box::new(inner_iter));
            Some(self.inner_iter.as_mut().unwrap().next().unwrap())
        } else {
            None
        }
    }
}

impl FusedIterator for AllVariantsIter<'_> {}
