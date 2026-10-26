# Open Mission Systems Rust CAL Interface Generation Specification

**Document:** rcal-SPC-RS (draft)
**Baseline:** rcal 3.0.1, OMS CAL v2.5
**Companion to:** OMSC-SPC-001 Rev L (CAL Specification) and OMSC-SPC-008 Rev K (C++ CAL Interface Generation Specification)

This document is the Rust counterpart to OMSC-SPC-008. It retroactively specifies the API implemented by the `rcal` crate so that an alternate Rust CAL Implementation could be written against it, and so that rcal's conformance can be traced to the `CERT CAL-` and `CERT CXX-` requirements.

Chapter numbering follows OMSC-SPC-008 so sections can be read side by side. Where a C++ concept has no Rust analog the section says so and the difference is recorded in [Appendix A](#appendix-a--deviations-from-omsc-spc-008).

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [The XML Schema](#2-the-xml-schema)
3. [The Rust OMS Schema Compiler Conceptual Architecture](#3-the-rust-oms-schema-compiler-conceptual-architecture)
4. [Modules](#4-modules)
5. [Names and Qualified Names](#5-names-and-qualified-names)
6. [Crate Structure](#6-crate-structure)
7. [Accessor Construction and Destruction](#7-accessor-construction-and-destruction)
8. [Documentation and Annotations](#8-documentation-and-annotations)
9. [The Primitive and Basic Data Types](#9-the-primitive-and-basic-data-types)
10. [Accessors](#10-accessors)
11. [Rust CAL CERT Verification](#11-rust-cal-cert-verification)
12. [List of Symbols, Abbreviations, and Acronyms](#12-list-of-symbols-abbreviations-and-acronyms)
13. [Glossary](#13-glossary)
- [Appendix A — Deviations from OMSC-SPC-008](#appendix-a--deviations-from-omsc-spc-008)

---

## 1 Introduction

### 1.1 Interface Specification Goals

An OMS Rust CAL API:

- exposes the CAL capabilities of OMSC-SPC-001 through idiomatic, safe, stable Rust;
- reuses the C++ CAL names (OMSC-SPC-008) where they fit, converted to Rust naming conventions (`getMySystemUUID` → `get_system_uuid`);
- replaces C++ mechanisms that have a better Rust equivalent: exceptions become `Result`, destructors become `Drop` or consuming `self: Box<Self>` methods, copy constructors become `Clone`, and abstract classes become traits;
- is thread-safe by construction. Every shared CAL type is `Send + Sync` (OMSC-SPC-001 §5.1.1, CAL-016015).

### 1.2 References

#### 1.2.1 OMS D&D Documents

| Document | Title |
|---|---|
| OMSC-SPC-001 Rev L | Open Mission Systems CAL Specification |
| OMSC-SPC-008 Rev K | Open Mission Systems C++ CAL Interface Generation Specification |

#### 1.2.2 Other Documents

| Document | Title |
|---|---|
| IETF RFC 4122 | A Universally Unique IDentifier (UUID) URN Namespace |
| W3C XML Schema 1.0 | XML Schema Part 1: Structures; Part 2: Datatypes |
| W3C EXI 1.0 (2nd ed.) | Efficient XML Interchange Format |
| The Rust Reference / Rust API Guidelines | Language and naming conventions |

### 1.3 Conventions

- **Requirement IDs.** Normative requirements are labelled `CERT RS-NNNNNN`. IDs go up in steps of 10 so later revisions can insert requirements without renumbering. The **Derived from** column lists the `CERT CXX-` / `CAL-` requirements that each RS requirement implements or replaces.
- **"shall define X in module M"** means that `X` is a `pub` item reachable at the path `rcal::M::X`, either declared there or re-exported.
- **"shall declare method f"** means that the trait or type has a method with the given signature. Lifetimes may be elided. Additional `where` bounds are allowed if they don't narrow the set of valid callers.
- **Fallible** means the method returns `CalResult<T>` (§9.2). A method that is not marked fallible shall not return an error and shall not panic on valid input.
- Requirements stated for an *OMS Rust CAL API* apply to the public API surface. Requirements stated for an *OMS Rust CAL Implementation* apply to runtime behaviour.
- Text marked **(informative)** is non-normative.

---

## 2 The XML Schema

The Rust CAL consumes the same OMS Message Schema, and the same XSD subset, described in OMSC-SPC-008 §2 (schema components, namespaces, prefixes, primitive types, names, variants). That material is not repeated here. Chapters 9 and 10 define how each schema construct is realized in Rust.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000010 | An OMS Rust CAL API shall be generated from a single OMS Message Schema XSD (including its `xs:include`/`xs:import` closure) supplied at build time. | CAL-005179 |

---

## 3 The Rust OMS Schema Compiler Conceptual Architecture

In rcal the OMS Schema Compiler is the crate's Cargo build script (`build.rs`). It runs at `cargo build` time, parses the XSD, and writes Rust source into `$OUT_DIR/uci_types/`. That source is compiled into the `rcal::uci::types` module (§6).

```text
XSD ──► build.rs (schema compiler) ──► $OUT_DIR/uci_types/*.rs ──► rcal::uci::types
                                                                     │
  rcal::uci::base  (UUID, BoundedList, re-exports) ◄─────────────────┤
  rcal::asb        (AbstractServiceBus: identity, status, lifecycle) │
  rcal::cal        (AbstractCal, Cal, readers/writers, QoS) ◄────────┘
  rcal::externalizer (Externalizer, ExternalizerLoader)
```

**Build inputs**

| Environment variable | Purpose | Default |
|---|---|---|
| `RCAL_XSD_PATH` | Path to the OMS Message Schema XSD | The lexically greatest `schema/UCI_MessageDefinitions_*.xsd` |
| `RCAL_SCHEMA_VERSION` | Value returned by `oms_schema_version()` | `UCI_<xs:schema/@version>` |
| `RCAL_OMS_COMPILER_VERSION` | Value returned by `oms_schema_compiler_version()` | `rcal/<crate version>` |
| `RCAL_ASB_CONNECTION_VERSION` | Value returned by `get_asb_connection_version()` | `rcal/<crate version>` |
| `RCAL_OMS_API_VERSION` | Value returned by `get_oms_api_version()` | `rcal/<crate version>` |
| `RCAL_CALCONFIG_PATH` | Optional: limit generation to the message types used by topics in this CAL configuration file, plus their transitive dependencies | (generate all) |
| `RCAL_CALCONFIG_SERVICES` | Optional: comma-separated service IDs that further narrow `RCAL_CALCONFIG_PATH` | (all services) |

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000020 | An OMS Rust CAL schema compiler shall generate the OMS Rust CAL API types at crate build time without requiring a separate code-generation step by the CAL Client. | CAL-005179 |
| CERT RS-000030 | An OMS Rust CAL schema compiler shall fail the build with a diagnostic when the XSD cannot be read or when no schema version can be determined. | CAL-005179 |
| CERT RS-000040 | An OMS Rust CAL schema compiler shall, when a CAL configuration subset is supplied, generate every type transitively required by the selected message types. | CAL-005179 |

---

## 4 Modules

The C++ API uses namespaces and include files. The Rust API uses modules. No inclusion step is needed, and items are reachable once the crate is a dependency (subject to Cargo features).

| C++ namespace / header | Rust module | Contents |
|---|---|---|
| `uci::base` (`UCIException.h`) | `rcal::uci` | `CalError`, `CalErrorKind`, `CalImplementationErrorKind`, `CalResult`, `ValidationError`, `CalMessage`, `CalSubMessage`, `sealed` |
| `uci::base` (`UUID.h`, `BoundedList.h`, …) | `rcal::uci::base` | `UUID`, `BoundedList`, plus re-exports of the ASB, CAL and externalizer interfaces |
| `uci::type` (generated) | `rcal::uci::types` | Generated accessor traits, structs, enums, aliases (chapter 10) |
| `xs` | `rcal::xs` | XSD primitive type aliases (§9.1) |
| `uci::base` (`AbstractServiceBusConnection.h`) | `rcal::asb` | `AbstractServiceBus`, `AsbConnectionState`, `AsbStatus`, `AsbStatusListener` |
| `uci::base` (`Reader.h`, `Writer.h`, `Listener.h`) | `rcal::cal` | `AbstractCal`, `AbstractReader`, `AbstractWriter`, `MessageListener`, QoS types, `Cal`, `get_cal`, `create_cal` |
| `uci::base` (`Externalizer*.h`) | `rcal::externalizer` | `Externalizer`, `ExternalizerLoader`, read/write free functions |
| — | `rcal::qname` (also re-exported at `rcal::QName`, `rcal::NamespaceResolver`) | XML qualified names (§5) |
| — | `rcal::calconfig` | CAL configuration file model (informative) |
| — | `rcal::logging` | Logger construction (informative) |
| — | `rcal::exi` (feature `exi`) | EXI codec (informative) |
| — | `rcal::service` (feature `service`) | `AbstractService` helper layer (informative) |

**Cargo features (informative)**

| Feature | Default | Enables |
|---|---|---|
| `dds` | yes | DDS ASB transport (`dust_dds`) |
| `service` | yes | `rcal::service` module |
| `zmq` | no | ZeroMQ ASB transport |
| `activemq` | no | ActiveMQ (STOMP) ASB transport |
| `compression` | no | `CompressionExternalizer` (gzip/zlib/deflate) |
| `exi` | no | `rcal::exi` and `ExiExternalizer` |
| `tui` | no | `asb_view` terminal UI binary |

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000050 | An OMS Rust CAL API shall define the modules `uci`, `uci::base`, `uci::types`, `xs`, `asb`, `cal`, and `externalizer` at the crate root. | CXX-004937, CXX-004954, CXX-005296, CXX-005419, CXX-012071 |
| CERT RS-000060 | An OMS Rust CAL API shall re-export from `uci::base` the items `AbstractServiceBus`, `AsbConnectionState`, `AsbStatus`, `AsbStatusListener`, `AbstractCal`, `AbstractCalCreateMessage`, `AbstractReader`, `AbstractWriter`, `Cal`, `MessageListener`, `MessageHeaderDefaults`, `TopicQos`, `Reliability`, `TimeBasedFilter`, `Expiration`, `MessageBuffer`, `Externalizer`, and `ExternalizerLoader`, so that the C++ `uci::base` namespace has a single Rust counterpart. | CXX-005419, CXX-010675, CXX-010733, CXX-010792, CXX-012071, CXX-012338 |
| CERT RS-000070 | An OMS Rust CAL API shall gate optional transports and externalizers behind Cargo features so that a CAL Client links only the middleware it uses. | CAL-005179 |

---

## 5 Names and Qualified Names

### 5.1 Qualified names

`rcal::QName` represents an XML qualified name (namespace URI plus local name). `rcal::NamespaceResolver` maps prefixes to URIs and formats a `QName` in its shortest unambiguous form.

```rust
pub struct QName { /* private */ }
impl QName {
    pub fn new(namespace: Option<&str>, local: &str) -> Self;
    pub fn with_display(namespace: Option<&str>, local: &str, display: &str) -> Self;
    pub fn parse(s: &str) -> Self;          // "local" | "prefix:local" | "{uri}local"
    pub fn local(&self) -> &str;
    pub fn namespace(&self) -> Option<&str>;
    pub fn clark(&self) -> String;          // "{uri}local"
    pub fn resolve(&self, resolver: &NamespaceResolver) -> String;
}
// impl Display, From<&str>, From<String>, From<QName> for String,
//      PartialEq<str>, PartialEq<&str>, PartialEq<String>

pub struct NamespaceResolver { /* private */ }
impl NamespaceResolver {
    pub fn new() -> Self;
    pub fn with_default_ns(ns: impl Into<String>) -> Self;
    pub fn set_default_ns(&mut self, ns: impl Into<String>);
    pub fn add_prefix(&mut self, prefix: impl Into<String>, uri: impl Into<String>);
    pub fn default_ns(&self) -> Option<&str>;
    pub fn prefix_for_uri(&self, uri: &str) -> Option<&str>;
    pub fn uri_for_prefix(&self, prefix: &str) -> Option<&str>;
    pub fn format(&self, namespace: Option<&str>, local: &str) -> String;
    pub fn resolve(&self, name: &str) -> QName;
}
```

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000080 | An OMS Rust CAL API shall define the type `QName` in the `qname` module, re-exported at the crate root, with the methods listed in §5.1. | CAL-005208 |
| CERT RS-000090 | `QName::parse` shall accept bare local names, `prefix:local` names, and Clark notation `{uri}local`. | CAL-005208 |
| CERT RS-000100 | `QName::resolve` shall return the bare local name when the namespace equals the resolver's default namespace, `prefix:local` when the namespace maps to a known prefix, and Clark notation otherwise. | CAL-005208 |
| CERT RS-000110 | An OMS Rust CAL API shall define the type `NamespaceResolver` in the `qname` module, re-exported at the crate root, with the methods listed in §5.1. | CAL-005208 |

### 5.2 Generated Rust names

The schema compiler derives Rust identifiers from XSD names as follows.

| XSD construct | Rust identifier | Example |
|---|---|---|
| complexType / simpleType / element name | Local part of the name, unchanged (PascalCase is preserved, prefix dropped) | `uci:SystemStatusMT` → `SystemStatusMT` |
| Concrete struct for a complexType or global element | Type name plus a trailing `_` | `SystemStatusMT_`, `SystemStatus_` |
| Element/field and accessor method name | snake_case: `_` inserted before an uppercase letter that follows a lowercase letter; runs of capitals are kept together; existing `_` are collapsed | `MessageHeader` → `message_header`, `SystemID` → `system_id` |
| Field name that is a Rust keyword | Keyword plus a trailing `_` | `type` → `type_` |
| Enumeration value | First character uppercased, remainder lowercased; a leading digit is prefixed with `V` | `TS` → `Ts`, `3D` → `V3d` |

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000120 | An OMS Rust CAL schema compiler shall name each generated accessor trait, enum, and type alias after the local part of the XSD type name. | CXX-005456, CXX-006166 |
| CERT RS-000130 | An OMS Rust CAL schema compiler shall name the concrete struct backing a complexType or global element `<TypeName>_`. | CXX-005456 |
| CERT RS-000140 | An OMS Rust CAL schema compiler shall convert element names to snake_case accessor names according to §5.2, appending `_` to any name that collides with a Rust keyword. | CXX-011213, CXX-011214 |
| CERT RS-000150 | An OMS Rust CAL schema compiler shall convert enumeration values to Rust enum variant names according to §5.2. | CXX-006172, CXX-006173 |

---

## 6 Crate Structure

Generated files are written one per XSD type to `$OUT_DIR/uci_types/<snake_name>.rs`. A generated `mod.rs` declares each file as a `#[doc(hidden)]` submodule and re-exports its contents with `#[doc(inline)] pub use <file>::*;`. As a result, every generated item is reachable directly at `rcal::uci::types::<Name>`.

Hand-written extensions live alongside the generated code in `src/uci/types/`, e.g. `SecurityInformationExt` (informative).

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000160 | An OMS Rust CAL API shall make every generated type reachable at `rcal::uci::types::<Name>` without additional sub-module qualification. | CXX-005456, CXX-006143 |

---

## 7 Accessor Construction and Destruction

A C++ CAL controls lifetimes by hiding constructors and destructors and providing `create`/`destroy` factories. In Rust:

- **Construction.** Generated message structs embed a private `uci::sealed::Token` field whose inner value only the `rcal` crate can construct. Outside the crate, a CAL Message can therefore only be created through `AbstractCalCreateMessage::create_message` (§9.10.4), which also pre-populates the message header. Sub-message structs implement `Default`.
- **Copying.** Generated types implement `Clone`. This replaces both the copy constructor and `copy()`.
- **Destruction.** Ordinary `Drop`. Resources that need an orderly shutdown and can report errors (readers, writers) have a consuming `close(self: Box<Self>)`, so use-after-close is a compile error.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000170 | An OMS Rust CAL API shall prevent CAL Clients from constructing a `CalMessage` type except through `AbstractCalCreateMessage::create_message`. | CAL-016035, CXX-005464, CXX-011066 |
| CERT RS-000180 | An OMS Rust CAL API shall define the type `Token` in the `uci::sealed` module whose field is not constructible outside the CAL crate. | CAL-016035 |
| CERT RS-000190 | An OMS Rust CAL API shall implement `Clone` for every generated accessor struct and enum. | CXX-005465, CXX-011063 |
| CERT RS-000200 | An OMS Rust CAL API shall release resources owned by any CAL object when it is dropped, without requiring an explicit destroy call. | CXX-011068, CXX-012381, CXX-012446 |

---

## 8 Documentation and Annotations

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000210 | An OMS Rust CAL schema compiler shall emit each `xs:annotation/xs:documentation` text as rustdoc on the corresponding generated trait, trait method, struct, field, or enum variant. | CAL-005179 |
| CERT RS-000220 | An OMS Rust CAL schema compiler shall mark generated files `// @generated — do not edit.` | CAL-005179 |

The hand-written modules `uci`, `uci::base`, `asb`, `cal` and `xs` enable `#![warn(missing_docs)]` (informative).

---

## 9 The Primitive and Basic Data Types

### 9.1 The Primitive Types

Defined in module `rcal::xs`. Each type is a `pub type` alias except `DateTime`, which is a newtype.

| XSD type | Rust name | Rust type | Notes |
|---|---|---|---|
| `xs:boolean` | `Boolean` | `bool` | |
| `xs:long` | `Long` | `i64` | |
| `xs:int` | `Int` | `i32` | |
| `xs:short` | `Short` | `i16` | |
| `xs:byte` | `Byte` | `i8` | |
| `xs:unsignedLong` | `UnsignedLong` | `u64` | |
| `xs:unsignedInt` | `UnsignedInt` | `u32` | |
| `xs:unsignedShort` | `UnsignedShort` | `u16` | |
| `xs:unsignedByte` | `UnsignedByte` | `u8` | |
| `xs:double` | `Double` | `f64` | |
| `xs:float` | `Float` | `f32` | |
| `xs:integer` | `Integer` | `i64` | CAL-016024 |
| `xs:duration` | `Duration` | `i64` (nanoseconds, signed) | CAL-016027 |
| `xs:dateTime` | `DateTime` | newtype over `chrono::DateTime<Utc>`; serialized as `i64` ns since POSIX epoch | CAL-016028 |
| `xs:time` | `Time` | `i64` (ns since 00:00:00Z) | CAL-016029 |
| `xs:string` | `XsString`, `StringAccessor` | `String` | |
| `xs:anyURI` | `AnyUri` | `String` | |
| `xs:normalizedString` | `NormalizedString` | `String` | |
| `xs:token` | `Token` | `String` | |
| `xs:hexBinary` | `HexBinary` | `Vec<u8>` | |
| `xs:base64Binary` | `Base64Binary` | `Vec<u8>` | |

```rust
pub struct DateTime(/* chrono::DateTime<chrono::Utc> */);
impl DateTime { pub fn now() -> Self; }
// derive Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash
// impl Default (UNIX epoch), Serialize/Deserialize (i64 ns),
//      From<chrono::DateTime<Utc>>, From<DateTime> for chrono::DateTime<Utc>,
//      From<i64>, From<DateTime> for i64
```

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000230 | An OMS Rust CAL API shall define, in the `xs` module, each Rust name in the §9.1 table as equivalent to the listed Rust type. | CXX-004937, CXX-004940, CXX-004946 |
| CERT RS-000240 | An OMS Rust CAL API shall represent `xs:integer` as `i64`. | CAL-016024 |
| CERT RS-000250 | An OMS Rust CAL API shall represent `xs:duration` as a signed 64-bit count of nanoseconds. | CAL-016027 |
| CERT RS-000260 | An OMS Rust CAL API shall define the type `DateTime` in the `xs` module that serializes as a signed 64-bit count of nanoseconds since 1970-01-01T00:00:00Z and converts losslessly to and from `i64` within the representable range. | CAL-016028 |
| CERT RS-000270 | An OMS Rust CAL API shall represent `xs:time` as a signed 64-bit count of nanoseconds since 00:00:00Z. | CAL-016029 |
| CERT RS-000280 | An OMS Rust CAL API shall represent string primitive types with owned `String` values so that CAL Messages are `Send + Sync + 'static`. | CXX-004940, CAL-016015 |

### 9.2 The Error Types (UCIException)

`UCIException` is replaced by a `Result`-based error type in module `rcal::uci`.

```rust
pub type CalResult<T> = Result<T, CalError>;

#[derive(Debug)]
pub struct CalError { /* private: kind, message, source */ }
impl CalError {
    pub fn new(kind: CalErrorKind, message: impl Into<String>) -> Self;
    pub fn with_source(kind: CalErrorKind, message: impl Into<String>,
                       source: impl std::error::Error + Send + Sync + 'static) -> Self;
    pub fn new_impl(kind: CalImplementationErrorKind, message: impl Into<String>) -> Self;
    pub fn with_impl_source(kind: CalImplementationErrorKind, message: impl Into<String>,
                            source: impl std::error::Error + Send + Sync + 'static) -> Self;
    pub fn kind(&self) -> &CalErrorKind;
    pub fn message(&self) -> &str;
    pub fn is_asb_failure(&self) -> bool;
}
// impl Display, std::error::Error (source() returns the wrapped cause)

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalErrorKind {
    InitializationFailure,                                   // CAL-005204
    TopicUnavailable,                                        // CAL-005369
    ResourcesUnavailable,                                    // CAL-016043
    InvalidState { current: AsbConnectionState },            // SPC-001 Table 5.9-2
    UuidConformanceError,                                    // CAL-016477
    OperationNotPermitted,                                   // CAL-016050
    AsbFailed,                                               // SPC-001 Table 5.9-2
    InvalidServiceIdentifier,
    SerializationError,
    AbstractInstantiation,                                   // CAL-016035
    ImplementationError { kind: Option<CalImplementationErrorKind> },
    ValidationError(ValidationError),
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalImplementationErrorKind { ConfigError, ListenerError, UserInterfaceError }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError { pub path: String, pub reason: String }
// impl Display ("{path}: {reason}"), std::error::Error
```

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000290 | An OMS Rust CAL API shall define the type `CalError` in the `uci` module that implements `std::error::Error + Display + Debug + Send + Sync`. | CXX-004954 |
| CERT RS-000300 | An OMS Rust CAL API shall define the type alias `CalResult<T>` in the `uci` module equivalent to `Result<T, CalError>`. | CXX-004954 |
| CERT RS-000310 | Every fallible operation of an OMS Rust CAL API shall report failure by returning `Err(CalError)` rather than by panicking. | CXX-004954, CXX-004971 |
| CERT RS-000320 | An OMS Rust CAL API shall declare the `CalError` constructors `new`, `with_source`, `new_impl`, and `with_impl_source`, and the accessors `kind`, `message`, and `is_asb_failure`, with the signatures in §9.2. | CXX-004962, CXX-004964, CXX-004966, CXX-004969 |
| CERT RS-000330 | `CalError` shall expose an optional causal error through `std::error::Error::source`. | CXX-004954 |
| CERT RS-000340 | An OMS Rust CAL API shall define the `#[non_exhaustive]` enum `CalErrorKind` in the `uci` module with at least the variants listed in §9.2. | CXX-004959, CAL-005204, CAL-005369, CAL-016035, CAL-016043, CAL-016050, CAL-016477 |
| CERT RS-000350 | An OMS Rust CAL API shall define the `#[non_exhaustive]` enum `CalImplementationErrorKind` in the `uci` module with at least the variants `ConfigError`, `ListenerError`, and `UserInterfaceError`. | CXX-004959 |
| CERT RS-000360 | An OMS Rust CAL API shall define the type `ValidationError` in the `uci` module with public fields `path: String` (dot-separated element path) and `reason: String`. | CAL-005179 |

### 9.3 The Accessor Type Constants

Not applicable. Rust has native type identity (`std::any::TypeId`, generics and trait objects), so the run-time `AccessorType` constants of OMSC-SPC-008 §9.3 are not defined. See Appendix A.

### 9.4 The Accessor Class

There is no common `Accessor` base class. Its role is filled by two marker traits in `rcal::uci`:

```rust
pub trait CalMessage: Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned {
    fn message_type_name() -> crate::QName where Self: Sized;
    #[doc(hidden)] fn cal_create() -> Self where Self: Sized;
    fn is_valid(&self) -> Result<(), ValidationError> { Ok(()) }
    fn as_message_type_mut(&mut self) -> Option<&mut dyn crate::uci::types::MessageType> { None }
}

pub trait CalSubMessage: Send + Sync + 'static {}
```

`reset()` has no direct counterpart. Assign `Default::default()` to a sub-message, or create a new message with `create_message`.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000370 | An OMS Rust CAL API shall define the trait `CalMessage` in the `uci` module with supertraits `Send + Sync + 'static + Serialize + DeserializeOwned` and the methods in §9.4. | CXX-004987, CAL-005208, CAL-016015 |
| CERT RS-000380 | `CalMessage::message_type_name` shall return the fully-qualified OMS message type name of the implementing type. | CAL-005208 |
| CERT RS-000390 | `CalMessage::is_valid` shall return `Ok(())` if every field satisfies its schema constraints, otherwise `Err(ValidationError)` for the first failing field. | CXX-011057 |
| CERT RS-000400 | An OMS Rust CAL API shall define the marker trait `CalSubMessage` in the `uci` module with supertraits `Send + Sync + 'static`. | CXX-004987, CAL-016015 |
| CERT RS-000410 | Only concrete, element-backed message types shall implement `CalMessage`. Abstract types and types not backed by a global element shall implement `CalSubMessage` instead. | CAL-016035 |

### 9.5 The StringAccessor Class

`StringAccessor` is an alias for `String` (§9.1). `str()`, `c_str()`, `setStringValue()` and the assignment and conversion operators are provided by `String`, `&str` and `From`/`Into`.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000420 | An OMS Rust CAL API shall define the type `StringAccessor` in the `xs` module equivalent to `String`. | CXX-005002, CXX-005008, CXX-005009, CXX-011141, CXX-011142, CXX-011146 |

### 9.6 The SimpleList Template

Schema list types (`xs:list`) and repeated primitive elements use `BoundedList<T, MIN, MAX>` (§9.7) or `Vec<T>`. No separate `SimpleList` type is defined.

### 9.7 The BoundedList Template

```rust
#[derive(Serialize, Deserialize)] #[serde(transparent)]
pub struct BoundedList<T, const MIN: usize = 0, const MAX: usize = { usize::MAX }>(/* Vec<T> */);

impl<T, const MIN: usize, const MAX: usize> BoundedList<T, MIN, MAX> {
    pub const UNBOUNDED_BOUND: usize = usize::MAX;
    pub fn new() -> Self;
    pub fn is_empty(&self) -> bool;
    pub fn get_minimum_occurs(&self) -> usize;   // == MIN
    pub fn get_maximum_occurs(&self) -> usize;   // == MAX
    pub fn is_valid_at(&self, path: &str) -> Result<(), ValidationError>;
}
impl<T: Default, const MIN: usize, const MAX: usize> BoundedList<T, MIN, MAX> {
    pub fn resize(&mut self, new_len: usize);    // fills with T::default()
}
// impl Deref<Target = Vec<T>>, DerefMut, Default, Debug, Clone, PartialEq,
//      From<Vec<T>>, From<BoundedList> for Vec<T>, IntoIterator (owned, &, &mut)
```

`size`, `max_size`, `capacity`, `empty`, `reserve`, `pop_back`, `clear`, `operator[]`, `at` and `push_back` come from `Vec<T>` through `Deref`/`DerefMut` (`len`, `capacity`, `is_empty`, `reserve`, `pop`, `clear`, `[]`, `get`, `push`).

**(informative)** Cardinality is checked by `is_valid_at`, which `CalMessage::is_valid` calls before a write, not on every `push`. A list can temporarily violate its bounds while it is being built.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000430 | An OMS Rust CAL API shall define the generic type `BoundedList<T, const MIN: usize = 0, const MAX: usize = usize::MAX>` in the `uci::base` module. | CXX-005168, CXX-005172 |
| CERT RS-000440 | `BoundedList` shall define the associated constant `UNBOUNDED_BOUND: usize = usize::MAX`. | CXX-005174 |
| CERT RS-000450 | `BoundedList` shall declare `get_minimum_occurs(&self) -> usize` returning `MIN` and `get_maximum_occurs(&self) -> usize` returning `MAX`. | CXX-005224, CXX-005233 |
| CERT RS-000460 | `BoundedList` shall dereference to `Vec<T>` (shared and mutable) so that the standard sequence operations are available. | CXX-005179, CXX-005181, CXX-005184, CXX-005187, CXX-005195, CXX-005216, CXX-011183, CXX-011184, CXX-011186, CXX-011187, CXX-011189, CXX-011190, CXX-011191, CXX-005249, CXX-005262 |
| CERT RS-000470 | `BoundedList<T: Default>` shall declare `resize(&mut self, new_len: usize)`, which truncates or extends the list, filling new slots with `T::default()`. | CXX-011201 |
| CERT RS-000480 | `BoundedList::is_valid_at` shall return `Err(ValidationError)` with the supplied `path` when the length is outside `MIN..=MAX`. | CXX-005224, CXX-005233 |
| CERT RS-000490 | `BoundedList` shall serialize and deserialize transparently as its element sequence. | CXX-005168 |

### 9.8 The Universally Unique Identification (UUID) Class

```rust
#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UUID(/* uuid::Uuid */);

impl UUID {
    pub fn parse_str(s: &str) -> CalResult<Self>;                     // fromString
    pub fn from_octets(bytes: [u8; 16]) -> CalResult<Self>;           // fromOctets
    pub fn try_from_raw(uuid: uuid::Uuid) -> CalResult<Self>;
    pub const fn nil() -> Self;                                       // UUID()
    pub fn generate(factory: Option<&calconfig::UUIDFactory>) -> Self;
    pub fn generate_v4() -> Self;                                     // generateUUID
    pub fn generate_v1(timestamp: UuidTimestamp, node_id: &[u8; 6]) -> Self;
    pub fn generate_v3(namespace: &UUID, name: &[u8]) -> Self;        // createVersion3UUID
    pub fn is_nil(&self) -> bool;                                     // isNil
    pub fn is_valid(&self) -> bool;                                   // isValid
    pub fn get_variant(&self) -> uuid::Variant;                       // getVariant
    pub fn get_version(&self) -> Option<uuid::Version>;               // getVersion
    pub fn as_bytes(&self) -> &[u8; 16];                              // getOctets
    pub fn as_raw(&self) -> &uuid::Uuid;
    pub fn into_raw(self) -> uuid::Uuid;
}
// impl Display (lowercase hyphenated; toString), Debug, FromStr<Err = CalError>,
//      slog::Value
pub use uuid::Timestamp as UuidTimestamp;
pub use uuid::timestamp::context::ContextV1;
```

The comparison operators (`==`, `!=`, `<`, `<=`, `>`, `>=`) come from the derived `PartialEq`/`Ord`, and the `std::hash` specialization from the derived `Hash`.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000500 | An OMS Rust CAL API shall define the type `UUID` in the `uci::base` module implementing `Clone`, `Copy`, `Default` (nil), `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Display`, `Debug`, `FromStr`, `Serialize`, and `Deserialize`. | CXX-005296, CXX-005339, CXX-005340, CXX-005351, CXX-005358, CXX-005365, CXX-005372, CXX-005411, CXX-011154, CXX-012657 |
| CERT RS-000510 | Every `UUID` value obtainable through the public API shall be either the nil UUID or an RFC 4122-variant UUID of version 1, 3, or 4. | CAL-005181, CAL-016477, CAL-016479 |
| CERT RS-000520 | `UUID::parse_str`, `UUID::from_octets`, and `UUID::try_from_raw` shall return `Err` with kind `UuidConformanceError` for malformed input or for a UUID that violates RS-000510. | CXX-011150, CXX-011152, CAL-016477 |
| CERT RS-000530 | An OMS Rust CAL API shall declare the infallible `UUID` generators `generate_v4`, `generate_v1`, and `generate_v3` with the signatures in §9.8. | CXX-011165, CXX-012587, CAL-005181 |
| CERT RS-000540 | `UUID::generate` shall produce a version 4 UUID when the supplied factory, or the default when `None`, is `Random`, and a version 1 UUID when it is `TimeBased`, using the configured node ID or the host MAC address. | CXX-011165, CAL-005181 |
| CERT RS-000550 | `UUID::nil` shall return the all-zero UUID, and `UUID::default()` shall equal `UUID::nil()`. | CXX-012628 |
| CERT RS-000560 | An OMS Rust CAL API shall declare the `UUID` inspectors `is_nil`, `is_valid`, `get_variant`, `get_version`, and `as_bytes` with the signatures in §9.8. | CXX-005328, CXX-005332, CXX-005334, CXX-005379, CXX-012615 |
| CERT RS-000570 | `Display for UUID` shall produce the lowercase, hyphenated 36-character RFC 4122 form. | CXX-005331 |
| CERT RS-000580 | `FromStr for UUID` shall behave identically to `UUID::parse_str`. | CXX-011150, CXX-012641 |

### 9.9 The AbstractServiceBusConnection Class

Defined in module `rcal::asb`. `AbstractServiceBus` is object-safe and contains no message-type knowledge. Typed operations are in `AbstractCal` (§9.10).

```rust
pub trait AbstractServiceBus: Send + Sync {
    fn get_logger(&self) -> &slog::Logger;
    fn service_identifier(&self) -> &str;
    fn asb_identifier(&self) -> &str;
    fn get_system_uuid(&self) -> UUID;                                   // getMySystemUUID
    fn get_service_uuid(&self) -> Option<UUID>;                          // getMyServiceUUID
    fn get_subsystem_uuid(&self) -> Option<UUID>;                        // getMySubsystemUUID
    fn get_component_uuid(&self, name: &str) -> Option<UUID>;            // getMyComponentUUID
    fn get_capability_uuid(&self, name: &str) -> Option<UUID>;           // getMyCapabilityUUID
    fn oms_schema_version(&self) -> &str;                                // getOmsSchemaVersion
    fn oms_schema_compiler_version(&self) -> &str;                       // getOmsSchemaCompilerVersion
    fn get_system_label(&self) -> Option<&str>;                          // getMySystemLabel
    fn get_asb_connection_version(&self) -> &str;                        // getAbstractServiceBusConnectionVersion
    fn get_oms_api_version(&self) -> &str;                               // getOMSApiVersion
    fn connection_status(&self) -> &AsbStatus;                           // getStatus
    fn register_status_listener(&mut self, listener: Arc<dyn AsbStatusListener>) -> CalResult<()>;
    fn unregister_status_listener(&mut self, listener: &Arc<dyn AsbStatusListener>) -> CalResult<()>;
    fn close(&mut self) -> CalResult<()>;                                // shutdown
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AsbConnectionState { Initializing, Normal, Degraded, Inoperable, Failed }
impl AsbConnectionState {
    pub fn allows_write(&self) -> bool;
    pub fn allows_read_no_wait(&self) -> bool;
    pub fn allows_add_listener(&self) -> bool;
    pub fn is_operational(&self) -> bool;
    pub fn validate_transition(&self, next: AsbConnectionState) -> CalResult<()>;
}
// impl Display

#[derive(Debug, Clone)]
pub struct AsbStatus { pub state: AsbConnectionState, pub description: String }
impl AsbStatus { pub fn new(state: AsbConnectionState, description: impl Into<String>) -> Self; }

pub fn get_asb_config_location(path: Option<String>) -> CalResult<String>;
```

**Permitted state transitions** (OMSC-SPC-001 Figure 5.9-2). Self-transitions are not permitted.

| From \ To | Initializing | Normal | Degraded | Inoperable | Failed |
|---|---|---|---|---|---|
| Initializing | — | ✔ | ✔ | ✔ | ✔ |
| Normal | ✘ | — | ✔ | ✔ | ✔ |
| Degraded | ✘ | ✔ | — | ✔ | ✔ |
| Inoperable | ✔ | ✔ | ✔ | — | ✔ |
| Failed | ✘ | ✘ | ✘ | ✘ | — |

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000590 | An OMS Rust CAL API shall define the object-safe trait `AbstractServiceBus: Send + Sync` in the `asb` module with the methods in §9.9. | CXX-005419, CAL-005203, CAL-016015 |
| CERT RS-000600 | `AbstractServiceBus::get_system_uuid` shall return the configured system UUID, or the nil UUID when none is configured. | CXX-011168, CAL-005203 |
| CERT RS-000610 | `AbstractServiceBus::get_service_uuid`, `get_subsystem_uuid`, `get_component_uuid`, and `get_capability_uuid` shall return `None` when the corresponding identifier is not configured. | CXX-011169, CXX-011170, CXX-011171, CXX-011172 |
| CERT RS-000620 | `AbstractServiceBus::get_system_label` shall return the configured system label, or `None`. | CXX-005424 |
| CERT RS-000630 | `AbstractServiceBus::oms_schema_version`, `oms_schema_compiler_version`, `get_asb_connection_version`, and `get_oms_api_version` shall return the build-time values defined in chapter 3. | CXX-011174, CXX-011175, CXX-011176, CXX-012694 |
| CERT RS-000640 | `AbstractServiceBus::connection_status` shall return the current `AsbStatus`. | CXX-011295 |
| CERT RS-000650 | An OMS Rust CAL Implementation shall invoke `AsbStatusListener::on_status_change` with the current status before `register_status_listener` returns `Ok`. | CAL-016366, CXX-011082 |
| CERT RS-000660 | `AbstractServiceBus::register_status_listener` shall return `Err` with kind `InvalidState` when called in the `Failed` state. | CXX-011082 |
| CERT RS-000670 | `AbstractServiceBus::unregister_status_listener` shall identify the listener by `Arc` pointer equality and shall succeed without effect if the listener is not registered. | CXX-011083 |
| CERT RS-000680 | `AbstractServiceBus::close` shall shut down the CAL instance and release its resources. | CXX-011164 |
| CERT RS-000690 | An OMS Rust CAL API shall define the `#[non_exhaustive]` enum `AsbConnectionState` in the `asb` module with the variants `Initializing`, `Normal`, `Degraded`, `Inoperable`, and `Failed`. | CXX-011275 |
| CERT RS-000700 | `AsbConnectionState::validate_transition` shall return `Ok` exactly for the transitions marked ✔ in §9.9 and `Err` with kind `InvalidState` otherwise. | CXX-011275 |
| CERT RS-000710 | `AsbConnectionState::allows_write` and `allows_read_no_wait` shall return `true` only for `Normal` and `Degraded`. `allows_add_listener` shall return `false` only for `Failed`. | CXX-011275 |
| CERT RS-000720 | An OMS Rust CAL API shall define the type `AsbStatus` in the `asb` module with public fields `state: AsbConnectionState` and `description: String`. | CXX-011277, CXX-011279, CXX-011281 |
| CERT RS-000730 | An OMS Rust CAL API shall define the free function `get_asb_config_location` in the `asb` module, which resolves the CAL configuration path from its argument, then `RCAL_CALCONFIG_PATH`, then `./CALConfig.toml`, and returns `Err(InitializationFailure)` if the resolved file does not exist. | CXX-005433 |

#### 9.9.1 Obtaining a CAL instance

This is the Rust equivalent of `uci_getAbstractServiceBusConnection`. Defined in module `rcal::cal`.

```rust
pub async fn get_cal(service_identifier: impl Into<String>,
                     asb_identifier: Option<impl Into<String>>,
                     config: Arc<CalConfig>, logger: slog::Logger) -> CalResult<Cal>;

pub async fn create_cal(service_identifier: impl Into<String>,
                        asb_identifier: Option<impl Into<String>>,
                        config: Arc<CalConfig>, logger: slog::Logger) -> CalResult<Cal>;

#[derive(Clone)]
pub struct Cal { /* private */ }
// impl AbstractServiceBus, AbstractCal (and therefore AbstractCalCreateMessage)
// inherent pass-throughs: service_identifier, asb_identifier, message_header_defaults,
// create_message, create_writer, create_reader, close, connection_status,
// register_status_listener, unregister_status_listener, get_system_uuid,
// get_service_uuid, get_subsystem_uuid, get_component_uuid, get_capability_uuid,
// get_system_label, add_receive_peer
```

When `asb_identifier` is `None`, the transport is taken from the service's configured transport. If no matching transport exists, `[system] default_transport` is used.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000740 | An OMS Rust CAL API shall define the `async` free function `get_cal` in the `cal` module with the signature in §9.9.1, returning a fully initialized CAL instance. | CXX-005433, CAL-005201 |
| CERT RS-000750 | `get_cal` shall return a handle to the same CAL instance for every call with the same (service identifier, ASB identifier) pair within a process. | CAL-005202 |
| CERT RS-000760 | `get_cal` and `create_cal` shall return `Err` with kind `InitializationFailure` when no matching or default transport is configured or the transport fails to initialize. | CAL-005204 |
| CERT RS-000770 | An OMS Rust CAL API shall define the `async` free function `create_cal` in the `cal` module that returns a new, uncached CAL instance. | CAL-005201 |
| CERT RS-000780 | An OMS Rust CAL API shall define the type `Cal` in the `cal` module implementing `Clone`, `AbstractServiceBus`, and `AbstractCal`, where clones refer to the same underlying instance. | CXX-005431, CXX-005432, CXX-011177, CAL-005202 |

### 9.10 The Listener, Reader and Writer Classes

Defined in module `rcal::cal`. In C++, each global element defines its own `Reader`, `Writer` and `Listener` (OMSC-SPC-008 §10.3.4). Rust uses a single generic interface parameterized by the message type `M: CalMessage` instead.

#### 9.10.1 MessageListener

```rust
pub trait MessageListener<M: CalMessage>: Send + Sync {
    fn on_message(&self, message: &Arc<M>);
}
```

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000790 | An OMS Rust CAL API shall define the trait `MessageListener<M: CalMessage>: Send + Sync` in the `cal` module declaring `fn on_message(&self, message: &Arc<M>)`. | CXX-010675, CXX-005546, CXX-005564, CAL-005379 |
| CERT RS-000800 | An OMS Rust CAL Implementation shall deliver each received message to every registered listener exactly once, passing the same `Arc<M>` to each. | CAL-005392, CAL-016046 |
| CERT RS-000810 | An OMS Rust CAL Implementation shall remove a message from the reader buffer only after all registered listeners' `on_message` calls have returned. | CAL-016045 |

#### 9.10.2 AbstractReader

```rust
pub trait AbstractReader<M: CalMessage>: Send + Sync {
    fn topic(&self) -> &str;
    fn add_listener(&mut self, listener: Arc<dyn MessageListener<M>>) -> CalResult<()>;
    fn remove_listener(&mut self, listener: &Arc<dyn MessageListener<M>>) -> CalResult<()>;
    fn read(&mut self, timeout: Option<Duration>) -> CalResult<Option<Arc<M>>>;
    fn read_no_wait(&mut self) -> CalResult<Option<Arc<M>>>;
    fn close(self: Box<Self>) -> CalResult<()>;
}
```

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000820 | An OMS Rust CAL API shall define the trait `AbstractReader<M: CalMessage>: Send + Sync` in the `cal` module with the methods in §9.10.2. | CXX-010733, CXX-005567, CXX-011072, CXX-011073, CXX-011074, CXX-011075, CXX-011076, CAL-005378 |
| CERT RS-000830 | `AbstractReader::read` shall block until a message is available, the timeout (if `Some`) elapses (returning `Ok(None)`), or the reader is released. | CXX-011074 |
| CERT RS-000840 | `AbstractReader::read_no_wait` shall return immediately with `Ok(Some(_))` if a message is buffered and `Ok(None)` otherwise. | CXX-011075 |
| CERT RS-000850 | `AbstractReader::read` and `read_no_wait` shall return `Err` with kind `OperationNotPermitted` while any listener is registered. | CAL-016050 |
| CERT RS-000860 | Each successful `read` or `read_no_wait` shall remove the returned message from the reader buffer. | CAL-016052 |
| CERT RS-000870 | A blocked `read` shall be released with `Err` of kind `AsbFailed` when the ASB enters the `Failed` state. | CAL-016049 |
| CERT RS-000880 | `AbstractReader::close` shall consume the reader and release its resources. | CXX-011076, CXX-005495 |

#### 9.10.3 AbstractWriter

```rust
pub trait AbstractWriter<M: CalMessage>: Send + Sync {
    fn topic(&self) -> &str;
    fn write(&mut self, message: &M) -> CalResult<()>;
    fn close(self: Box<Self>) -> CalResult<()>;
}
```

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000890 | An OMS Rust CAL API shall define the trait `AbstractWriter<M: CalMessage>: Send + Sync` in the `cal` module with the methods in §9.10.3. | CXX-010792, CXX-005624, CXX-011078, CXX-011079, CAL-005368 |
| CERT RS-000900 | `AbstractWriter::write` shall return `Err` with kind `TopicUnavailable` if the topic is unreachable, `ResourcesUnavailable` if transport resources are exhausted, or `InvalidState` if the ASB state does not allow writes. | CAL-005369, CAL-016043 |
| CERT RS-000910 | An OMS Rust CAL Implementation shall call `CalMessage::is_valid` before transmitting and shall return `Err` with kind `ValidationError` without transmitting if validation fails. | CXX-011057 |
| CERT RS-000920 | `AbstractWriter::close` shall consume the writer and release its resources. | CXX-011079, CXX-005506 |

#### 9.10.4 AbstractCal, message creation, and QoS

```rust
pub trait AbstractCal: AbstractServiceBus {
    fn message_header_defaults(&self) -> MessageHeaderDefaults;
    fn create_writer<M: CalMessage>(&mut self, topic: &str, qos: TopicQos)
        -> CalResult<Box<dyn AbstractWriter<M>>>;
    fn create_reader<M: CalMessage>(&mut self, topic: &str, qos: TopicQos)
        -> CalResult<Box<dyn AbstractReader<M>>>;
}

pub trait AbstractCalCreateMessage {
    fn create_message<M: CalMessage>(&self) -> CalResult<M>;
}
impl<T: AbstractCal> AbstractCalCreateMessage for T { /* … */ }

#[derive(Debug, Clone)]
pub struct MessageHeaderDefaults {
    pub system_id: UUID, pub system_name: Option<String>,
    pub service_id: Option<UUID>, pub service_name: Option<String>,
    pub mission_id: Option<UUID>, pub schema_version: String,
    pub mode: MessageModeEnum, pub classification: ClassificationEnum,
    pub owner_producer: Vec<OwnerProducerChoiceType_>,
}

#[derive(Debug, Clone, Default)]
pub struct TopicQos {
    pub reliability: Reliability,
    pub time_based_filter: Option<TimeBasedFilter>,
    pub expiration: Option<Expiration>,
    pub writer_buffer: Option<MessageBuffer>,
    pub reader_buffer: Option<MessageBuffer>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Reliability { #[default] BestEffort, Reliable }
#[derive(Debug, Clone)] pub struct TimeBasedFilter { pub min_separation: Duration }
#[derive(Debug, Clone)] pub struct Expiration { pub max_age: Duration }
#[derive(Debug, Clone)] pub struct MessageBuffer { pub max_messages: usize }

#[macro_export]
macro_rules! update_message_header { ($msg:expr) => { /* refresh MessageHeader.Timestamp */ } }
```

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-000930 | An OMS Rust CAL API shall define the trait `AbstractCal: AbstractServiceBus` in the `cal` module with the methods in §9.10.4. | CAL-005201, CAL-005364, CAL-005374 |
| CERT RS-000940 | `AbstractCal::create_writer` and `create_reader` shall return `Err` with kind `TopicUnavailable` when `M::message_type_name()` differs from the message type configured for the topic. | CAL-005208, CXX-005494, CXX-005499 |
| CERT RS-000950 | `AbstractCal::create_writer` shall return `Err` with kind `OperationNotPermitted` for a topic configured as input-only, and `create_reader` shall do the same for a topic configured as output-only. | CAL-005364, CAL-005374 |
| CERT RS-000960 | `AbstractCal::create_reader` shall establish the topic connection and message buffering before returning. | CAL-005394, CAL-016044 |
| CERT RS-000970 | An OMS Rust CAL Implementation shall map the caller-supplied topic name to the configured CAL topic name when the configuration defines a mapping. | CAL-005209 |
| CERT RS-000980 | An OMS Rust CAL Implementation shall merge per-topic configured QoS into the caller's `TopicQos`, with caller-supplied `Some` values and non-default reliability taking precedence. | CAL-005210 |
| CERT RS-000990 | An OMS Rust CAL API shall define the trait `AbstractCalCreateMessage` in the `cal` module with a blanket implementation for every `AbstractCal`. | CXX-011066 |
| CERT RS-001000 | `create_message::<M>()` shall return a default-initialized `M` whose `MessageHeader` (SystemID, SchemaVersion, Mode, Timestamp, and when configured ServiceID and MissionID) and `SecurityInformation` (Classification, OwnerProducer) are populated from `message_header_defaults()`. | CXX-011066, CAL-016035 |
| CERT RS-001010 | An OMS Rust CAL API shall define `MessageHeaderDefaults`, `TopicQos`, `Reliability`, `TimeBasedFilter`, `Expiration`, and `MessageBuffer` in the `cal` module with the public fields in §9.10.4. | CAL-005210, CAL-005431, CAL-005434, CAL-005437, CAL-005444, CAL-005445 |
| CERT RS-001020 | `TopicQos::default()` shall select best-effort reliability with no time-based filter, no expiration, and unbounded buffers. | CAL-005210, CAL-016076 |
| CERT RS-001030 | An OMS Rust CAL Implementation shall drop a received message that arrives within `min_separation` of the previously accepted message when a `TimeBasedFilter` is set. | CAL-005431 |
| CERT RS-001040 | An OMS Rust CAL Implementation shall discard buffered messages older than `max_age` when an `Expiration` is set. | CAL-005437 |
| CERT RS-001050 | An OMS Rust CAL Implementation shall drop the oldest buffered message when a `MessageBuffer` limit is exceeded. | CAL-005444, CAL-005445, CAL-015746, CAL-016079 |
| CERT RS-001060 | An OMS Rust CAL API shall export the macro `update_message_header!` that sets `MessageHeader.Timestamp` of a CAL Message to the current UTC time. | CAL-005179 |

### 9.11 The Reader Class

Covered by §9.10.2.

### 9.12 The Writer Class

Covered by §9.10.3.

### 9.13 The AbstractServiceBusConnectionStatusListener Class

```rust
pub trait AsbStatusListener: Send + Sync {
    fn on_status_change(&self, status: &AsbStatus);
}
```

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-001070 | An OMS Rust CAL API shall define the trait `AsbStatusListener: Send + Sync` in the `asb` module declaring `fn on_status_change(&self, status: &AsbStatus)`. | CXX-011324, CXX-011341, CXX-011355 |
| CERT RS-001080 | An OMS Rust CAL Implementation shall invoke `on_status_change` on every registered listener on each ASB state change. | CXX-011355, CAL-016366 |

### 9.14 The Externalizer Class

Defined in module `rcal::externalizer`. An `Externalizer` is a message-type-agnostic byte transform with identity metadata. Chained externalizers are linked through `next()`. Typed serialization is done by free functions, which replace the C++ `read`/`write` overloads that take a `uci::base::Accessor&`.

```rust
pub trait Externalizer: Send + Sync {
    fn encode(&self, bytes: &[u8]) -> CalResult<Vec<u8>> { /* identity */ }
    fn decode(&self, bytes: &[u8]) -> CalResult<Vec<u8>> { /* identity */ }
    fn next(&self) -> Option<&dyn Externalizer> { None }
    fn serialization_format(&self) -> Option<SerializationFormat> { None }
    fn get_cal_api_version(&self) -> &str;
    fn get_encoding(&self) -> &str;
    fn get_schema_version(&self) -> &str { /* crate version */ }
    fn get_vendor_version(&self) -> &str { /* crate version */ }
    fn get_vendor(&self) -> &str { "rcal" }
    fn message_read_only(&self) -> bool { false }
    fn message_write_only(&self) -> bool { false }
    fn supports_object_read(&self) -> bool { true }
    fn supports_object_write(&self) -> bool { true }
}

pub fn write_to_bytes<M: CalMessage + Serialize>(ext: &dyn Externalizer, msg: &M, root: &str) -> CalResult<Vec<u8>>;
pub fn write_to_string<M: CalMessage + Serialize>(ext: &dyn Externalizer, msg: &M, root: &str) -> CalResult<String>;
pub fn write_to_writer<M: CalMessage + Serialize>(ext: &dyn Externalizer, msg: &M, root: &str, writer: &mut dyn Write) -> CalResult<()>;
pub fn read_from_bytes<M: CalMessage + DeserializeOwned>(ext: &dyn Externalizer, bytes: &[u8]) -> CalResult<M>;
pub fn read_from_str<M: CalMessage + DeserializeOwned>(ext: &dyn Externalizer, s: &str) -> CalResult<M>;
pub fn read_from_reader<M: CalMessage + DeserializeOwned>(ext: &dyn Externalizer, reader: &mut dyn Read) -> CalResult<M>;
```

**(informative)** Built-in externalizers: `XmlExternalizer` (`"xml"`), `PrettyExternalizer` (`"pretty"`, write-only tree dump), `CompressionExternalizer` (feature `compression`) and `ExiExternalizer` (feature `exi`, `"exi"`). They can be composed with `ExternalizerBuilder` / `externalizer::builder(name)` or from the CAL configuration with `build_externalizer(name, &config)`.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-001090 | An OMS Rust CAL API shall define the trait `Externalizer: Send + Sync` in the `externalizer` module with the methods and default behaviours in §9.14. | CXX-012071, CXX-012308, CXX-012321, CXX-012335, CXX-012703 |
| CERT RS-001100 | `Externalizer::get_cal_api_version`, `get_encoding`, `get_schema_version`, `get_vendor_version`, and `get_vendor` shall return the corresponding identity strings. | CXX-012238, CXX-012252, CXX-012266, CXX-012280, CXX-012294 |
| CERT RS-001110 | `Externalizer::message_read_only`, `message_write_only`, `supports_object_read`, and `supports_object_write` shall report the externalizer's capabilities. | CXX-012688, CXX-012689, CXX-012690, CXX-012691 |
| CERT RS-001120 | When encoding, the externalizer chain shall apply the head's `encode` first and then each `next()` in order. When decoding, the order shall be reversed. | CXX-012146, CXX-012099 |
| CERT RS-001130 | An OMS Rust CAL API shall define the free functions `write_to_bytes`, `write_to_string`, and `write_to_writer` in the `externalizer` module with the signatures in §9.14. | CXX-012146, CXX-012161, CXX-012176 |
| CERT RS-001140 | An OMS Rust CAL API shall define the free functions `read_from_bytes`, `read_from_str`, and `read_from_reader` in the `externalizer` module with the signatures in §9.14. | CXX-012099, CXX-012115, CXX-012131 |
| CERT RS-001150 | The read and write free functions shall return `Err` with kind `SerializationError` on any encoding, decoding, I/O, or format failure, including an attempt to read through a write-only externalizer. | CXX-012099, CXX-012146 |

### 9.15 The ExternalizerLoader Class

```rust
pub trait ExternalizerLoader: Send + Sync {
    fn get_externalizer(&self, encoding: &str, schema_version: &str, vendor_version: &str)
        -> CalResult<Box<dyn Externalizer>>;
}
pub fn get_externalizer_loader() -> Box<dyn ExternalizerLoader>;
```

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-001160 | An OMS Rust CAL API shall define the trait `ExternalizerLoader: Send + Sync` in the `externalizer` module with the method in §9.15. | CXX-012338, CXX-012395, CXX-012408, CXX-012422, CXX-012704 |
| CERT RS-001170 | `ExternalizerLoader::get_externalizer` shall return `Err` with kind `SerializationError` for an unsupported encoding. | CXX-012367 |
| CERT RS-001180 | An OMS Rust CAL API shall define the free function `get_externalizer_loader` in the `externalizer` module returning the default loader. | CXX-012434 |
| CERT RS-001190 | Externalizers and loaders shall be released when their `Box` is dropped. No destroy function shall be required. | CXX-012381, CXX-012446 |

---

## 10 Accessors

This chapter specifies the code the schema compiler generates into `rcal::uci::types`.

### 10.1 Generating Rust Data Type and Enumeration Item Names

See §5.2.

### 10.2 Complex Type Declarations

For each `xs:complexType` that is not a pure type alias, the compiler generates:

1. an **accessor trait** `<TypeName>`, which is the type CAL Clients program against;
2. a **concrete struct** `<TypeName>_` (`#[doc(hidden)]`) with private (`pub(super)`) fields, deriving `Debug, Clone, Serialize, Deserialize`, and also `Default` unless it is a message type;
3. `impl <TypeName> for <TypeName>_` and, for each ancestor in the extension chain, `impl <Ancestor> for <TypeName>_`;
4. an inherent `pub fn is_valid_at(&self, path: &str) -> Result<(), ValidationError>`;
5. `impl CalSubMessage for <TypeName>_` when the type is abstract or not backed by a global element.

`xs:extension` of a complexType becomes **supertrait inheritance**:

```rust
pub trait SystemStatusMT : crate::uci::types::MessageType {
    fn message_data(&self) -> &dyn crate::uci::types::SystemStatusMDT;
    fn message_data_mut(&mut self) -> &mut dyn crate::uci::types::SystemStatusMDT;
}
```

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-001200 | An OMS Rust CAL schema compiler shall generate, for each complexType, an accessor trait `<TypeName>` and a concrete struct `<TypeName>_` that implements it. | CXX-005456, CXX-007316 |
| CERT RS-001210 | An OMS Rust CAL schema compiler shall make the accessor trait of a complexType derived by `xs:extension` a subtrait of its base type's accessor trait, and the concrete struct shall implement every ancestor trait. | CXX-005456 |
| CERT RS-001220 | Generated concrete struct fields shall not be public outside the generated module. CAL Clients shall access fields only through the accessor trait. | CXX-005464, CXX-005465 |
| CERT RS-001230 | Each generated concrete struct shall implement `Debug`, `Clone`, `Serialize`, and `Deserialize`, and shall implement `Default` unless it is a CAL Message type. | CXX-005464, CXX-011063, CAL-016035 |
| CERT RS-001240 | Each generated concrete struct shall declare `is_valid_at(&self, path: &str) -> Result<(), ValidationError>`, which recursively validates every field (§10.2.4) and reports the first failure with a dot-separated path. | CXX-011057 |

#### 10.2.3 The Element Access Methods

For each child element `E` (snake name `e`, Rust type `T`):

| Element variant | Generated trait methods |
|---|---|
| Required, simple type `T` | `fn e(&self) -> &T;` `fn e_mut(&mut self) -> &mut T;` |
| Required, complex type | `fn e(&self) -> &dyn T;` `fn e_mut(&mut self) -> &mut dyn T;` |
| Optional (`minOccurs="0"`, `maxOccurs="1"`) | `fn e(&self) -> Option<&T>;` `fn e_mut(&mut self) -> Option<&mut T>;` `fn e_set(&mut self, value: T_);` `fn e_enable(&mut self);` `fn e_disable(&mut self);` (complex `T` returned as `&dyn T`) |
| Repeated (`maxOccurs > 1` or `unbounded`) | `fn e(&self) -> &[T_];` `fn e_mut(&mut self) -> &mut BoundedList<T_, MIN, MAX>;` |
| `uci:UUID` element | As above with `T = rcal::uci::base::UUID` |

`MIN`/`MAX` come from `minOccurs`/`maxOccurs`. `maxOccurs="unbounded"` uses the `usize::MAX` default.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-001250 | For each required child element, the accessor trait shall declare a shared getter `e(&self)` and a mutable getter `e_mut(&mut self)`. | CXX-011213, CXX-011214, CXX-011215, CXX-011216, CXX-011217, CXX-012706, CXX-012707, CXX-012708 |
| CERT RS-001260 | Getters for complexType-typed elements shall return `&dyn <Trait>` / `&mut dyn <Trait>`. | CXX-011213, CXX-011214 |
| CERT RS-001270 | For each optional child element, the accessor trait shall declare `e(&self) -> Option<&T>`, `e_mut(&mut self) -> Option<&mut T>`, `e_set(&mut self, value)`, `e_enable(&mut self)` (sets the default value), and `e_disable(&mut self)` (removes the element). | CAL-005290, CXX-011229, CXX-011230, CXX-011231, CXX-012709 |
| CERT RS-001280 | For each repeated child element, the accessor trait shall declare `e(&self) -> &[T]` and `e_mut(&mut self) -> &mut BoundedList<T, MIN, MAX>`, with `MIN` and `MAX` taken from the schema occurrence constraints. | CXX-011226, CXX-011227, CXX-011228, CXX-005224, CXX-005233 |
| CERT RS-001290 | Elements of type `uci:UUID` shall be typed as `rcal::uci::base::UUID`. | CXX-006154, CXX-011054 |
| CERT RS-001300 | Optional elements shall be omitted from serialized output when absent, and repeated elements shall be omitted when empty. | CAL-005290 |

#### 10.2.4 Validation rules

`is_valid_at` checks, in field order:

- repeated elements: `BoundedList::is_valid_at`, then each item;
- enumeration elements: not `EnumNotSet` (§10.6.3);
- complex elements: recursive `is_valid_at`;
- `xs:string`-based elements with facets: `length`/`minLength`/`maxLength` (character count) and `pattern` (compiled once with `regex::Regex`);
- `xs:double`/`xs:float`-based elements with `minInclusive`/`maxInclusive`.

Optional elements are only checked when present.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-001310 | Generated validation shall reject string values whose character count violates `length`, `minLength`, or `maxLength`, or that do not match `pattern`. | CXX-011057 |
| CERT RS-001320 | Generated validation shall reject floating-point values outside `minInclusive`/`maxInclusive`. | CXX-011057 |

### 10.3 Global Element Accessors

For each global `xs:element` whose type is a concrete message type, the compiler generates a **message wrapper**:

```rust
#[doc(hidden)]
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(transparent)]
pub struct SystemStatus_(pub crate::uci::types::SystemStatusMT_);
// impl Serialize (adds xmlns, xmlns:xsi, xmlns:uci, xmlns:xs attributes)
// impl Deref/DerefMut<Target = SystemStatusMT_>
impl crate::uci::CalMessage for SystemStatus_ {
    fn message_type_name() -> crate::QName { /* {ns}SystemStatusMT */ }
    fn cal_create() -> Self { /* sealed */ }
    fn is_valid(&self) -> Result<(), ValidationError> { self.0.is_valid_at("SystemStatus") }
    fn as_message_type_mut(&mut self) -> Option<&mut dyn MessageType> { Some(&mut self.0) }
}
```

Readers and writers for a message are obtained generically through `AbstractCal::create_reader::<SystemStatus_>` / `create_writer::<SystemStatus_>` (§9.10). No per-message `createReader`/`createWriter` is generated.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-001330 | An OMS Rust CAL schema compiler shall generate, for each global element of a concrete message type, a wrapper struct `<ElementName>_` that implements `CalMessage` and dereferences (shared and mutable) to the element type's concrete struct. | CXX-005456, CXX-005494, CXX-005499 |
| CERT RS-001340 | The generated `CalMessage::is_valid` shall validate the whole message, using the element name as the root of the error path. | CXX-011057 |
| CERT RS-001350 | The generated wrapper shall serialize the target, `xsi`, `uci`, and `xs` namespace declarations on the root element. | CAL-005179 |
| CERT RS-001360 | For message types derived from `MessageType`, the generated `as_message_type_mut` shall return `Some`. | CXX-011066 |

### 10.4 Accessor Specification

The table in §10.2.3 is the accessor specification.

### 10.5 The Primitive Accessors

Primitive-typed elements use the `xs` aliases directly (§9.1). No `<Primitive>Accessor` wrapper classes are generated.

**(informative)** The current schema compiler does not map `xs:base64Binary`, even though `xs::Base64Binary` is defined. Elements of that type are not supported until the mapping is added.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-001370 | An OMS Rust CAL schema compiler shall map each XSD primitive in §9.1, except `xs:base64Binary`, to the corresponding `rcal::xs` type. | CXX-005765, CXX-005775, CXX-011087, CXX-011088, CXX-011089, CXX-013023 |

### 10.6 The Simple Accessors

#### 10.6.1 The SimpleRestriction Accessor

A simpleType that restricts a primitive without enumeration facets becomes a type alias of the base Rust type:

```rust
/// XSD simpleType `VisibleString32Type`.
pub type VisibleString32Type = crate::xs::XsString;
```

The facets are enforced by `is_valid_at` of the containing type (§10.2.4).

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-001380 | An OMS Rust CAL schema compiler shall generate, for each non-enumerated simpleType restriction, a `pub type <TypeName>` alias equivalent to its resolved base Rust type. | CXX-006553, CXX-006143, CXX-006144 |

#### 10.6.2 The Universally Unique Identifier Accessor

See RS-001290.

#### 10.6.3 The Enumeration Accessor

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub enum ClassificationEnum {
    #[default] #[serde(rename = "enumNotSet")] EnumNotSet,
    #[serde(rename = "R")]  R,
    #[serde(rename = "TS")] Ts,
    // …
}
impl ClassificationEnum {
    pub fn is_valid_at(&self, path: &str) -> Result<(), ValidationError>;  // Err if EnumNotSet
    pub fn as_str(&self) -> Option<&'static str>;                          // XSD value; None if EnumNotSet
}
// impl Deserialize (accepts bare text or {"$text": …}; unknown values are errors)
```

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-001390 | An OMS Rust CAL schema compiler shall generate, for each enumerated simpleType, a Rust `enum <TypeName>` with one variant per `xs:enumeration` value, plus a default variant `EnumNotSet`. | CXX-006166, CXX-006172, CXX-006173 |
| CERT RS-001400 | Generated enumerations shall implement `Debug`, `Clone`, `Default` (= `EnumNotSet`), `PartialEq`, `Eq`, `Serialize`, and `Deserialize`, serializing each variant as its XSD literal. | CXX-006211, CXX-006267, CXX-006275, CXX-006315, CXX-006323, CXX-006332, CXX-006340, CXX-011062 |
| CERT RS-001410 | Generated enumerations shall declare `as_str(&self) -> Option<&'static str>` returning the XSD literal, or `None` for `EnumNotSet`. | CXX-006417, CXX-011059, CXX-006457 |
| CERT RS-001420 | Generated enumerations shall reject unknown literals during deserialization. | CXX-006253, CXX-006260, CXX-006424, CXX-011060 |
| CERT RS-001430 | Generated enumerations shall declare `is_valid_at(&self, path)` returning `Err` when the value is `EnumNotSet`. | CXX-011057, CXX-011149 |

### 10.7 The Complex Accessors

#### 10.7.1 The LocalElement Accessor

Local elements are realized as fields with the access methods of §10.2.3.

#### 10.7.2 The Choice Accessor

An `xs:choice` complexType becomes an untagged Rust enum with one struct variant per alternative:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum OwnerProducerChoiceType_ {
    GovernmentIdentifier { #[serde(rename = "GovernmentIdentifier")] inner: OwnerProducerEnum },
    NATO_SpecialWord     { #[serde(rename = "NATO_SpecialWord")]     inner: crate::xs::XsString },
}
impl Default for OwnerProducerChoiceType_ { /* first alternative, default value */ }
impl OwnerProducerChoiceType_ { pub fn is_valid_at(&self, path: &str) -> Result<(), ValidationError>; }
impl crate::uci::CalSubMessage for OwnerProducerChoiceType_ {}
```

Selecting an alternative is done by constructing a variant. Querying it is done with `match`/`matches!`. These replace `get…ChoiceOrdinal`, `set…ChoiceOrdinal`, `is<E>` and `choose<E>`.

| ID | Requirement | Derived from |
|---|---|---|
| CERT RS-001440 | An OMS Rust CAL schema compiler shall generate, for each `xs:choice` complexType, an `enum <TypeName>_` with one variant per alternative, named after the alternative element and carrying its value in a field `inner`. | CXX-007137, CXX-007138, CXX-011109, CXX-011110, CXX-012695, CXX-012696, CXX-012714, CXX-012715 |
| CERT RS-001450 | Generated choice enums shall implement `Debug`, `Clone`, `Serialize`, `Deserialize`, `Default` (the first alternative with its default value), and `CalSubMessage`, and shall declare `is_valid_at` validating the selected alternative. | CXX-007137, CXX-011057 |

#### 10.7.3 The GlobalElement Accessor

See §10.3.

### 10.8 Extension Traits (informative)

Hand-written traits can add convenience methods to generated types, e.g. `uci::types::SecurityInformationExt::security_banner(&self) -> String` for `SecurityInformationType`.

---

## 11 Rust CAL CERT Verification

| Method | Applies to | Location |
|---|---|---|
| **Inspection**: the API compiles with the stated signatures | All "shall define / shall declare" requirements | Public API, `cargo doc` |
| **Unit test** | Behavioural requirements on hand-written types (`UUID`, `BoundedList`, `xs::DateTime`, `AsbConnectionState`, `QName`, externalizers, configuration) | `#[cfg(test)] mod tests` in `src/**` |
| **Integration test** | Reader/writer, listener, QoS, status, and transport behaviour | `tests/dds_asb_system.rs`, `tests/zmq_*.rs`, `tests/activemq_*.rs` |
| **Generated-code test** | Chapter 10 requirements | Compilation of `uci::types` plus round-trip and validation tests in the integration suites and examples |

Requirements stated for an *OMS Rust CAL Implementation* are verified per transport.

---

## 12 List of Symbols, Abbreviations, and Acronyms

| Term | Meaning |
|---|---|
| ASB | Abstract Service Bus |
| CAL | Critical Abstraction Layer |
| CERT | Certification requirement |
| DDS | Data Distribution Service |
| EXI | Efficient XML Interchange |
| OMS | Open Mission Systems |
| QoS | Quality of Service |
| RS | Rust (requirement prefix) |
| UCI | Universal Command and Control Interface |
| UUID | Universally Unique Identifier |
| XSD | XML Schema Definition |

## 13 Glossary

| Term | Definition |
|---|---|
| Accessor trait | Generated trait through which a CAL Client reads and writes a schema type. |
| CAL Client | Application code that uses the OMS Rust CAL API. |
| CAL Implementation | Code that provides the behaviour behind the API (rcal and its transports). |
| CAL Message | A type implementing `CalMessage`: a concrete, publishable global element. |
| Concrete struct | Generated `<TypeName>_` storage type that implements the accessor trait. |
| Schema compiler | `build.rs`, which generates `rcal::uci::types` from the XSD. |
| Sub-message | A type implementing `CalSubMessage`: nested data that cannot be published alone. |

---

## Appendix A — Deviations from OMSC-SPC-008

| OMSC-SPC-008 item | Rust realization | Rationale |
|---|---|---|
| `UCIException` + `throwUciException` macro (§9.2) | `CalError` / `CalResult`, built with `CalError::new` | Rust uses `Result`, not exceptions |
| `UCIException::ErrorCode` (`uint32_t`) | `CalErrorKind` enum | Typed error classification |
| `accessorType` constants (§9.3) | Not provided | Static typing and `TypeId` cover this |
| `Accessor` base class and `reset()` (§9.4) | `CalMessage` / `CalSubMessage` marker traits; reassign `Default::default()` | No inheritance of data in Rust |
| `StringAccessor` (§9.5) | `String` | Native owned string |
| `SimpleList` (§9.6) | `BoundedList` / `Vec` | One list type is enough |
| `BoundedList` method set (§9.7) | `Deref<Target = Vec<T>>` + `resize`, `get_*_occurs` | Reuse std API |
| Bounds enforced on `push_back` | Bounds checked by `is_valid_at` before write | Allows incremental construction |
| `UUID::ValueUUID`, `getValueUUID`, `UUID(uint64_t, uint64_t)`, `getClockSequence`, `getTimestamp`, `getNodeAddress`, `getNamespaceUUID`, `createVersion3UUID(name)`, `toString(Variant/Version)` | Not provided. Use `as_raw()` for the underlying `uuid::Uuid` API | Covered by the `uuid` crate |
| Default/copy constructors, `operator=`, destructors disabled | Private fields, sealed `Token`, `Clone`, `Drop`, `close(self: Box<Self>)` | Ownership model |
| `AbstractServiceBusConnection::shutdown` | `AbstractServiceBus::close` | Rust naming |
| `uci_getAbstractServiceBusConnection` | `cal::get_cal` / `cal::create_cal` (`async`) | Async transports |
| Per-type `Reader`/`Writer`/`Listener` classes, `createReader`/`destroyReader`/`createWriter`/`destroyWriter` (§10.3.4) | Generic `AbstractReader<M>`, `AbstractWriter<M>`, `MessageListener<M>` from `AbstractCal` | Generics replace per-type classes |
| `read(maxMessages, timeout, out)` returning a count | `read(timeout) -> CalResult<Option<Arc<M>>>` (one message per call) | Simpler ownership |
| `<Type>::create` / `destroy` | `AbstractCalCreateMessage::create_message`; `Drop` | Sealed construction |
| `getUCITypeVersion` | `AbstractServiceBus::oms_schema_version` | Single schema per build |
| `Externalizer::read`/`write` overloads on `Accessor&` | `externalizer::read_from_*` / `write_to_*` generic free functions | Keeps `Externalizer` object-safe |
| `destroyExternalizer`, `uci_destroyExternalizerLoader` | `Drop` | Ownership model |
| Enumeration `EnumerationItem`, `getNumberOfItems`, ordering operators, `toName`/`fromName`/`setValueFromName`, `operator<<` | Rust `enum`, derived `PartialEq`, `as_str`, serde | Native enums |
| Choice `…ChoiceOrdinal`, `is<E>`, `choose<E>` | Rust `enum` variants with `match` | Native sum types |
| `has<E>`/`enable<E>`/`clear<E>` | `e() -> Option<_>`, `e_enable`, `e_set`, `e_disable` | `Option` |
