use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use snafu::{ResultExt, Snafu};
use syn::DeriveInput;

use crate::attributes::AttributeError;
use crate::r#enum::{Enum, Variant};
use crate::r#struct::{Field, Struct};

pub mod attributes;
pub mod r#enum;
mod prefix_mappings;
pub mod r#struct;

#[derive(Debug, Snafu)]
pub enum Error {
    // #[snafu(display("union types are not supported"))]
    // UnionType(Span),
    #[snafu(display("invalid `ld` attribute: {source}"))]
    InvalidAttribute { source: AttributeError },

    #[snafu(display("missing field"))]
    MissingField { span: Span },

    #[snafu(display("multiple fields"))]
    MultipleFields { span: Span },
    // #[snafu(display("missing field serialization method"))]
    // UnknownFieldSerializationMethod(Span),
    //
    // #[snafu(display("invalid IRI `{0}`"))]
    // InvalidIri(String, Span),
    //
    // #[snafu(display("missing variant IRI"))]
    // MissingVariantIri(Span),
}

pub enum LinkedDataType<F> {
    Enum(Enum<F>),
    Struct(Struct<F>),
}

pub trait TokenGenerator: Sized {
    fn generate_type_tokens(linked_data_type: &LinkedDataType<Self>, tokens: &mut TokenStream);

    fn generate_struct_tokens(r#struct: &Struct<Self>, tokens: &mut TokenStream);

    fn generate_enum_tokens(r#enum: &Enum<Self>, tokens: &mut TokenStream);

    fn generate_variant_tokens(variant: &Variant<Self>, tokens: &mut TokenStream);

    fn generate_field_tokens(field: &Field<Self>, tokens: &mut TokenStream);
}

impl<F: TokenGenerator> ToTokens for LinkedDataType<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        F::generate_type_tokens(self, tokens)
    }
}

impl<F: TokenGenerator> TryFrom<DeriveInput> for LinkedDataType<F> {
    type Error = Error;

    fn try_from(derive_input: DeriveInput) -> Result<Self, Self::Error> {
        // derive_input.attrs.get(0).unwrap().parse_args_with
        // let type_attributes: StructAttributes = derive_input
        //     .attrs
        //     .try_into()
        //     .context(InvalidAttributeSnafu)?;
        match derive_input.data {
            syn::Data::Struct(data) => {
                r#struct::extract::<F>(derive_input.attrs, derive_input.ident, data)
                    .map(LinkedDataType::Struct)
                    .context(InvalidAttributeSnafu)
            }
            syn::Data::Enum(data) => {
                r#enum::extract::<F>(derive_input.attrs, derive_input.ident, data)
                    .map(LinkedDataType::Enum)
                    .context(InvalidAttributeSnafu)
            }
            syn::Data::Union(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {}
