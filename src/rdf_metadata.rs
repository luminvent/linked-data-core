use std::marker::PhantomData;

use oxiri::Iri;
use proc_macro_error::abort;
use syn::DeriveInput;
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::attributes::field::RdfFieldAttributes;
use crate::attributes::r#type::{RdfEnumAttributes, RdfStructAttributes};
use crate::attributes::variant::{PredicatePath, RdfVariantAttributes};
use crate::prefix_mappings::PrefixMappings;
use crate::{Error, TokenGenerator};

pub enum RdfType<F> {
  Enum(RdfEnum<F>),
  Struct(RdfStruct<F>),
}

pub struct RdfEnum<G> {
  attributes: RdfEnumAttributes,
  pub ident: syn::Ident,
  pub variants: Vec<RdfVariant<G>>,
}

pub struct RdfVariant<G> {
  attributes: RdfVariantAttributes,
  pub index: usize,
  /// The variant's single field type, or `None` for a unit variant.
  ///
  /// A unit variant marks the enum as a closed list of RDF resources (see
  /// [`RdfEnum::is_closed_list`]): the variant itself, rather than a wrapped value, is the leaf
  /// RDF term, identified by its own IRI.
  pub ty: Option<syn::Type>,
  _generator: PhantomData<G>,
}

pub struct RdfStruct<G> {
  attributes: RdfStructAttributes,
  pub ident: syn::Ident,
  pub fields: Vec<RdfField<G>>,
}

#[derive(Debug)]
pub struct RdfField<G> {
  attributes: RdfFieldAttributes,
  pub ty: syn::Type,
  _generator: PhantomData<G>,
}

pub fn unwrap_or_abort<T>(result: Result<T, Error>) -> T {
  match result {
    Ok(value) => value,
    Err(error) => abort!(error.span(), error),
  }
}

impl<G: TokenGenerator> RdfType<G> {
  pub fn from_derive(derive_input: DeriveInput) -> Self {
    unwrap_or_abort(Self::try_from_derive(derive_input))
  }

  fn try_from_derive(derive_input: DeriveInput) -> Result<Self, Error> {
    match derive_input.data {
      syn::Data::Struct(data) => {
        let mut r#struct = RdfStruct {
          ident: derive_input.ident,
          attributes: derive_input.attrs.try_into()?,
          fields: vec![],
        };
        r#struct.visit_data_struct(&data);
        Ok(RdfType::Struct(r#struct))
      }
      syn::Data::Enum(data) => {
        let mut r#enum = RdfEnum {
          ident: derive_input.ident,
          attributes: derive_input.attrs.try_into()?,
          variants: vec![],
        };
        r#enum.visit_data_enum(&data);

        let has_unit_variant = r#enum.variants.iter().any(|variant| variant.ty.is_none());
        let has_data_variant = r#enum.variants.iter().any(|variant| variant.ty.is_some());
        if has_unit_variant && has_data_variant {
          return Err(Error::MixedVariantKinds {
            span: data.enum_token.span(),
          });
        }

        Ok(RdfType::Enum(r#enum))
      }
      syn::Data::Union(data_union) => Err(Error::UnionType {
        span: data_union.union_token.span(),
      }),
    }
  }
}

impl<G> RdfEnum<G> {
  /// A closed list of RDF resources: every variant is a unit variant, and each one is a leaf
  /// term identified by its own IRI (e.g. a controlled vocabulary), rather than a tagged union
  /// wrapping heterogeneous inner values. `try_from_derive` rejects mixing unit and non-unit
  /// variants, so checking the first variant reflects every variant.
  pub fn is_closed_list(&self) -> bool {
    self.variants.first().is_none_or(RdfVariant::is_unit)
  }

  pub fn prefix_mappings(&self) -> &PrefixMappings {
    &self.attributes.prefix_mappings
  }
}

impl<'ast, F> Visit<'ast> for RdfEnum<F> {
  fn visit_variant(&mut self, variant: &'ast syn::Variant) {
    let variant = unwrap_or_abort(RdfVariant::from_variant(
      variant.clone(),
      self.variants.len(),
      &self.attributes.prefix_mappings,
    ));
    self.variants.push(variant);
  }
}

impl<F> RdfVariant<F> {
  fn from_variant(
    variant: syn::Variant,
    index: usize,
    prefix_mappings: &PrefixMappings,
  ) -> Result<Self, Error> {
    let mut fields = variant.fields.iter();
    let field = fields.next();

    if let Some(extra_field) = fields.next() {
      return Err(Error::StructVariant {
        span: extra_field.span(),
      });
    }

    let (inner_attrs, ty) = match field {
      Some(field) => (field.attrs.clone(), Some(field.ty.clone())),
      None => (vec![], None),
    };

    Ok(RdfVariant {
      attributes: RdfVariantAttributes::try_from_attrs(
        &variant,
        inner_attrs,
        variant.attrs.clone(),
        prefix_mappings,
      )?,
      index,
      ty,
      _generator: PhantomData,
    })
  }

  pub fn predicate_path(&self) -> &PredicatePath {
    &self.attributes.predicate_path
  }

  /// `true` for a unit variant: a leaf member of a closed list of RDF resources, identified by
  /// its own IRI rather than wrapping a value. See [`RdfEnum::is_closed_list`].
  pub fn is_unit(&self) -> bool {
    self.ty.is_none()
  }
}

impl<G> RdfStruct<G> {
  pub fn type_iri(&self) -> Option<&Iri<String>> {
    self.attributes.r#type.as_ref()
  }

  pub fn prefix_mappings(&self) -> &PrefixMappings {
    &self.attributes.prefix_mappings
  }
}

impl<'ast, F> Visit<'ast> for RdfStruct<F> {
  fn visit_field(&mut self, field: &'ast syn::Field) {
    let rdf_field = unwrap_or_abort(RdfField::try_from_field(
      field.clone(),
      &self.attributes.prefix_mappings,
    ));
    self.fields.push(rdf_field);
  }
}

impl<F> RdfField<F> {
  fn try_from_field(field: syn::Field, prefix_mappings: &PrefixMappings) -> Result<Self, Error> {
    let attributes = RdfFieldAttributes::try_from_attrs(field.attrs, prefix_mappings)?;

    Ok(RdfField {
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

  pub fn predicate(&self) -> Option<&Iri<String>> {
    self.attributes.predicate.as_ref()
  }

  pub fn is_id(&self) -> bool {
    self.attributes.is_id
  }
}
