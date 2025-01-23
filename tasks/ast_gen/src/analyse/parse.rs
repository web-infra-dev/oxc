use quote::ToTokens;
use rustc_hash::FxHashMap;
use syn::{
    punctuated::Punctuated, AttrStyle, Attribute, Expr, ExprLit, Field, Fields, GenericArgument,
    Generics, Ident, ItemEnum, ItemStruct, Lit, Meta, Path, PathArguments, PathSegment, Token,
    Type, TypePath, TypeReference, Variant, Visibility as SynVisibility,
};

use crate::{
    codegen::{AttrTarget, Codegen},
    schema::{
        BoxDef, CellDef, Def, EnumDef, FieldDef, File, FileId, OptionDef, PrimitiveDef, Schema,
        StructDef, TypeDef, TypeId, VariantDef, VecDef, Visibility,
    },
    DERIVES, GENERATORS,
};

use super::{
    ident_name,
    skeleton::{EnumSkeleton, Skeleton, StructSkeleton},
    Derives, FxIndexMap, FxIndexSet,
};

/// Parse `Skeleton`s into `TypeDef`s.
pub fn parse(
    skeletons: FxIndexMap<String, Skeleton>,
    files: Vec<File>,
    codegen: &Codegen,
) -> Schema {
    // Split `skeletons` into a `IndexSet<String>` (type names) and `Vec<Skeleton>` (skeletons)
    let mut skeletons_vec = Vec::with_capacity(skeletons.len());
    let type_names = skeletons
        .into_iter()
        .map(|(name, skeleton)| {
            skeletons_vec.push(skeleton);
            name
        })
        .collect();

    let parser = Parser::new(type_names, files, codegen);
    parser.parse_all(skeletons_vec)
}

/// Types parser.
struct Parser<'c> {
    type_names: FxIndexSet<String>,
    files: Vec<File>,
    codegen: &'c Codegen,
    extra_types: Vec<TypeDef>,
    options: FxHashMap<TypeId, TypeId>,
    boxes: FxHashMap<TypeId, TypeId>,
    vecs: FxHashMap<TypeId, TypeId>,
    cells: FxHashMap<TypeId, TypeId>,
}

impl<'c> Parser<'c> {
    /// Create `Parser`.
    fn new(type_names: FxIndexSet<String>, files: Vec<File>, codegen: &'c Codegen) -> Self {
        Self {
            type_names,
            files,
            codegen,
            extra_types: vec![],
            options: FxHashMap::default(),
            boxes: FxHashMap::default(),
            vecs: FxHashMap::default(),
            cells: FxHashMap::default(),
        }
    }

    /// Parse all `Skeleton`s into `TypeDef`s and return `Schema`.
    fn parse_all(mut self, skeletons: Vec<Skeleton>) -> Schema {
        let mut defs =
            skeletons.into_iter().map(|skeleton| self.parse_type(skeleton)).collect::<Vec<_>>();
        defs.extend(self.extra_types);
        Schema { defs, files: self.files }
    }

