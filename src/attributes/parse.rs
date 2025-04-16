use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Result, Token};

use crate::attributes::ast::{
  EnumAttribute, FieldAttribute, PrefixAttribute, PrefixMapping, StructAttribute, TypeAttribute,
  VariantAttribute,
};

mod kw {
  syn::custom_keyword!(prefix);
  syn::custom_keyword!(ignore);
  syn::custom_keyword!(flatten);
  syn::custom_keyword!(id);
  syn::custom_keyword!(graph);
}

impl Parse for StructAttribute {
  fn parse(input: ParseStream) -> Result<Self> {
    let lookahead = input.lookahead1();

    if lookahead.peek(Token![type]) {
      let type_attr: TypeAttribute = input.parse()?;
      Ok(StructAttribute::Type(type_attr))
    } else if lookahead.peek(kw::prefix) {
      let prefix_attr: PrefixAttribute = input.parse()?;
      Ok(StructAttribute::Prefix(prefix_attr))
    } else {
      Err(lookahead.error())
    }
  }
}

impl Parse for EnumAttribute {
  fn parse(input: ParseStream) -> Result<Self> {
    let lookahead = input.lookahead1();

    if lookahead.peek(kw::prefix) {
      let prefix_attr: PrefixAttribute = input.parse()?;
      Ok(EnumAttribute::Prefix(prefix_attr))
    } else {
      Err(lookahead.error())
    }
  }
}

impl Parse for VariantAttribute {
  fn parse(input: ParseStream) -> Result<Self> {
    let iri = input.parse::<LitStr>()?;
    Ok(VariantAttribute::Iri(iri))
  }
}

impl Parse for FieldAttribute {
  fn parse(input: ParseStream) -> Result<Self> {
    let lookahead = input.lookahead1();

    if input.peek(LitStr) {
      let lit_str = input.parse::<LitStr>()?;
      Ok(FieldAttribute::Iri(lit_str))
    } else if lookahead.peek(kw::ignore) {
      let _: kw::ignore = input.parse()?;
      Ok(FieldAttribute::Ignore)
    } else if lookahead.peek(kw::flatten) {
      let _: kw::flatten = input.parse()?;
      Ok(FieldAttribute::Flatten)
    } else if lookahead.peek(kw::id) {
      let _: kw::id = input.parse()?;
      Ok(FieldAttribute::Id)
    } else if lookahead.peek(kw::graph) {
      let _: kw::graph = input.parse()?;
      Ok(FieldAttribute::Graph)
    } else {
      Err(lookahead.error())
    }
  }
}

impl Parse for TypeAttribute {
  fn parse(input: ParseStream) -> Result<Self> {
    // Parse the actual "type" keyword token
    let _: Token![type] = input.parse()?;
    input.parse::<Token![=]>()?;
    let identifier = input.parse::<LitStr>()?;
    Ok(TypeAttribute { identifier })
  }
}

impl Parse for PrefixAttribute {
  fn parse(input: ParseStream) -> Result<Self> {
    // Parse the prefix keyword
    let _: kw::prefix = input.parse()?;

    // Parse the content in parentheses
    let content;
    syn::parenthesized!(content in input);

    // Parse the mapping within the parentheses
    let mapping = content.parse()?;

    Ok(PrefixAttribute { mapping })
  }
}

impl Parse for PrefixMapping {
  fn parse(input: ParseStream) -> Result<Self> {
    let prefix = input.parse::<LitStr>()?;
    input.parse::<Token![=]>()?;
    let value = input.parse::<LitStr>()?;

    Ok(PrefixMapping { prefix, iri: value })
  }
}

#[cfg(test)]
mod tests {
  use core::panic;

  use syn::parse_quote;

  use super::*;

  const IRI: &str = "http://foo/";
  const PREFIX: &str = "foo";

  #[test]
  fn test_struct_attribute_parse() {
    let attr: StructAttribute = parse_quote! { type = #IRI };
    match &attr {
      StructAttribute::Type(r#type) => {
        assert_eq!(r#type.identifier.value(), IRI);
      }
      _ => panic!(),
    }
  }

  #[test]
  fn test_struct_prefix_attribute_parse() {
    let attr: StructAttribute = parse_quote! { prefix(#PREFIX = #IRI) };
    match &attr {
      StructAttribute::Prefix(prefix_attr) => {
        assert_eq!(prefix_attr.mapping.prefix.value(), PREFIX);
        assert_eq!(prefix_attr.mapping.iri.value(), IRI);
      }
      _ => panic!(),
    }
  }

  #[test]
  fn test_enum_prefix_attribute_parse() {
    let attr: EnumAttribute = parse_quote! { prefix(#PREFIX = #IRI) };
    match &attr {
      EnumAttribute::Prefix(prefix_attr) => {
        assert_eq!(prefix_attr.mapping.prefix.value(), PREFIX);
        assert_eq!(prefix_attr.mapping.iri.value(), IRI);
      }
    }
  }

  #[test]
  fn test_variant_attribute_parse() {
    let attr: VariantAttribute = parse_quote! { #IRI };
    match &attr {
      VariantAttribute::Iri(iri) => {
        assert_eq!(iri.value(), IRI);
      }
    }
  }

  #[test]
  fn test_type_attribute_parse() {
    let type_attr: TypeAttribute = parse_quote! { type = #IRI };
    assert_eq!(type_attr.identifier.value(), IRI);
  }

  #[test]
  fn test_prefix_mapping_parse() {
    let mapping: PrefixMapping = parse_quote! { #PREFIX = #IRI };
    assert_eq!(mapping.prefix.value(), PREFIX);
    assert_eq!(mapping.iri.value(), IRI);
  }

  #[test]
  fn test_error_on_invalid_attribute() {
    let result: Result<StructAttribute> = syn::parse2(quote::quote! {
        unknown = "value"
    });
    assert!(result.is_err());

    let result: Result<EnumAttribute> = syn::parse2(quote::quote! {
        typ = "not allowed"
    });
    assert!(result.is_err());
  }

  #[test]
  fn test_field_ignore_parse() {
    let attr: FieldAttribute = parse_quote! { ignore };
    match attr {
      FieldAttribute::Ignore => {}
      _ => panic!("Expected Ignore variant"),
    }
  }

  #[test]
  fn test_field_iri_parse() {
    let attr: FieldAttribute = parse_quote! { #IRI };
    match attr {
      FieldAttribute::Iri(iri) => assert_eq!(iri.value(), IRI),
      _ => panic!("Expected Iri variant"),
    }
  }

  #[test]
  fn test_field_flatten_parse() {
    let attr: FieldAttribute = parse_quote! { flatten };
    match attr {
      FieldAttribute::Flatten => {}
      _ => panic!("Expected Flatten variant"),
    }
  }

  #[test]
  fn test_field_id_and_graph_parse() {
    let id_attr: FieldAttribute = parse_quote! { id };
    let graph_attr: FieldAttribute = parse_quote! { graph };

    match id_attr {
      FieldAttribute::Id => {}
      _ => panic!("Expected Id variant"),
    }

    match graph_attr {
      FieldAttribute::Graph => {}
      _ => panic!("Expected Graph variant"),
    }
  }
}
