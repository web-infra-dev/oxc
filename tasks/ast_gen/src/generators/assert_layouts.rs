use std::cmp::{max, min};

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    output::{output_path, Output},
    schema::{
        Def, Discriminant, EnumDef, Layout, Niche, Offset, PlatformLayout, PrimitiveDef, Schema,
        StructDef, TypeDef, TypeId, Visibility,
    },
    Generator,
};

use super::define_generator;

pub struct AssertLayouts;

define_generator!(AssertLayouts);

impl Generator for AssertLayouts {
    fn modify(&self, schema: &mut Schema) {
        for type_id in 0..schema.defs.len() {
            calculate_layout(type_id, schema);
        }
    }

    fn generate(&self, schema: &Schema) -> Output {
        let (assertions_64, assertions_32) = schema
            .defs
            .iter()
            .map(generate_layout_assertions)
            .collect::<(Vec<TokenStream>, Vec<TokenStream>)>();

        let output = quote! {
            use std::mem::{align_of, offset_of, size_of};

            ///@@line_break
            use oxc_regular_expression::ast::*;

            ///@@line_break
            use crate::ast::*;

            ///@@line_break
            #[cfg(target_pointer_width = "64")]
            const _: () = { #(#assertions_64)* };

            ///@@line_break
            #[cfg(target_pointer_width = "32")]
            const _: () = { #(#assertions_32)* };

            ///@@line_break
            #[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
            const _: () = panic!("Platforms with pointer width other than 64 or 32 bit are not supported");
        };

        Output::Rust { path: output_path(crate::AST_CRATE, "assert_layouts.rs"), tokens: output }
    }
}

fn calculate_layout(type_id: TypeId, schema: &mut Schema) -> &Layout {
    let def = schema.def(type_id);
    if !def.layout().is_initialized() {
        match def {
            TypeDef::Struct(_) => {
                schema.def_struct_mut(type_id).layout = calculate_layout_struct(type_id, schema);
            }
            TypeDef::Enum(_) => {
                schema.def_enum_mut(type_id).layout = calculate_layout_enum(type_id, schema);
            }
            TypeDef::Primitive(def) => {
                schema.def_primitive_mut(type_id).layout = calculate_layout_primitive(def);
            }
            TypeDef::Option(_) => {
                schema.def_option_mut(type_id).layout = calculate_layout_option(type_id, schema);
            }
            TypeDef::Box(_) => {
                schema.def_box_mut(type_id).layout = calculate_layout_box();
            }
            TypeDef::Vec(_) => {
                schema.def_vec_mut(type_id).layout = calculate_layout_vec();
            }
            TypeDef::Cell(_) => {
                schema.def_cell_mut(type_id).layout = calculate_layout_cell(type_id, schema);
            }
        }
    }
    schema.def(type_id).layout()
}

fn calculate_layout_struct(type_id: TypeId, schema: &mut Schema) -> Layout {
    let mut layout_64 = PlatformLayout::from_size_align(0, 1);
    let mut layout_32 = PlatformLayout::from_size_align(0, 1);

    let def = schema.def_struct(type_id);
    for field_index in 0..def.fields.len() {
        let field_type_id = schema.def_struct(type_id).field(field_index).type_id;
        let field_layout = calculate_layout(field_type_id, schema);

        #[expect(clippy::items_after_statements)]
        fn update(layout: &mut PlatformLayout, field_layout: &PlatformLayout) -> u32 {
            // Field needs to be aligned
            let offset = layout.size.next_multiple_of(field_layout.align);

            // Update alignment
            layout.align = max(layout.align, field_layout.align);

            // Update niche.
            // Take the largest niche. Preference for earlier niche if 2 fields have niches of same size.
            if let Some(field_niche) = &field_layout.niche {
                if layout.niche.as_ref().is_none_or(|niche| field_niche.count > niche.count) {
                    let mut niche = field_niche.clone();
                    niche.offset += offset;
                    layout.niche = Some(niche);
                }
            }

            // Next field starts after this one
            layout.size = offset + field_layout.size;

            // Return offset of this field
            offset
        }

        let offset_64 = update(&mut layout_64, &field_layout.layout_64);
        let offset_32 = update(&mut layout_32, &field_layout.layout_32);

        // Store offset on `field`
        let field = schema.def_struct_mut(type_id).field_mut(field_index);
        field.offset = Offset { offset_64, offset_32 };
    }

    // Round up size to alignment
    layout_64.size = layout_64.size.next_multiple_of(layout_64.align);
    layout_32.size = layout_32.size.next_multiple_of(layout_32.align);

    Layout { layout_64, layout_32 }
}

fn calculate_layout_enum(type_id: TypeId, schema: &mut Schema) -> Layout {
    // `#[repr(C, u8)]` enums have alignment of highest-aligned variant.
    // Size is size of largest variant + alignment of most-aligned variant.
    // Fieldless `#[repr(u8)]` enums obey the same rules. Fieldless variants act as size 0, align 1.
    // `#[repr(C, u8)]` and `#[repr(u8)]` enums must always have at least one variant.
    struct State {
        min_discriminant: Discriminant,
        max_discriminant: Discriminant,
        layout_64: PlatformLayout,
        layout_32: PlatformLayout,
    }

    fn process_variants(type_id: TypeId, state: &mut State, schema: &mut Schema) {
        let State { min_discriminant, max_discriminant, layout_64, layout_32 } = state;

        let def = schema.def_enum(type_id);
        for variant_index in 0..def.variants.len() {
            let variant = schema.def_enum(type_id).variant(variant_index);

            *min_discriminant = min(*min_discriminant, variant.discriminant);
            *max_discriminant = max(*max_discriminant, variant.discriminant);

            if !variant.fields.is_empty() {
                assert_eq!(variant.fields.len(), 1);

                let variant_layout = calculate_layout(variant.fields[0].type_id, schema);

                layout_64.size = max(layout_64.size, variant_layout.layout_64.size);
                layout_64.align = max(layout_64.align, variant_layout.layout_64.align);
                layout_32.size = max(layout_32.size, variant_layout.layout_32.size);
                layout_32.align = max(layout_32.align, variant_layout.layout_32.align);
            }
        }

        let def = schema.def_enum(type_id);
        for inherits_index in 0..def.inherits.len() {
            let inherits_type_id = schema.def_enum(type_id).inherits[inherits_index];
            process_variants(inherits_type_id, state, schema);
        }
    }

    let mut state = State {
        min_discriminant: Discriminant::MAX,
        max_discriminant: 0,
        layout_64: PlatformLayout::from_size_align(0, 1),
        layout_32: PlatformLayout::from_size_align(0, 1),
    };
    process_variants(type_id, &mut state, schema);
    let State { min_discriminant, max_discriminant, mut layout_64, mut layout_32 } = state;

    layout_64.size += layout_64.align;
    layout_32.size += layout_32.align;

    // Any unused discriminant values at start of end of the range form a niche.
    // Note: The unused discriminants must be at start or end of range, *not* in the middle.
    // `#[repr(u8)] enum Foo { A = 0, B = 255 }` has no niche.
    // The largest available range (from start or from end) is used for the niche.
    let niches_start = min_discriminant;
    let niches_end = Discriminant::MAX - max_discriminant;

    if niches_start != 0 || niches_end != 0 {
        let is_range_start = niches_start >= niches_end;
        let count = u32::from(if is_range_start { niches_start } else { niches_end });
        let niche = Niche::new(0, 1, is_range_start, count);
        layout_64.niche = Some(niche.clone());
        layout_32.niche = Some(niche);
    }

    Layout { layout_64, layout_32 }
}

fn calculate_layout_option(type_id: TypeId, schema: &mut Schema) -> Layout {
    let def = schema.def_option(type_id);
    let inner_layout = calculate_layout(def.inner_type_id, schema);

    // `Option`s consume 1 niche if there is one, or add `bool` as discriminant before the inner type.
    // The discriminant has same niche as a `bool`.
    #[expect(clippy::items_after_statements)]
    fn consume_niche(layout: &mut PlatformLayout) {
        if let Some(niche) = &mut layout.niche {
            if niche.count == 1 {
                layout.niche = None;
            } else {
                niche.count -= 1;
            }
        } else {
            layout.size += layout.align;
            layout.niche = Some(Niche::new(0, 1, false, 254));
        }
    }

    let mut layout = inner_layout.clone();
    consume_niche(&mut layout.layout_64);
    consume_niche(&mut layout.layout_32);
    layout
}

fn calculate_layout_box() -> Layout {
    // `Box`es are pointer-sized, with a single niche (like `NonNull`)
    Layout {
        layout_64: PlatformLayout::from_size_align_niche(8, 8, Niche::new(0, 8, true, 1)),
        layout_32: PlatformLayout::from_size_align_niche(4, 4, Niche::new(0, 4, true, 1)),
    }
}

fn calculate_layout_vec() -> Layout {
    // `Vec`s contain 4 x pointer-sized fields.
    // They have a single niche on the first field - the pointer which is `NonNull`.
    Layout {
        layout_64: PlatformLayout::from_size_align_niche(32, 8, Niche::new(0, 8, true, 1)),
        layout_32: PlatformLayout::from_size_align_niche(16, 4, Niche::new(0, 4, true, 1)),
    }
}

fn calculate_layout_cell(type_id: TypeId, schema: &mut Schema) -> Layout {
    let def = schema.def_cell(type_id);
    let inner_layout = calculate_layout(def.inner_type_id, schema);

    // `Cell`s have same layout as inner type, but with no niche
    let mut layout = inner_layout.clone();
    layout.layout_64.niche = None;
    layout.layout_32.niche = None;
    layout
}

fn calculate_layout_primitive(def: &PrimitiveDef) -> Layout {
    let semantic_id_layout = Layout::from_size_align_niche(4, 4, Niche::new(0, 4, true, 1));
    let str_layout = Layout {
        layout_64: PlatformLayout::from_size_align_niche(16, 8, Niche::new(0, 8, true, 1)),
        layout_32: PlatformLayout::from_size_align_niche(8, 4, Niche::new(0, 4, true, 1)),
    };
    let usize_layout = Layout {
        layout_64: PlatformLayout::from_size_align(8, 8),
        layout_32: PlatformLayout::from_size_align(4, 4),
    };

    #[expect(clippy::match_same_arms)]
    match def.name() {
        "bool" => Layout::from_size_align_niche(1, 1, Niche::new(0, 1, false, 254)),
        "u8" => Layout::from_type::<u8>(),
        "u16" => Layout::from_type::<u16>(),
        "u32" => Layout::from_type::<u32>(),
        "u64" => Layout::from_type::<u64>(),
        "u128" => {
            panic!("Cannot calculate alignment for u128. It differs depending on Rust version.")
        }
        "usize" => usize_layout.clone(),
        "i8" => Layout::from_type::<i8>(),
        "i16" => Layout::from_type::<i16>(),
        "i32" => Layout::from_type::<i32>(),
        "i64" => Layout::from_type::<i64>(),
        "i128" => {
            panic!("Cannot calculate alignment for i128. It differs depending on Rust version.")
        }
        "isize" => usize_layout.clone(),
        "f32" => Layout::from_type::<f32>(),
        "f64" => Layout::from_type::<f64>(),
        "&str" => str_layout.clone(),
        "Atom" => str_layout,
        "ScopeId" => semantic_id_layout.clone(),
        "SymbolId" => semantic_id_layout.clone(),
        "ReferenceId" => semantic_id_layout,
        "PointerAlign" => Layout {
            layout_64: PlatformLayout::from_size_align(0, 8),
            layout_32: PlatformLayout::from_size_align(0, 4),
        },
        name => panic!("Unknown primitive type: {name}"),
    }
}

fn generate_layout_assertions(def: &TypeDef) -> (TokenStream, TokenStream) {
    match def {
        TypeDef::Struct(def) => generate_layout_assertions_for_struct(def),
        TypeDef::Enum(def) => generate_layout_assertions_for_enum(def),
        _ => (TokenStream::new(), TokenStream::new()),
    }
}

fn generate_layout_assertions_for_struct(def: &StructDef) -> (TokenStream, TokenStream) {
    fn generate_assertions(def: &StructDef, is_64: bool, struct_ident: &Ident) -> TokenStream {
        let layout = if is_64 { &def.layout.layout_64 } else { &def.layout.layout_32 };

        let size_align_assertions = generate_size_align_assertions(layout, struct_ident);

        let offset_asserts = def.fields.iter().filter_map(|field| {
            let field_ident = field.ident()?;
            if field.visibility != Visibility::Public {
                return None;
            }

            let offset =
                if is_64 { field.offset.offset_64 } else { field.offset.offset_32 } as usize;
            Some(quote! {
                assert!(offset_of!(#struct_ident, #field_ident) == #offset);
            })
        });

        quote! {
            #size_align_assertions
            #(#offset_asserts)*
        }
    }

    // TODO: Generate assertions for offsets
    let ident = def.ident();
    (generate_assertions(def, true, &ident), generate_assertions(def, false, &ident))
}

fn generate_layout_assertions_for_enum(def: &EnumDef) -> (TokenStream, TokenStream) {
    let ident = def.ident();
    (
        generate_size_align_assertions(&def.layout.layout_64, &ident),
        generate_size_align_assertions(&def.layout.layout_32, &ident),
    )
}

fn generate_size_align_assertions(layout: &PlatformLayout, ident: &Ident) -> TokenStream {
    let size = layout.size as usize;
    let align = layout.align as usize;
    quote! {
        ///@@line_break
        assert!(size_of::<#ident>() == #size);
        assert!(align_of::<#ident>() == #align);
    }
}