    /// Get `TypeId` for type name.
    fn type_id(&mut self, name: &str) -> TypeId {
        // Get type ID if already known
        if let Some(type_id) = self.type_names.get_index_of(name) {
            return type_id;
        }

        // Generate new type for known primitives/special cases
        let primitive = |name| TypeDef::Primitive(PrimitiveDef::new(name));

        let def = match name {
            "bool" => primitive("bool"),
            "u8" => primitive("u8"),
            "u16" => primitive("u16"),
            "u32" => primitive("u32"),
            "u64" => primitive("u64"),
            "u128" => primitive("u128"),
            "usize" => primitive("usize"),
            "i8" => primitive("i8"),
            "i16" => primitive("i16"),
            "i32" => primitive("i32"),
            "i64" => primitive("i64"),
            "i128" => primitive("i128"),
            "isize" => primitive("isize"),
            "f32" => primitive("f32"),
            "f64" => primitive("f64"),
            "&str" => primitive("&str"),
            "Atom" => primitive("Atom"),
            "ScopeId" => primitive("ScopeId"),
            "SymbolId" => primitive("SymbolId"),
            "ReferenceId" => primitive("ReferenceId"),
            "PointerAlign" => primitive("PointerAlign"),
            // Cannot be parsed normally as is defined inside `bitflags!` macro.
            // TODO: Find a way to encode this in the actual file.
            // e.g. `#[ast(alias_for(RegExpFlags))] struct RegExpFlagsAlias(u8);`
            "RegExpFlags" => TypeDef::Struct(StructDef::new(
                "RegExpFlags".to_string(),
                false,
                self.get_file_id("oxc_ast::ast::literal"),
                Derives::none(),
                vec![FieldDef::new(None, self.type_id("u8"), Visibility::Public)],
                false,
            )),
            _ => panic!("Unknown type: {name}"),
        };

        self.create_new_type(name.to_string(), def)
    }

    /// Create a new type definition.
    fn create_new_type(&mut self, name: String, def: TypeDef) -> TypeId {
        let type_id = self.type_names.len();
        let was_inserted = self.type_names.insert(name);
        assert!(was_inserted);
        self.extra_types.push(def);
        type_id
    }

    /// Get `FileId` for file with provided import path.
    fn get_file_id(&self, import_path: &str) -> FileId {
        let file_and_id =
            self.files.iter().enumerate().find(|(_, file)| file.import_path == import_path);
        match file_and_id {
            Some((file_id, _)) => file_id,
            None => panic!("Could not find file with import path: {import_path}"),
        }
    }

    /// Parse `Skeleton` to yield a `TypeDef`.
    fn parse_type(&mut self, skeleton: Skeleton) -> TypeDef {
        match skeleton {
            Skeleton::Struct(skeleton) => self.parse_struct(skeleton),
            Skeleton::Enum(skeleton) => self.parse_enum(skeleton),
        }
    }

    /// Parse `StructSkeleton` to yield a `TypeDef`.
    fn parse_struct(&mut self, skeleton: StructSkeleton) -> TypeDef {
        let StructSkeleton { name, item, file_id } = skeleton;
        let has_lifetime = check_generics(&item.generics, &name);
        let fields = self.parse_fields(&item.fields);
        let generated_derives = self.get_generated_derives(&item.attrs);
        let is_visited = check_ast_attr(&item.attrs);
        let mut def = TypeDef::Struct(StructDef::new(
            name,
            has_lifetime,
            file_id,
            generated_derives,
            fields,
            is_visited,
        ));

        // Parse attrs on type and fields
        self.parse_type_attrs(&mut def, &item.attrs);
        self.parse_field_attrs(&mut def, &item, generated_derives);

        def
    }

    fn parse_field_attrs(&self, def: &mut TypeDef, item: &ItemStruct, generated_derives: Derives) {
        let TypeDef::Struct(def) = def else {
            panic!("A derive or generator mutated `TypeDef::Struct` to another kind of `TypeDef`");
        };

        for (field_index, field) in item.fields.iter().enumerate() {
            for attr in &field.attrs {
                if !matches!(attr.style, AttrStyle::Outer) {
                    continue;
                }
                let Some(attr_ident) = attr.path().get_ident() else { continue };
                let attr_name = ident_name(attr_ident);

                if let Some(&target) = self.codegen.field_attrs.get(&*attr_name) {
                    let result = match target {
                        AttrTarget::Derive(derive_id) => {
                            // Check this struct has the relevant trait `#[generate_derive]`-ed on it
                            let derive = DERIVES[derive_id];
                            if !generated_derives.has(derive_id) {
                                panic_not_derived(def.name(), &attr_name, derive.trait_name());
                            }

                            derive.parse_field_attr(&attr_name, &attr.meta, def, field_index)
                        }
                        AttrTarget::Generator(generator_id) => {
                            let generator = GENERATORS[generator_id];
                            generator.parse_field_attr(&attr_name, &attr.meta, def, field_index)
                        }
                    };

                    assert!(
                        result.is_ok(),
                        "Invalid use of `#[{attr_name}]` on `{}::{}` struct field",
                        def.name(),
                        def.field(field_index).name_or_unnamed()
                    );
                }
            }
        }
    }

