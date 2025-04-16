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

/// Represents attribute contents that can be parsed from #[ld(...)] on struct fields.
///
/// Possible formats:
/// - ignore
/// - "http://example.org/property"
/// - flatten
/// - id
/// - type
/// - graph
#[derive(Debug)]
pub enum FieldAttribute {
    /// Marks the field to be ignored during serialization/deserialization
    Ignore,
    /// Specifies the IRI for the field
    Iri(LitStr),
    /// Indicates that field's contents should be flattened
    Flatten,
    /// Marks the field as an ID field
    Id,
    /// Marks the field as a graph value
    Graph,
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
