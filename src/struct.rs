use std::marker::PhantomData;

use iref::IriBuf;
use proc_macro2::TokenStream;
use quote::ToTokens;
use snafu::ResultExt;
use syn::visit::Visit;
use syn::{Attribute, Ident};

use crate::attributes::AttributeError;
use crate::attributes::field::FieldAttributes;
use crate::attributes::r#type::StructAttributes;
use crate::prefix_mappings::PrefixMappings;
use crate::{Error, InvalidAttributeSnafu, TokenGenerator};

pub struct Struct<G> {
    attributes: StructAttributes,
    pub ident: Ident,
    pub fields: Vec<Field<G>>,
}

pub struct Field<G> {
    attributes: FieldAttributes,
    pub ty: syn::Type,
    _generator: PhantomData<G>,
}

impl<G> Struct<G> {
    pub fn type_iri(&self) -> Option<&IriBuf> {
        self.attributes.r#type.as_ref()
    }
}

impl<F: TokenGenerator> ToTokens for Struct<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        F::generate_struct_tokens(self, tokens)
    }
}

/// TODO Get the errors out!!!
impl<'ast, F> Visit<'ast> for Struct<F> {
    fn visit_field(&mut self, field: &'ast syn::Field) {
        let field_obj =
            Field::from_field(field.clone(), &self.attributes.prefix_mappings)
                .unwrap();
        self.fields.push(field_obj);
    }
}

impl<F> Field<F> {
    fn from_field(
        field: syn::Field,
        prefix_mappings: &PrefixMappings,
    ) -> Result<Self, Error> {
        let attributes =
            FieldAttributes::try_from_attrs(field.attrs, prefix_mappings)
                .context(InvalidAttributeSnafu)?;

        Ok(Field {
            attributes,
            ty: field.ty,
            _generator: PhantomData,
        })
    }

    pub fn is_flattened(&self) -> bool {
        self.attributes.flatten
    }

    pub fn is_graph(&self) -> bool {
        self.attributes.is_graph
    }

    pub fn is_ignored(&self) -> bool {
        self.attributes.ignore
    }

    pub fn predicate(&self) -> Option<&IriBuf> {
        self.attributes.predicate.as_ref()
    }

    pub fn is_id(&self) -> bool {
        self.attributes.is_id
    }
}

impl<F: TokenGenerator> ToTokens for Field<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        F::generate_field_tokens(self, tokens)
    }
}

pub fn extract<G>(
    attrs: Vec<Attribute>,
    ident: Ident,
    data: syn::DataStruct,
) -> Result<Struct<G>, AttributeError> {
    let mut visitor = Struct {
        ident,
        attributes: attrs.try_into()?,
        fields: vec![],
    };
    visitor.visit_data_struct(&data);

    Ok(visitor)
}
