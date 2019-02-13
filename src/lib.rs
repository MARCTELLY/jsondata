// Copyright (c) 2018 R Pratap Chakravarthy and AUTHORS

//! Jsondata is yet another [JSON] implementation in Rust, but
//! optimized for big-data and document-databases that store
//! documents in [JSON] format. Following is the scope defined
//! for this package:
//!
//! * Support for 128-bit signed integers.
//! * Deferred conversion of numbers.
//! * Serialization from Rust native type to JSON text.
//! * De-serialization from JSON text to Rust native type.
//! * [CRUD] operation on JSON documents, using [JSON Pointer].
//! * Sorted keys in property object.
//! * Streaming JSON parser.
//! * Support JSON5 standard.
//! * Common arithmetic and logic operations.
//! * Sortable JSON.
//!
//! To parse JSON text, use [parse]:
//!
//! ```
//! let text = r#"[null,true,false,10,"true"]"#;
//! let json = text.parse::<jsondata::Json>().unwrap();
//! ```
//!
//! To serialise [Json] type to JSON text:
//!
//! ```
//! let text = r#"[null,true,false,10,"true"]"#;
//! let json = text.parse::<jsondata::Json>().unwrap();
//!
//! let text1 = json.to_string();    // one way to serialize to JSON
//! let text2 = format!("{}", json); // another way to serialize to JSON
//! assert_eq!(text1, text2);
//! ```
//!
//! When parsing a JSON text to [Json] instance, numbers are not parsed
//! right away, hence calls to [integer] and [float] methods will have
//! to compute the value every time,
//!
//! ```
//! let mut json = "1000".parse::<jsondata::Json>().unwrap();
//! json.integer().unwrap(); // "1000" is parsed
//! json.integer().unwrap(); // "1000" is parsed again
//!
//! match json.compute() { // pre-compute all numbers in the json document.
//!     Ok(_) => (),
//!     Err(s) => println!("{}", s),
//! }
//! ```
//!
//! If JSON text is going to come from untrusted parties,
//!
//! ```
//! let mut json = r#"{"a": 1000}"#.parse::<jsondata::Json>().unwrap();
//! match json.validate() { // validate
//!     Ok(_) => (),
//!     Err(s) => println!("{}", s),
//! }
//! ```
//!
//! [JSON]: https://tools.ietf.org/html/rfc8259
//! [CRUD]: https://en.wikipedia.org/wiki/Create,_read,_update_and_delete
//! [JSON Pointer]: https://tools.ietf.org/html/rfc6901
//! [parse]: str::method.parse
//! [integer]: enum.Json.html#method.integer
//! [float]: enum.Json.html#method.float

#![doc(html_favicon_url = "https://cdn4.iconfinder.com/data/icons/fugue/icon_shadowless/json.png")]
#![doc(
    html_logo_url = "https://upload.wikimedia.org/wikipedia/commons/thumb/c/c9/JSON_vector_logo.svg/1024px-JSON_vector_logo.svg.png"
)]

mod json;
mod jsons;
mod lex;
mod num;
mod ops;
mod parse;
mod property;

pub mod jptr;

// Re-exports for API documentation.
pub use crate::json::Json;
pub use crate::jsons::Jsons;
pub use crate::property::Property;

#[cfg(test)]
mod jptr_test;
#[cfg(test)]
mod json_test;
#[cfg(test)]
mod jsons_test;
#[cfg(test)]
mod ops_test;
