use std::ops::Add;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use super::assignment::AssignmentLikeNode;
use crate::{
    array, doc::{Doc, DocBuilder, Group, IfBreak, Line}, format::{assignment, Separator}, group, hardline, if_break, indent, indent_if_break, line, softline, space, ss, Format, Prettier, 
};

pub(super) fn print_class<'a>(p: &mut Prettier<'a>, class: &Class<'a>) -> Doc<'a> {
    let mut parts = p.vec();
    let mut heritage_clauses_parts = p.vec();
    let mut group_parts = p.vec();

    // Keep old behaviour of extends in same line
    // If there is only on extends and there are not comments
    // ToDo: implement comment checks 
    // @link https://github.com/prettier/prettier/blob/aa3853b7765645b3f3d8a76e41cf6d70b93c01fd/src/language-js/print/class.js#L62
    let group_mode = class.implements.as_ref().map(|v| !v.is_empty()).unwrap_or(false);

    if let Some(params) = &class.type_parameters {
        group_parts.push(params.format(p));
        group_parts.push(space!());
    }

    if let Some(super_class) = &class.super_class {
        let mut extend_parts = p.vec();

        extend_parts.push(ss!("extends "));
        extend_parts.push(super_class.format(p));

        if let Some(super_type_parameters) = &class.super_type_parameters {
            extend_parts.push(super_type_parameters.format(p));
        }
        
        if group_mode {
            heritage_clauses_parts.push(softline!());
        }

        heritage_clauses_parts.push(Doc::Array(extend_parts));
    }

    heritage_clauses_parts.push(print_heritage_clauses_implements(p, class));

    println!("{group_mode:?} {}", should_indent_only_heritage_clauses(class));

    for decorator in &class.decorators {
        parts.push(ss!("@"));
        parts.push(decorator.expression.format(p));
        parts.extend(hardline!());
    }

    if class.declare {
        parts.push(ss!("declare "));
    }

    if class.r#abstract {
        parts.push(ss!("abstract "));
    }

    parts.push(ss!("class "));
    
    if let Some(id) = &class.id {
        parts.push(id.format(p));
        parts.push(space!());
    }
    
    if group_mode {
        let printend_parts_group = if should_indent_only_heritage_clauses(class) {
            array!(p, Doc::Array(group_parts), indent!(p, Doc::Array(heritage_clauses_parts)))
        } else {
            indent!(p, Doc::Array(group_parts), Doc::Array(heritage_clauses_parts))
        };

        parts.push(printend_parts_group);
    } else {
        parts.push(array!(p, Doc::Array(heritage_clauses_parts), Doc::Array(group_parts)))
    }

    parts.push(class.body.format(p));
    Doc::Array(parts)
}

pub(super) fn print_class_body<'a>(p: &mut Prettier<'a>, class_body: &ClassBody<'a>) -> Doc<'a> {
    let mut parts_inner = p.vec();

    for (i, node) in class_body.body.iter().enumerate() {
        parts_inner.push(node.format(p));

        if !p.options.semi
            && node.is_property()
            && should_print_semicolon_after_class_property(node, class_body.body.get(i + 1))
        {
            parts_inner.push(ss!(";"));
        }

        if i < class_body.body.len() - 1 {
            parts_inner.extend(hardline!());

            if p.is_next_line_empty(node.span()) {
                parts_inner.extend(hardline!());
            }
        }
    }

    // TODO: if there are any dangling comments, print them

    let mut parts = p.vec();
    // TODO is class_body.len() != 0, print hardline after heritage

    parts.push(ss!("{"));
    if !parts_inner.is_empty() {
        let indent = {
            let mut parts = p.vec();
            parts.extend(hardline!());
            parts.push(Doc::Array(parts_inner));
            Doc::Indent(parts)
        };
        parts.push(array![p, indent]);
        parts.extend(hardline!());
    }

    parts.push(ss!("}"));

    Doc::Array(parts)
}

