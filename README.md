# Linked data core

[![Crates.io](https://img.shields.io/crates/v/linked-data-core.svg)](https://crates.io/crates/linked-data-core)
[![Documentation](https://docs.rs/linked-data-core/badge.svg)](https://docs.rs/linked-data-core)
[![CI](https://github.com/luminvent/linked-data-core/actions/workflows/ci.yml/badge.svg)](https://github.com/luminvent/linked-data-core/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Core library for building Linked Data derive macros.

`linked-data-core` parses Rust structs and enums annotated with `#[ld(...)]` attributes into an
RDF-oriented model (`RdfType`, `RdfStruct`, `RdfEnum`, `RdfField`, `RdfVariant`). It resolves
prefixed names into full IRIs and validates the annotations with spanned compile errors. Code
generation is left to you: implement the `TokenGenerator` trait to turn that model into whatever
your derive macro needs to emit (serialization, deserialization, SPARQL queries, etc.).

## Installation

This crate is meant to be used from a `proc-macro` crate:

```toml
[lib]
proc-macro = true

[dependencies]
linked-data-core = "0.2"
proc-macro-error3 = "3"
proc-macro2 = "1"
quote = "1"
syn = "3"
```

## Writing a derive macro

```rust
use linked_data_core::{RdfEnum, RdfField, RdfStruct, RdfType, RdfVariant, TokenGenerator};
use proc_macro::TokenStream;
use proc_macro_error3::proc_macro_error;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

struct MyGenerator;

impl TokenGenerator for MyGenerator {
  fn generate_type_tokens(rdf_type: &RdfType<Self>, tokens: &mut proc_macro2::TokenStream) {
    match rdf_type {
      RdfType::Struct(r#struct) => Self::generate_struct_tokens(r#struct, tokens),
      RdfType::Enum(r#enum) => Self::generate_enum_tokens(r#enum, tokens),
    }
  }

  fn generate_struct_tokens(r#struct: &RdfStruct<Self>, tokens: &mut proc_macro2::TokenStream) {
    let ident = &r#struct.ident;
    let type_iri = r#struct.type_iri().map(|iri| iri.as_str());
    // `r#struct.fields` are `RdfField`s, which also implement `ToTokens`.
    // ...
  }

  fn generate_enum_tokens(r#enum: &RdfEnum<Self>, tokens: &mut proc_macro2::TokenStream) {
    // ...
  }

  fn generate_variant_tokens(variant: &RdfVariant<Self>, tokens: &mut proc_macro2::TokenStream) {
    // ...
  }

  fn generate_field_tokens(field: &RdfField<Self>, tokens: &mut proc_macro2::TokenStream) {
    // ...
  }
}

#[proc_macro_error]
#[proc_macro_derive(MyLinkedData, attributes(ld))]
pub fn derive_my_linked_data(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let rdf_type = RdfType::<MyGenerator>::from_derive(input);
  quote!(#rdf_type).into()
}
```

`RdfType::from_derive` reports invalid annotations with `proc_macro_error3::abort!`, so the
derive entry point must be annotated with `#[proc_macro_error]`.

Every model type implements `quote::ToTokens` by delegating to the matching `TokenGenerator`
method, so they can be interpolated directly in `quote!`.

## Attributes

All annotations live in `#[ld(...)]` attributes. IRIs can be written in full
(`"http://schema.org/name"`) or as prefixed names (`"schema:name"`) once a prefix is declared on
the type.

### Structs

| Attribute                         | Description                                          |
| --------------------------------- | ---------------------------------------------------- |
| `prefix("ex" = "http://ex.org/")` | Declares a prefix usable in this type's IRIs.        |
| `type = "ex:Person"`              | The `rdf:type` of the resource (at most once).       |

### Struct fields

| Attribute    | Description                                                 |
| ------------ | ----------------------------------------------------------- |
| `"ex:name"`  | The predicate IRI linking the resource to the field value.  |
| `id`         | The field holds the subject IRI of the resource.            |
| `flatten`    | The field's own properties are merged into the parent.      |
| `graph`      | The field holds a named graph.                              |
| `ignore`     | The field is skipped.                                       |

```rust
#[derive(MyLinkedData)]
#[ld(prefix("schema" = "http://schema.org/"))]
#[ld(type = "schema:Person")]
struct Person {
  #[ld(id)]
  id: String,

  #[ld("schema:name")]
  name: String,

  #[ld(flatten)]
  contact: ContactInfo,

  #[ld(ignore)]
  cache: Option<String>,
}
```

### Enums

Enums accept `prefix(...)` like structs. An enum must contain either only single-field variants
or only unit variants; mixing both is rejected.

#### Tagged unions

Single-field (tuple) variants wrap a value. The IRI can be set on the variant or on its field:

```rust
#[derive(MyLinkedData)]
#[ld(prefix("ex" = "http://example.org/"))]
enum Contact {
  #[ld("ex:email")]
  Email(String),

  Phone(#[ld("ex:phone")] String),

  #[ld("ex:value")]
  Address(#[ld("ex:address")] Address),
}
```

`RdfVariant::predicate_path` returns the resulting `PredicatePath`:

- An IRI on either the variant or its field gives a direct predicate
  (`PredicatePath::Predicate`):

  ```turtle
  :s ex:email :o .
  ```

- An IRI on both gives a path through an intermediate blank node (`PredicatePath::ChainedPath`),
  where `to_blank` is the field's IRI and `from_blank` the variant's:

  ```turtle
  :s ex:address _:b .
  _:b ex:value :o .
  ```

#### Closed lists

When every variant is a unit variant, the enum is a closed list of RDF resources, such as a
controlled vocabulary. Each variant is identified by its own IRI, and `RdfEnum::is_closed_list`
returns `true`:

```rust
#[derive(MyLinkedData)]
#[ld(prefix("ex" = "http://example.org/status/"))]
enum Status {
  #[ld("ex:draft")]
  Draft,
  #[ld("ex:published")]
  Published,
}
```

## Errors

Invalid annotations are reported as compile errors pointing at the offending code, including:

- unions, and enum variants with more than one field;
- enums mixing unit and single-field variants;
- a variant without an IRI;
- more than one `type` on a struct, or more than one IRI on a field or variant;
- malformed IRIs and invalid prefixes.

## Development

```sh
cargo test --all-features --workspace
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
```

CI runs on Rust 1.86.0.

## License

Licensed under the [MIT license](LICENSE).
