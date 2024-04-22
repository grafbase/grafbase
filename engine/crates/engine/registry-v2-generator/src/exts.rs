use cynic_parser::type_system::{FieldDefinition, ObjectDefinition, ScalarDefinition, TypeDefinition, UnionDefinition};
use quote::quote;

pub trait DistinctExt {
    fn is_distinct(&self) -> bool;
}

impl DistinctExt for ObjectDefinition<'_> {
    fn is_distinct(&self) -> bool {
        // Indicates that a type is "distinct" i.e. no other instances of this type can
        // have the same values.  We can use this to generate an optimised PartialEq
        self.directives().any(|directive| directive.name() == "distinct")
    }
}

impl DistinctExt for UnionDefinition<'_> {
    fn is_distinct(&self) -> bool {
        // Indicates that a type is "distinct" i.e. no other instances of this type can
        // have the same values.  We can use this to generate an optimised PartialEq
        self.directives().any(|directive| directive.name() == "distinct")
    }
}

pub trait ScalarExt {
    fn is_inline(&self) -> bool;
    fn should_box(&self) -> bool;
    fn reader_returns_ref(&self) -> bool;
    fn reader_fn_override(&self) -> Option<proc_macro2::TokenStream>;
}

impl ScalarExt for ScalarDefinition<'_> {
    fn should_box(&self) -> bool {
        self.directives().any(|directive| directive.name() == "box")
    }

    fn is_inline(&self) -> bool {
        self.directives().any(|directive| directive.name() == "inline")
    }

    fn reader_returns_ref(&self) -> bool {
        self.directives().any(|directive| directive.name() == "ref")
    }

    fn reader_fn_override(&self) -> Option<proc_macro2::TokenStream> {
        if self.name() == "String" {
            return Some(quote! { &'a str });
        }
        None
    }
}

pub trait FileDirectiveExt<'a> {
    fn file_name(&self) -> &'a str;
}

impl<'a> FileDirectiveExt<'a> for TypeDefinition<'a> {
    fn file_name(&self) -> &'a str {
        self.directives()
            .find(|directive| directive.name() == "file")
            .and_then(|directive| directive.arguments().next()?.value().as_str())
            .unwrap_or(self.name())
    }
}

pub trait UnionExt<'a> {
    fn variant_name_override(&self, index: usize) -> Option<&'a str>;
}

impl<'a> UnionExt<'a> for UnionDefinition<'a> {
    fn variant_name_override(&self, index: usize) -> Option<&'a str> {
        self.directives()
            .find(|directive| directive.name() == "variant")?
            .arguments()
            .next()?
            .value()
            .as_list_iter()?
            .nth(index)?
            .as_str()
    }
}

pub trait FieldExt {
    fn is_inline(&self) -> bool;
    fn should_have_reader_fn(&self) -> bool;
    fn should_default(&self) -> bool;
}

impl FieldExt for FieldDefinition<'_> {
    fn is_inline(&self) -> bool {
        self.directives().any(|directive| directive.name() == "inline")
    }
    fn should_have_reader_fn(&self) -> bool {
        !self.directives().any(|directive| directive.name() == "noreader")
    }
    fn should_default(&self) -> bool {
        self.directives().any(|directive| directive.name() == "default")
    }
}