#[derive(Debug)]
pub enum ClassMemberish<'a, 'b> {
    PropertyDefinition(&'b PropertyDefinition<'a>),
    AccessorProperty(&'b AccessorProperty<'a>),
}

impl<'a, 'b> ClassMemberish<'a, 'b> {
    fn format_key(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                property_definition.key.format(p)
            }
            ClassMemberish::AccessorProperty(accessor_property) => accessor_property.key.format(p),
        }
    }

    fn decorators(&self) -> Option<&oxc_allocator::Vec<Decorator<'a>>> {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                Some(&property_definition.decorators)
            }

            ClassMemberish::AccessorProperty(accessor_property) => None,
        }
    }

    fn is_static(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => property_definition.r#static,
            ClassMemberish::AccessorProperty(accessor_property) => accessor_property.r#static,
        }
    }

    fn is_override(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                property_definition.r#override
            }
            ClassMemberish::AccessorProperty(accessor_property) => false,
        }
    }

    fn is_readonly(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => property_definition.readonly,
            ClassMemberish::AccessorProperty(_) => true,
        }
    }

    fn is_declare(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => property_definition.declare,
            ClassMemberish::AccessorProperty(_) => false,
        }
    }

    fn is_abstract(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                property_definition.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition
            }
            ClassMemberish::AccessorProperty(accessor_property) => {
                accessor_property.r#type == AccessorPropertyType::TSAbstractAccessorProperty
            }
        }
    }

    fn is_optional(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => property_definition.optional,
            ClassMemberish::AccessorProperty(_) => false,
        }
    }

    fn is_definite(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => property_definition.definite,
            ClassMemberish::AccessorProperty(accessor_property) => accessor_property.definite,
        }
    }

    fn right_expr(&self) -> Option<&Expression<'a>> {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                property_definition.value.as_ref()
            }
            ClassMemberish::AccessorProperty(_) => None,
        }
    }

    fn format_accessibility(&self, p: &mut Prettier<'a>) -> Option<Doc<'a>> {
        match self {
            ClassMemberish::AccessorProperty(def) => def.accessibility.map(|v| ss!(v.as_str())),
            ClassMemberish::PropertyDefinition(def) => def.accessibility.map(|v| ss!(v.as_str())),
        }
    }

    fn format_type_annotation(&self, p: &mut Prettier<'a>) -> Option<Doc<'a>> {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                property_definition.type_annotation.as_ref().map(|v| v.type_annotation.format(p))
            }
            ClassMemberish::AccessorProperty(accessor_definition) => {
                accessor_definition.type_annotation.as_ref().map(|v| v.type_annotation.format(p))
            }
        }
    }
}

pub(super) fn print_class_property<'a>(
    p: &mut Prettier<'a>,
    node: &ClassMemberish<'a, '_>,
) -> Doc<'a> {
    let mut parts = p.vec();

    if let Some(decarators) = node.decorators() {
        for decorator in decarators {
            parts.push(ss!("@"));
            parts.push(decorator.expression.format(p));
            parts.extend(hardline!());
        }
    }

    if let Some(accessibility) = node.format_accessibility(p) {
        parts.push(accessibility);
        parts.push(space!());
    }

    if node.is_declare() {
        parts.push(ss!("declare "));
    }

    if node.is_static() {
        parts.push(ss!("static "));
    }

    if node.is_abstract() {
        parts.push(ss!("abstract "));
    }

    if node.is_override() {
        parts.push(ss!("override "));
    }

    if node.is_readonly() {
        parts.push(ss!("readonly "));
    }

    parts.push(node.format_key(p));

    if node.is_optional() {
        parts.push(ss!("?"));
    } else if node.is_definite() {
        parts.push(ss!("!"));
    }

    if let Some(type_annotation) = node.format_type_annotation(p) {
        parts.push(ss!(": "));
        parts.push(type_annotation);
    }

    let right_expr = node.right_expr();
    let node = match node {
        ClassMemberish::PropertyDefinition(v) => AssignmentLikeNode::PropertyDefinition(v),
        ClassMemberish::AccessorProperty(v) => AssignmentLikeNode::AccessorProperty(v),
    };
    let mut result =
        assignment::print_assignment(p, node, Doc::Array(parts), Doc::Str(" ="), right_expr);

    if p.options.semi {
        let mut parts = p.vec();
        parts.push(result);
        parts.push(ss!(";"));
        result = Doc::Array(parts);
    }
    result
}

