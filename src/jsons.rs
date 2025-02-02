// Copyright © 2019 R Pratap Chakravarthy. All rights reserved.

use std::io;

use unicode_reader::CodePoints;

use crate::{json::Json, Error, Result};

/// Jsons can parse a stream of JSON text supplied by any [Read] instance.
/// For Example:
///
/// ```
/// let file = std::fs::File::open("testdata/stream1.jsons").unwrap();
/// let mut iter: jsondata::Jsons<std::fs::File> = file.into();
///
/// for json in iter {
///     println!("{:?}", json)
/// }
/// ```
///
/// Note that the iterated value is of type ``Result<Json, Error>``,
/// where errors can be handled in following manner :
///
/// ```ignore
/// for json in iter {
///     match json {
///         Ok(value) if value.integer() > 100 => {
///             /* handle Json value*/
///         },
///         Ok(value) if value.is_error() => {
///             /* value.error() to fetch the error String */
///         },
///         Err(err) => {
///             /* handle error returned by the Read instance */
///         },
///     }
/// }
/// ```
///
/// [Read]: std::io::Read
pub struct Jsons<R>
where
    R: io::Read,
{
    codes: CodePoints<io::Bytes<R>>,
    quant: String,
}

impl<R> From<R> for Jsons<R>
where
    R: io::Read,
{
    fn from(input: R) -> Jsons<R> {
        Jsons {
            codes: input.into(),
            quant: String::with_capacity(1024),
        }
    }
}

impl<R> Iterator for Jsons<R>
where
    R: io::Read,
{
    type Item = Result<Json>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut markers = String::new();
        let mut ok_ch = self.read_whitespace()?;
        loop {
            let ch = match ok_ch {
                Ok(ch) => {
                    //println!("{}", ch);
                    self.quant.push(ch);
                    match ch {
                        '{' => markers.push('}'),
                        '[' => markers.push(']'),
                        '}' | ']' => loop {
                            if let Some(m) = markers.pop() {
                                if m == ch {
                                    break;
                                }
                            } else if markers.is_empty() {
                                break;
                            }
                        },
                        '"' => match Jsons::read_string(self)? {
                            Ok(_) => (),
                            Err(err) => break Some(Err(err)),
                        },
                        _ => (),
                    }
                    //println!("loop {:?} {}", self.quant.as_bytes(), ch);
                    ch
                }
                Err(err) => break Some(Err(err)),
            };
            let eov = ch.is_whitespace() || ch == '}' || ch == ']' || ch == '"';
            if markers.is_empty() && eov {
                let res = match self.quant.parse() {
                    Ok(json) => Some(Ok(json)),
                    Err(s) => Some(Ok(Json::__Error(s))),
                };
                //println!("quant {:?} {:?}", self.quant.as_bytes(), res);
                self.quant.truncate(0);
                break res;
            }
            ok_ch = match self.codes.next() {
                Some(res) => err_at!(IoError, res),
                None if !self.quant.is_empty() => {
                    let res = match self.quant.parse() {
                        Ok(json) => Some(Ok(json)),
                        Err(s) => Some(Ok(Json::__Error(s))),
                    };
                    //println!("quant {:?} {:?}", self.quant.as_bytes(), res);
                    self.quant.truncate(0);
                    break res;
                }
                None => break None,
            }
        }
    }
}

impl<R> Jsons<R>
where
    R: io::Read,
{
    fn read_string(&mut self) -> Option<Result<()>> {
        let mut escape = false;
        loop {
            match self.codes.next() {
                Some(Ok(ch)) if escape => {
                    self.quant.push(ch);
                    escape = false;
                }
                Some(Ok('\\')) => {
                    self.quant.push('\\');
                    escape = true;
                }
                Some(Ok('"')) => {
                    self.quant.push('"');
                    break Some(Ok(()));
                }
                Some(Ok(ch)) => self.quant.push(ch),
                Some(Err(err)) => break Some(err_at!(IoError, msg: "{}", err)),
                None => break Some(Ok(())),
            }
        }
    }

    fn read_whitespace(&mut self) -> Option<Result<char>> {
        loop {
            match self.codes.next()? {
                Ok(ch) if !ch.is_whitespace() => break Some(Ok(ch)),
                Ok(_) => (),
                res => break Some(err_at!(IoError, res)),
            }
        }
    }
}

#[cfg(test)]
#[path = "jsons_test.rs"]
mod jsons_test;
