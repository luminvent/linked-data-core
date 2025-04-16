use std::str::FromStr;

use iref::IriBuf;
use snafu::ResultExt;

use crate::attributes::ast::{EnumAttribute, PrefixAttribute, StructAttribute};
use crate::attributes::{parse_iri, parse_ld_attributes};
use crate::prefix_mappings::{Prefix, PrefixMappings};
use crate::{Error, InvalidMappingSnafu};

#[derive(Debug)]
pub struct RdfStructAttributes {
  pub prefix_mappings: PrefixMappings,
  pub r#type: Option<IriBuf>,
}

#[derive(Debug)]
pub struct RdfEnumAttributes {
  pub prefix_mappings: PrefixMappings,
}

impl TryFrom<Vec<syn::Attribute>> for RdfStructAttributes {
  type Error = Error;

  fn try_from(attrs: Vec<syn::Attribute>) -> Result<Self, Self::Error> {
    let mut type_attrs = Vec::new();
    let prefix_mappings = parse_ld_attributes(&attrs)?
      .into_iter()
      .filter_map(|attr| match attr {
        StructAttribute::Prefix(prefix) => Some(PrefixMappings::try_from(prefix)),
        StructAttribute::Type(type_attr) => {
          type_attrs.push(type_attr);
          None
        }
      })
      .collect::<Result<PrefixMappings, Error>>()?;

    if let Some(type_attr) = type_attrs.get(1) {
      return Err(Error::MultipleTypes {
        span: type_attr.identifier.span(),
      });
    }

    let mut types = type_attrs
      .into_iter()
      .map(|type_attr| {
        prefix_mappings
          .expand(type_attr.identifier.value())
          .context(InvalidMappingSnafu {
            span: type_attr.identifier.span(),
          })
      })
      .collect::<Result<Vec<_>, _>>()?;

    Ok(RdfStructAttributes {
      prefix_mappings,
      r#type: types.pop(),
    })
  }
}

impl TryFrom<Vec<syn::Attribute>> for RdfEnumAttributes {
  type Error = Error;

  fn try_from(attrs: Vec<syn::Attribute>) -> Result<Self, Self::Error> {
    let prefix_mappings = parse_ld_attributes(&attrs)?
      .into_iter()
      .map(|attr| match attr {
        EnumAttribute::Prefix(prefix_attr) => PrefixMappings::try_from(prefix_attr),
      })
      .collect::<Result<PrefixMappings, Error>>()?;

    Ok(RdfEnumAttributes { prefix_mappings })
  }
}

impl TryFrom<PrefixAttribute> for PrefixMappings {
  type Error = Error;

  fn try_from(attr: PrefixAttribute) -> Result<Self, Self::Error> {
    let lit_iri = attr.mapping.iri;
    let iri = parse_iri(lit_iri)?;
    let lit_prefix = attr.mapping.prefix;
    let prefix = Prefix::from_str(&lit_prefix.value()).context(InvalidMappingSnafu {
      span: lit_prefix.span(),
    })?;

    let mut prefix_mappings = PrefixMappings::default();
    prefix_mappings.insert_prefix_mapping(prefix, iri);
    Ok(prefix_mappings)
  }
}

impl TryFrom<Vec<PrefixAttribute>> for PrefixMappings {
  type Error = Error;