fn should_print_semicolon_after_class_property<'a>(
    node: &ClassElement<'a>,
    next_node: Option<&ClassElement<'a>>,
) -> bool {
    if !node.computed() {
        if let ClassElement::PropertyDefinition(property_definition) = node {
            if property_definition.value.is_none() && property_definition.type_annotation.is_none()
            {
                if let Some(key) = property_definition.key.static_name() {
                    if key == "static" || key == "get" || key == "set" {
                        return true;
                    }
                }
            }
        }
    }

    let Some(next_node) = next_node else {
        return false;
    };

    if next_node.r#static() || next_node.accessibility().is_some() {
        return false;
    }

    if !next_node.computed() {
        if let Some(prop_key) = next_node.property_key() {
            if let Some(prop_key) = prop_key.static_name() {
                if prop_key == "in" || prop_key == "instanceof" {
                    return true;
                }
            }
        }
    }

    match next_node {
        ClassElement::PropertyDefinition(property_definition) => property_definition.computed,
        ClassElement::StaticBlock(_) => false,
        ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => true,
        ClassElement::MethodDefinition(method_definition) => {
            let is_async = method_definition.value.r#async;

            if is_async
                || method_definition.kind == MethodDefinitionKind::Get
                || method_definition.kind == MethodDefinitionKind::Set
            {
                return false;
            }

            let is_generator = method_definition.value.generator;

            if method_definition.computed || is_generator {
                return true;
            }

            false
        }
    }
}

/**
 * @link https://github.com/prettier/prettier/blob/aa3853b7765645b3f3d8a76e41cf6d70b93c01fd/src/language-js/print/class.js#L148
 */
fn print_heritage_clauses_implements<'a>(p: &mut Prettier<'a>, class: &Class<'a>) -> Doc<'a> {
    let mut parts = p.vec();

    if class.implements.is_none() {
        return Doc::Array(parts);
    }

    let implements = class.implements.as_ref().unwrap();

    if implements.len() == 0 {
        return Doc::Array(parts);
    }

    if should_indent_only_heritage_clauses(class) {
        parts.push(Doc::IfBreak(IfBreak{
            break_contents: p.boxed(line!()),
            flat_content: p.boxed(ss!("")),
            group_id: None  // ToDo - how to attach group id
        }));
    } else {
        parts.extend(hardline!());
    }

    parts.push(ss!("implements "));
    
    let implements_docs = implements.iter().map(|v| v.format(p)).collect();

    parts.push(indent!(p, group!(p, Doc::Array(p.join(Separator::CommaLine, implements_docs)))));
    parts.push(space!());

    Doc::Group(Group::new(parts))
}

fn should_indent_only_heritage_clauses(class: &Class) -> bool {
    // Todo - Check for Comments
    // @link https://github.com/prettier/prettier/blob/aa3853b7765645b3f3d8a76e41cf6d70b93c01fd/src/language-js/print/class.js#L137
    class.type_parameters.is_some() && !has_multiple_heritage(class)
}

fn has_multiple_heritage(class: &Class) -> bool {
    let mut len = if class.super_class.is_some() { 1 } else { 0 };

    if let Some(implements) = &class.implements {
        len.add(i32::try_from(implements.len()).unwrap());
    }

    len > 1
}