    /// Parse `EnumSkeleton` to yield a `TypeDef`.
    fn parse_enum(&mut self, skeleton: EnumSkeleton) -> TypeDef {
        let EnumSkeleton { name, item, inherits, file_id } = skeleton;
        let has_lifetime = check_generics(&item.generics, &name);
        let variants = item.variants.iter().map(|variant| self.parse_variant(variant)).collect();
        let inherits = inherits.into_iter().map(|name| self.type_id(&name)).collect();
        let generated_derives = self.get_generated_derives(&item.attrs);
        let is_visited = check_ast_attr(&item.attrs);
        let mut def = TypeDef::Enum(EnumDef::new(
            name,
            has_lifetime,
            file_id,
            generated_derives,
            variants,
            inherits,
            is_visited,
        ));

        // Parse attrs on type and variants
        self.parse_type_attrs(&mut def, &item.attrs);
        self.parse_variant_attrs(&mut def, &item, generated_derives);

        def
    }

    fn parse_variant_attrs(&self, def: &mut TypeDef, item: &ItemEnum, generated_derives: Derives) {
        let TypeDef::Enum(def) = def else {
            panic!("A derive or generator mutated `TypeDef::Enum` to another kind of `TypeDef`");
        };

        for (variant_index, variant) in item.variants.iter().enumerate() {
            for attr in &variant.attrs {
                if !matches!(attr.style, AttrStyle::Outer) {
                    continue;
                }
                let Some(attr_ident) = attr.path().get_ident() else { continue };
                let attr_name = ident_name(attr_ident);

                if let Some(&target) = self.codegen.field_attrs.get(&*attr_name) {
                    let result = match target {
                        AttrTarget::Derive(derive_id) => {
                            // Check this struct has the relevant trait `#[generate_derive]`-ed on it
                            let derive = DERIVES[derive_id];
                            if !generated_derives.has(derive_id) {
                                panic_not_derived(def.name(), &attr_name, derive.trait_name());
                            }

                            derive.parse_variant_attr(&attr_name, &attr.meta, def, variant_index)
                        }
                        AttrTarget::Generator(generator_id) => {
                            let generator = GENERATORS[generator_id];
                            generator.parse_variant_attr(&attr_name, &attr.meta, def, variant_index)
                        }
                    };

                    assert!(
                        result.is_ok(),
                        "Invalid use of `#[{attr_name}]` on `{}::{}` enum variant",
                        def.name(),
                        def.variant(variant_index).name(),
                    );
                }
            }
        }
    }

    /// Parse `Fields` to `Vec<FieldDef>`.
    fn parse_fields(&mut self, fields: &Fields) -> Vec<FieldDef> {
        fields.iter().map(|field| self.parse_field(field)).collect()
    }

    /// Parse `Field` to `FieldDef`.
    fn parse_field(&mut self, field: &Field) -> FieldDef {
        let name = field.ident.as_ref().map(ident_name);
        let ty = &field.ty;
        let type_id = self
            .parse_type_name(ty)
            .unwrap_or_else(|| panic!("Cannot parse type reference: {}", ty.to_token_stream()));
        let visibility = match &field.vis {
            SynVisibility::Public(_) => Visibility::Public,
            SynVisibility::Restricted(_) => Visibility::Restricted,
            SynVisibility::Inherited => Visibility::Private,
        };
        FieldDef::new(name, type_id, visibility)
    }