  fn try_from(attrs: Vec<PrefixAttribute>) -> Result<Self, Self::Error> {
    attrs.into_iter().map(PrefixMappings::try_from).collect()
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use syn::{Attribute, parse_quote};

  use super::*;
  use crate::attributes::ast::PrefixAttribute;
  use crate::prefix_mappings::Prefix;

  const TEST_IRI: &str = "http://example.org/test";
  const TEST_PREFIX: &str = "ex";
  const TEST_PREFIX_IRI: &str = "http://example.org/";

  #[test]
  fn test_prefix_mappings_from_vec_prefix_attribute() {
    let prefix_attr1: PrefixAttribute = parse_quote! { prefix(#TEST_PREFIX = #TEST_PREFIX_IRI) };
    let prefix_attr2: PrefixAttribute = parse_quote! { prefix("foo" = "http://foo.org/") };

    let prefix_mappings = PrefixMappings::try_from(vec![prefix_attr1, prefix_attr2]).unwrap();

    let prefix1 = Prefix::from_str(TEST_PREFIX).unwrap();
    let prefix2 = Prefix::from_str("foo").unwrap();
    assert!(prefix_mappings.get(prefix1).is_some());
    assert_eq!(
      prefix_mappings.get(prefix2).unwrap().as_str(),
      "http://foo.org/"
    );
  }

  #[test]
  fn test_struct_attributes_from_attributes() {
    let attrs: Vec<Attribute> = parse_quote! {
        #[ld(prefix(#TEST_PREFIX = #TEST_PREFIX_IRI))]
        #[ld(type = #TEST_IRI)]
    };

    let struct_attrs = RdfStructAttributes::try_from(attrs).unwrap();

    assert!(struct_attrs.r#type.is_some());
    assert_eq!(struct_attrs.r#type.unwrap().as_str(), TEST_IRI);
  }

  #[test]
  fn test_struct_attributes_with_prefixed_type() {
    let attrs: Vec<Attribute> = parse_quote! {
        #[ld(prefix(#TEST_PREFIX = #TEST_PREFIX_IRI))]
        #[ld(type = "ex:resource")]
    };

    let struct_attrs = RdfStructAttributes::try_from(attrs).unwrap();

    assert!(struct_attrs.r#type.is_some());
    assert_eq!(
      struct_attrs.r#type.unwrap().as_str(),
      &format!("{}resource", TEST_PREFIX_IRI)
    );
  }

  #[test]
  fn test_struct_attributes_multiple_types_error() {
    let attrs: Vec<Attribute> = parse_quote! {
        #[ld(type = #TEST_IRI)]
        #[ld(type = "http://another.org/type")]
    };

    let result = RdfStructAttributes::try_from(attrs);
    assert!(result.is_err());
    match result.unwrap_err() {
      Error::MultipleTypes { .. } => {} // Expected error
      other => panic!("Expected MultipleTypes error, got {:?}", other),
    }
  }

  #[test]
  fn test_enum_attributes_from_attributes() {
    let attrs: Vec<Attribute> = parse_quote! {
        #[ld(prefix(#TEST_PREFIX = #TEST_PREFIX_IRI))]
    };

    let enum_attrs = RdfEnumAttributes::try_from(attrs).unwrap();

    let prefix = Prefix::from_str(TEST_PREFIX).unwrap();
    let mappings = &enum_attrs.prefix_mappings;
    assert!(mappings.get(prefix).is_some());
  }

  #[test]
  fn test_invalid_iri_in_prefix_attribute() {
    let prefix_attr: PrefixAttribute =
      parse_quote! { prefix(#TEST_PREFIX = "not a valid iri with spaces") };

    let result = PrefixMappings::try_from(prefix_attr);
    assert!(result.is_err());
    match result.unwrap_err() {
      Error::InvalidIri { .. } => {} // Expected error
      other => panic!("Expected InvalidIri error, got {:?}", other),
    }
  }

  #[test]
  fn test_invalid_prefix_in_prefix_attribute() {
    let prefix_attr: PrefixAttribute = parse_quote! { prefix("invalid:prefix" = #TEST_PREFIX_IRI) };

    let result = PrefixMappings::try_from(prefix_attr);
    assert!(result.is_err());
    match result.unwrap_err() {
      Error::InvalidMapping { .. } => {} // Expected error
      other => panic!("Expected InvalidMapping error, got {:?}", other),
    }
  }

  #[test]
  fn test_empty_struct_attributes() {
    let attrs: Vec<Attribute> = parse_quote! {
        #[derive(Debug)]
    };

    let struct_attrs = RdfStructAttributes::try_from(attrs).unwrap();

    assert!(struct_attrs.r#type.is_none());
    let prefix = Prefix::from_str("nonexistent").unwrap();
    assert!(struct_attrs.prefix_mappings.get(prefix).is_none());
  }
}
