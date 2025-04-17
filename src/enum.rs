use std::marker::PhantomData;

use proc_macro2::TokenStream;
use quote::ToTokens;
use snafu::ResultExt;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Ident};

use crate::attributes::AttributeError;
use crate::attributes::r#type::EnumAttributes;
use crate::attributes::variant::{PredicatePath, VariantAttributes};
use crate::prefix_mappings::PrefixMappings;
use crate::{Error, InvalidAttributeSnafu, TokenGenerator};

pub struct Enum<G> {
    ident: Ident,
    attributes: EnumAttributes,
    variants: Vec<Variant<G>>,
}

pub struct Variant<G> {
    attributes: VariantAttributes,
    ty: syn::Type,
    _generator: PhantomData<G>,
}

impl<G> Enum<G> {
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn variants(&self) -> &[Variant<G>] {
        &self.variants
    }
}

impl<F: TokenGenerator> ToTokens for Enum<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        F::generate_enum_tokens(self, tokens)
    }
}

/// TODO Get the errors out!!!
impl<'ast, F> Visit<'ast> for Enum<F> {
    fn visit_variant(&mut self, i: &'ast syn::Variant) {
        let variant = Variant::from_variant(i.clone(), &self.attributes.prefix_mappings).unwrap();
        self.variants.push(variant);
    }
}

impl<F> Variant<F> {
    fn from_variant(
        variant: syn::Variant,
        prefix_mappings: &PrefixMappings,
    ) -> Result<Self, Error> {
        let mut fields = variant.fields.iter();

        let Some(field) = fields.next() else {
            return Err(Error::MissingField {
                span: variant.fields.span(),
            });
        };

        if let Some(field) = fields.next() {
            return Err(Error::MissingField { span: field.span() });
        }

        Ok(Variant {
            attributes: VariantAttributes::try_from_attrs(
                field.attrs.clone(),
                variant.attrs,
                prefix_mappings,
            )
            .context(InvalidAttributeSnafu)?,
            ty: field.ty.clone(),
            _generator: PhantomData,
        })
    }

    pub fn predicate_path(&self) -> &PredicatePath {
        &self.attributes.predicate_path
    }

    pub fn ty(&self) -> &syn::Type {
        &self.ty
    }
}

impl<F: TokenGenerator> ToTokens for Variant<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        F::generate_variant_tokens(self, tokens)
    }
}

pub fn extract<G>(
    attrs: Vec<Attribute>,
    ident: Ident,
    data: syn::DataEnum,
) -> Result<Enum<G>, AttributeError> {
    let mut visitor = Enum {
        ident,
        attributes: attrs.try_into()?,
        variants: vec![],
    };
    visitor.visit_data_enum(&data);

    Ok(visitor)
}