    /// Parse `Variant` to `VariantDef`.
    fn parse_variant(&mut self, variant: &Variant) -> VariantDef {
        let name = ident_name(&variant.ident);
        let fields = self.parse_fields(&variant.fields);

        let discriminant = {
            let Some((_, discriminant)) = variant.discriminant.as_ref() else {
                panic!("All enum variants must have explicit discriminants: {name}");
            };
            let Expr::Lit(ExprLit { lit: Lit::Int(lit), .. }) = discriminant else {
                panic!("Invalid enum discriminant {discriminant:?} on {name}");
            };
            let Ok(discriminant) = lit.base10_parse() else {
                panic!("Invalid base10 enum discriminant {discriminant:?} on {name}");
            };
            discriminant
        };

        VariantDef { name, fields, discriminant }
    }

    fn parse_type_name(&mut self, ty: &Type) -> Option<TypeId> {
        match ty {
            Type::Path(type_path) => self.parse_type_path(type_path),
            Type::Reference(type_ref) => self.parse_type_reference(type_ref),
            _ => None,
        }
    }

    fn parse_type_path(&mut self, type_path: &TypePath) -> Option<TypeId> {
        let segment = type_path_segment(type_path)?;
        let name = ident_name(&segment.ident);
        match &segment.arguments {
            PathArguments::None => Some(self.type_id(&name)),
            PathArguments::Parenthesized(_) => None,
            PathArguments::AngleBracketed(angled) => {
                // Get first arg, skipping over lifetime arg
                let mut args = angled.args.iter();
                let arg = match args.next().unwrap() {
                    GenericArgument::Lifetime(_) => args.next(),
                    arg => Some(arg),
                };

                if let Some(arg) = arg {
                    self.parse_complex_type_path(&name, arg)
                } else {
                    Some(self.type_id(&name))
                }
            }
        }
    }

    fn parse_complex_type_path(
        &mut self,
        wrapper_name: &str,
        arg: &GenericArgument,
    ) -> Option<TypeId> {
        let GenericArgument::Type(ty) = arg else { return None };

        let inner_type_id = self.parse_type_name(ty)?;

        let type_id = match wrapper_name {
            "Option" => self.options.get(&inner_type_id).copied().unwrap_or_else(|| {
                let name = format!("Option<{}>", &self.type_names[inner_type_id]);
                let def = TypeDef::Option(OptionDef::new(name.clone(), inner_type_id));
                let type_id = self.create_new_type(name, def);
                self.options.insert(inner_type_id, type_id);
                type_id
            }),
            "Box" => self.boxes.get(&inner_type_id).copied().unwrap_or_else(|| {
                let name = format!("Box<{}>", &self.type_names[inner_type_id]);
                let def = TypeDef::Box(BoxDef::new(name.clone(), inner_type_id));
                let type_id = self.create_new_type(name, def);
                self.boxes.insert(inner_type_id, type_id);
                type_id
            }),
            "Vec" => self.vecs.get(&inner_type_id).copied().unwrap_or_else(|| {
                let name = format!("Vec<{}>", &self.type_names[inner_type_id]);
                let def = TypeDef::Vec(VecDef::new(name.clone(), inner_type_id));
                let type_id = self.create_new_type(name, def);
                self.vecs.insert(inner_type_id, type_id);
                type_id
            }),
            "Cell" => self.cells.get(&inner_type_id).copied().unwrap_or_else(|| {
                let name = format!("Cell<{}>", &self.type_names[inner_type_id]);
                let def = TypeDef::Cell(CellDef::new(name.clone(), inner_type_id));
                let type_id = self.create_new_type(name, def);
                self.cells.insert(inner_type_id, type_id);
                type_id
            }),
            _ => return None,
        };
        Some(type_id)
    }

    fn parse_type_reference(&mut self, type_ref: &TypeReference) -> Option<TypeId> {
        if type_ref.mutability.is_some() {
            return None;
        }
        let Type::Path(type_path) = &*type_ref.elem else { return None };
        let segment = type_path_segment(type_path)?;
        if segment.ident != "str" || segment.arguments != PathArguments::None {
            return None;
        }
        Some(self.type_id("&str"))
    }

