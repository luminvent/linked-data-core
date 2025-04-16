use syn::LitStr;

/// Represents attribute contents that can be parsed from #[ld(...)] on structs.
/// 
/// Possible formats:
/// - type = "http://example.org/Person"
/// - prefix("ex" = "http://example.org/")
#[derive(Debug)]
pub enum StructAttribute {
    Type(TypeAttribute),
    Prefix(PrefixAttribute),
}

/// Represents attribute contents that can be parsed from #[ld(...)] on enums.
///
/// Possible formats:
/// - prefix("ex" = "http://example.org/")
#[derive(Debug)]
pub enum EnumAttribute {
    Prefix(PrefixAttribute),
}

/// Represents attribute contents that can be parsed from #[ld(...)] on enum variants.
///
/// Possible formats:
/// - "http://example.org/property"
#[derive(Debug)]
pub enum VariantAttribute {
    Iri(LitStr),
}

/// Represents a type attribute value.
///
/// Format: type = "http://example.org/Type" or type = "prefix:Type"
#[derive(Debug)]
pub struct TypeAttribute {
    pub identifier: LitStr,
}

/// Represents a prefix attribute value.
///
/// Format: prefix("ex" = "http://example.org/")
#[derive(Debug)]
pub struct PrefixAttribute {
    pub mapping: PrefixMapping,
}

/// Represents a prefix mapping with a prefix and an IRI.
///
/// Format: "ex" = "http://example.org/"
#[derive(Debug)]
pub struct PrefixMapping {
    pub prefix: LitStr,
    pub iri: LitStr,
}
