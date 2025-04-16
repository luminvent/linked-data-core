use snafu::ResultExt;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Ident};

use crate::attributes::AttributeError;
use crate::attributes::r#type::EnumAttributes;
use crate::attributes::variant::VariantAttributes;
use crate::prefix_mappings::PrefixMappings;
use crate::{Error, InvalidAttributeSnafu};

pub struct Enum {
    pub ident: Ident,
    pub attributes: EnumAttributes,
    pub variants: Vec<Variant>,
}

pub struct Variant {
    pub attributes: VariantAttributes,
    pub r#type: syn::Type,
}

/// TODO Get the errors out!!!
impl<'ast> Visit<'ast> for Enum {
    fn visit_variant(&mut self, i: &'ast syn::Variant) {
        let variant =
            Variant::from_variant(i.clone(), &self.attributes.prefix_mappings)
                .unwrap();
        self.variants.push(variant);
    }
}

impl Variant {
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
            r#type: field.ty.clone(),
        })
    }
}

pub fn generate(
    attrs: Vec<Attribute>,
    ident: Ident,
    data: syn::DataEnum,
) -> Result<Enum, AttributeError> {
    let mut visitor = Enum {
        ident,
        attributes: attrs.try_into()?,
        variants: vec![],
    };
    visitor.visit_data_enum(&data);

    Ok(visitor)
}