    fn parse_type_attrs(&mut self, def: &mut TypeDef, attrs: &[Attribute]) {
        for attr in attrs {
            if !matches!(attr.style, AttrStyle::Outer) {
                continue;
            }
            let Some(attr_ident) = attr.path().get_ident() else { continue };
            let attr_name = ident_name(attr_ident);

            if let Some(&target) = self.codegen.field_attrs.get(&*attr_name) {
                let result = match target {
                    AttrTarget::Derive(derive_id) => {
                        // Check this struct has the relevant trait `#[generate_derive]`-ed on it
                        let derive = DERIVES[derive_id];
                        if !def.generates_derive(derive_id) {
                            panic_not_derived(def.name(), &attr_name, derive.trait_name());
                        }

                        derive.parse_type_attr(&attr_name, &attr.meta, def)
                    }
                    AttrTarget::Generator(generator_id) => {
                        let generator = GENERATORS[generator_id];
                        generator.parse_type_attr(&attr_name, &attr.meta, def)
                    }
                };

                assert!(result.is_ok(), "Invalid use of `#[{attr_name}]` on `{}` type", def.name());
            }
        }
    }

    /// Get derives which are generated with `#[generate_derive(...)]` attrs.
    fn get_generated_derives(&self, attrs: &[Attribute]) -> Derives {
        let mut derives = Derives::none();
        for attr in attrs {
            if attr.path().is_ident("generate_derive") {
                let args: Punctuated<Ident, Token![,]> =
                    attr.parse_args_with(Punctuated::parse_terminated).unwrap();
                for arg in args {
                    let derive_id = self.codegen.get_derive_id_by_name(&ident_name(&arg));
                    derives.add(derive_id);
                }
            }
        }

        derives
    }
}

/// Check generics.
///
/// Return `true` if type has a lifetime.
///
/// # Panics
/// Panics if type has type params, const params, or more than one lifetime.
fn check_generics(generics: &Generics, name: &str) -> bool {
    assert!(
        generics.type_params().next().is_none(),
        "Types with generic type params are not supported: {name}"
    );
    assert!(
        generics.const_params().next().is_none(),
        "Types with generic const params are not supported: {name}"
    );

    match generics.lifetimes().count() {
        0 => false,
        1 => true,
        _ => panic!("Types with more than 1 lifetime are not supported: {name}"),
    }
}

/// Check `#[ast]` attr.
///
/// Return `true` for `#[ast(visit)]`, `false` for just `#[ast]`.
///
/// # Panics
/// Panics if does not match either of those patterns.
fn check_ast_attr(attrs: &[Attribute]) -> bool {
    let ast_attr = attrs.iter().find(|attr| attr.path().is_ident("ast")).unwrap();
    match &ast_attr.meta {
        Meta::Path(_) => return false,
        Meta::List(_) => {
            if let Ok(path) = ast_attr.parse_args::<Path>() {
                if path.is_ident("visit") {
                    return true;
                }
            }
        }
        Meta::NameValue(_) => {}
    }

    panic!("Invalid `#[ast] attr: {}", ast_attr.to_token_stream());
}

/// Get first segment from `TypePath`.
///
/// Returns `None` if has `qself` or leading colon, or if more than 1 segment.
fn type_path_segment(type_path: &TypePath) -> Option<&PathSegment> {
    if type_path.qself.is_some() || type_path.path.leading_colon.is_some() {
        return None;
    }

    let segments = &type_path.path.segments;
    if segments.len() != 1 {
        return None;
    }
    segments.first()
}

/// Panic with message that expected trait is not derived
fn panic_not_derived(type_name: &str, attr_name: &str, trait_name: &str) {
    panic!(
        "`{type_name}` type has `#[{attr_name}]` attribute, but `{trait_name}` trait \
        that handles `#[{attr_name}]` is not derived on `{type_name}`.\n\
        Expected `#[generate_derive({trait_name})]` to be present."
    );
}
