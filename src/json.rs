use std::convert::From;
use std::default::Default;
use std::fmt::{self, Display, Write};
use std::io;
use std::str::FromStr;
use std::cmp::{PartialOrd, Ordering};
use unicode_reader::CodePoints;

use jptr;
use lex::Lex;
use num::{Floating, Integral};
use parse::parse_value;
use property::{self, Property};

/// Json type implements JavaScript Object Notation as per specification
/// [RFC-8259](https://tools.ietf.org/html/rfc8259).
///
/// * Numbers are implemented with deferred conversion, using
///   ``Integral`` and ``Floating`` types.
/// * Arrays are implemented as vector of Json values Vec<[Json]>.
/// * Objects are implemented as vector of properties, Vec<[Property]>,
///   where each property is a tuple of (key, value). Here key is [String]
///   type and value is [Json] type.
///
/// [string]: std::string::String
#[derive(Clone, Debug)]
pub enum Json {
    Null,
    Bool(bool),
    Integer(Integral),
    Float(Floating),
    String(String),
    Array(Vec<Json>),
    Object(Vec<Property>),
    // Hidden variants
    #[doc(hidden)]
    __Error(String),
}

/// Implementation provides methods to construct and validate Json values.
impl Json {
    /// Construct [Json] from [bool], [i128], [f64], [String], [str],
    /// [Vec].
    ///
    /// Array can be composed as:
    ///
    /// ```
    /// extern crate jsondata;
    /// use jsondata::Json;
    ///
    /// let mut js = Json::new::<Vec<Json>>(Vec::new());
    /// js.append("", Json::new(10));
    /// js.append("", Json::new("hello world".to_string()));
    /// ```
    ///
    /// It is also possbile to construct the vector of Json outside
    /// the append() method, and finally use Json::new() to construct
    /// the array.
    ///
    /// Object can be composed as:
    ///
    /// ```
    /// extern crate jsondata;
    /// use jsondata::{Json, Property};
    ///
    /// let mut js = Json::new::<Vec<Property>>(Vec::new());
    /// js.set("/key1", Json::new(10));
    /// js.set("/key2", Json::new(true));
    /// ```
    ///
    /// It is also possbile to construct the vector of properties outside
    /// the set() method, and finally use Json::new() to construct
    /// the object.
    pub fn new<T>(value: T) -> Json
    where
        Self: From<T>,
    {
        value.into()
    }

    /// Validate parts of JSON text that are not yet parsed. Typically,
    /// when used in database context, JSON documents are validated once
    /// but parsed multiple times.
    pub fn validate(&mut self) -> Result<(), String> {
        use json::Json::{Array, Float, Integer, Object};

        match self {
            Array(items) => {
                for item in items.iter_mut() {
                    item.validate()?
                }
            }
            Object(props) => {
                for prop in props.iter_mut() {
                    prop.value_mut().validate()?
                }
            }
            Integer(item) => {
                item.compute()?;
            }
            Float(item) => {
                item.compute()?;
            }
            _ => (),
        };
        Ok(())
    }

    /// Compute parses unparsed text and convert them into numbers.
    /// When a JSON document is parsed once but operated on multiple
    /// times it is better to call compute for better performance.
    ///
    /// ```
    /// extern crate jsondata;
    /// use jsondata::Json;
    ///
    /// let text = r#"[null,true,false,10,"true"]"#;
    /// let mut json: Json = text.parse().unwrap();
    /// json.compute();
    ///
    /// // perform lookup and arithmetic operations on parsed document.
    /// ```
    pub fn compute(&mut self) -> Result<(), String> {
        use json::Json::{Array, Float, Integer, Object};

        match self {
            Array(items) => {
                for item in items.iter_mut() {
                    item.compute()?
                }
            }
            Object(props) => {
                for prop in props.iter_mut() {
                    prop.value_mut().compute()?
                }
            }
            Integer(item) => {
                item.compute()?;
            }
            Float(item) => {
                item.compute()?;
            }
            _ => (),
        };
        Ok(())
    }

    fn typename(&self) -> String {
        match self {
            Json::Null => "null".to_string(),
            Json::Bool(_) => "bool".to_string(),
            Json::Integer(_) => "integer".to_string(),
            Json::Float(_) => "float".to_string(),
            Json::String(_) => "string".to_string(),
            Json::Array(_) => "array".to_string(),
            Json::Object(_) => "object".to_string(),
            Json::__Error(_) => "error".to_string(),
        }
    }
}

/// Implementation provides CRUD access into [Json] document using
/// [Json Pointer]. For all methods,
///
/// * Path must be valid JSON Pointer.
/// * Path fragment must be valid key if parent container is an object.
/// * Path fragment must be a number index if parent container is an array.
///
/// [JSON Pointer]: https://tools.ietf.org/html/rfc6901
impl Json {
    /// get a json field, within the document, locatable by ``path``.
    pub fn get(&self, path: &str) -> Result<Json, String> {
        if path.is_empty() {
            Ok(self.clone())
        } else {
            let path = jptr::fix_prefix(path)?;
            let (json, frag) = jptr::g_lookup(self, path)?;
            let json = jptr::g_lookup_container(json, &frag)?;
            Ok(json.clone())
        }
    }

    /// set a json field, within the document, locatable by ``path``.
    pub fn set(&mut self, path: &str, value: Json) -> Result<(), String> {
        if path.is_empty() {
            return Ok(());
        }

        let path = jptr::fix_prefix(path)?;

        let (json, frag) = jptr::lookup(self, path)?;
        match json {
            Json::Array(arr) => match frag.parse::<usize>() {
                Ok(n) if n >= arr.len() => Err(format!("jptr: index out of bound {}", n)),
                Ok(n) => {
                    arr[n] = value;
                    Ok(())
                }
                Err(err) => Err(format!("jptr: not array-index {}", err)),
            },
            Json::Object(props) => match property::search_by_key(&props, &frag) {
                Ok(n) => {
                    props[n].set_value(value);
                    Ok(())
                }
                Err(n) => {
                    props.insert(n, Property::new(frag, value));
                    Ok(())
                }
            },
            _ => Err(format!("jptr: not a container {} at {}", json, frag)),
        }
    }

    /// delete a json field, within the document, locatable by ``path``.
    pub fn delete(&mut self, path: &str) -> Result<(), String> {
        if path.is_empty() {
            return Ok(());
        }

        let path = jptr::fix_prefix(path)?;

        let (json, frag) = jptr::lookup(self, path)?;
        match json {
            Json::Array(arr) => match frag.parse::<usize>() {
                Ok(n) if n >= arr.len() => Err(format!("jptr: index out of bound {}", n)),
                Ok(n) => {
                    arr.remove(n);
                    Ok(())
                }
                Err(err) => Err(format!("jptr: not array-index {}", err)),
            },
            Json::Object(props) => match property::search_by_key(&props, &frag) {
                Ok(n) => {
                    props.remove(n);
                    Ok(())
                }
                Err(_) => Err(format!("jptr: key {} not found", frag)),
            },
            _ => Err(format!("{} not a container type", json.typename())),
        }
    }

    /// append a string or array to a json field within the document that is
    /// either a string or array.
    pub fn append(&mut self, path: &str, value: Json) -> Result<(), String> {
        if path.is_empty() {
            return Ok(());
        }
        let path = jptr::fix_prefix(path)?;

        let (json, frag) = jptr::lookup(self, path)?;
        let json = jptr::lookup_container(json, &frag)?;
        match json {
            Json::String(s1) => {
                if let Json::String(s2) = value {
                    let mut s = String::new();
                    s.push_str(&s1);
                    s.push_str(&s2);
                    Ok(())
                } else {
                    let tn = value.typename();
                    Err(format!("jptr: cannot add {} to `{}`", tn, s1))
                }
            }
            Json::Array(arr) => {
                let n = arr.len();
                arr.insert(n, value);
                Ok(())
            }
            _ => Err(format!("jptr: not a container {} at {}", json, frag)),
        }
    }
}

/// Implementation clones underlying type for each Json variant.
/// The return value is always an [Option] because JSON
/// follows a schemaless data representation.
impl Json {
    pub fn boolean(&self) -> Option<bool> {
        match self {
            Json::Bool(s) => Some(*s),
            _ => None,
        }
    }

    pub fn integer(&self) -> Option<i128> {
        match self {
            Json::Integer(item) => item.integer(),
            _ => None,
        }
    }

    pub fn float(&self) -> Option<f64> {
        match self {
            Json::Float(item) => item.float(),
            _ => None,
        }
    }

    pub fn string(&self) -> Option<String> {
        match self {
            Json::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn array(&self) -> Option<Vec<Json>> {
        match self {
            Json::Array(arr) => Some(arr.clone()),
            _ => None,
        }
    }

    pub fn object(&self) -> Option<Vec<Property>> {
        match self {
            Json::Object(obj) => Some(obj.clone()),
            _ => None,
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Json::__Error(_) => true,
            _ => false,
        }
    }

    pub fn error(&self) -> Option<String> {
        match self {
            Json::__Error(err) => Some(err.clone()),
            _ => None,
        }
    }
}

impl PartialEq for Json {
    fn eq(&self, other: &Json) -> bool {
        use Json::{Array, Bool, Float, Integer, Null, Object, String as S};

        match (self, other) {
            (Null, Null) => true,
            (Bool(a), Bool(b)) => a == b,
            (Integer(_), Integer(_)) => self.integer() == other.integer(),
            (Integer(a), Float(b)) => match (a.integer(), b.float()) {
                    (Some(x), Some(y)) => {
                        let n = y as i128;
                        if n == std::i128::MIN || n == std::i128::MAX || y.is_nan() {
                            return false
                        }
                        x == n
                    },
                    _ => false,
            },
            (Float(_), Float(_)) => {
                let (fs, fo) = (self.float().unwrap(), other.float().unwrap());
                if fs.is_finite() && fo.is_finite() {
                    return fs == fo
                } else if fs.is_nan() && fo.is_nan() {
                    return true
                } else if fs.is_infinite() && fo.is_infinite() {
                    return fs.is_sign_positive() == fo.is_sign_positive()
                }
                false
            },
            (Float(a), Integer(b)) => match (a.float(), b.integer()) {
                    (Some(x), Some(y)) => {
                        let n = x as i128;
                        if n == std::i128::MIN || n == std::i128::MAX || x.is_nan() {
                            return false
                        }
                        y == n
                    },
                    _ => false,
                },
            (S(a), S(b)) => a == b,
            (Array(a), Array(b)) => a == b,
            (Object(a), Object(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Json {
    fn partial_cmp(&self, other: &Json) -> Option<Ordering> {
        use Json::{Array, Bool, Float, Integer, Null, Object, String as S};

        match (self, other) {
            // typically we assume that value at same position is same type.
            (Null, Null) => Some(Ordering::Equal),
            (Bool(a), Bool(b)) =>
                if (*a) == (*b) {
                    Some(Ordering::Equal)
                } else if !(*a) {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                },
            (Integer(a), Integer(b)) => a.partial_cmp(b),
            (Float(a), Float(b)) => a.partial_cmp(b),
            (Integer(a), Float(b)) => match (a.integer(), b.float()) {
                (Some(x), Some(y)) => x.partial_cmp(&(y as i128)),
                (Some(_), None) => Some(Ordering::Greater),
                (None, Some(_)) => Some(Ordering::Less),
                (None, None) => Some(Ordering::Equal),
            },
            (Float(a), Integer(b)) => match (a.float(), b.integer()) {
                (Some(x), Some(y)) => (x as i128).partial_cmp(&y),
                (Some(_), None) => Some(Ordering::Greater),
                (None, Some(_)) => Some(Ordering::Less),
                (None, None) => Some(Ordering::Equal),
            },
            (S(a), S(b)) => a.partial_cmp(b),
            (Array(this), Array(that)) => {
                for (i, a) in this.iter().enumerate() {
                    if i == that.len() {
                        return Some(Ordering::Greater)
                    }
                    let cmp = a.partial_cmp(&that[i]);
                    if cmp != Some(Ordering::Equal) {
                        return cmp
                    }
                }
                if this.len() == that.len() {
                    Some(Ordering::Equal)
                } else {
                    Some(Ordering::Less)
                }
            },
            (Object(this), Object(that)) => {
                for (i, a) in this.iter().enumerate() {
                    if i == that.len() {
                        return Some(Ordering::Greater)
                    }
                    let cmp = a.key_ref().partial_cmp(that[i].key_ref());
                    if cmp != Some(Ordering::Equal) {
                        return cmp
                    }
                    let cmp = a.value_ref().partial_cmp(that[i].value_ref());
                    if cmp != Some(Ordering::Equal) {
                        return cmp
                    }
                }
                if this.len() == that.len() {
                    Some(Ordering::Equal)
                } else {
                    Some(Ordering::Less)
                }
            },
            // handle error cases, error variants sort at the end.
            (_, Json::__Error(_)) => Some(Ordering::Less),
            (Json::__Error(_), _) => Some(Ordering::Greater),
            // handle cases of mixed types.
            (Null, _) => Some(Ordering::Less),
            (_, Null) => Some(Ordering::Greater),
            (Bool(_), _) => Some(Ordering::Less),
            (_, Bool(_)) => Some(Ordering::Greater),
            (Integer(_), _) => Some(Ordering::Less),
            (_, Integer(_)) => Some(Ordering::Greater),
            (Float(_), _) => Some(Ordering::Less),
            (_, Float(_)) => Some(Ordering::Greater),
            (S(_), _) => Some(Ordering::Less),
            (_, S(_)) => Some(Ordering::Greater),
            (Array(_), _) => Some(Ordering::Less),
            (_, Array(_)) => Some(Ordering::Greater),
        }
    }
}

impl Default for Json {
    fn default() -> Json {
        Json::Null
    }
}

impl From<bool> for Json {
    fn from(val: bool) -> Json {
        Json::Bool(val)
    }
}

impl From<i128> for Json {
    fn from(val: i128) -> Json {
        Json::Integer(Integral::new(val))
    }
}

impl From<f64> for Json {
    fn from(val: f64) -> Json {
        Json::Float(Floating::new(val))
    }
}

impl From<String> for Json {
    fn from(val: String) -> Json {
        Json::String(val)
    }
}

impl From<&str> for Json {
    fn from(val: &str) -> Json {
        Json::String(val.to_string())
    }
}

impl From<Vec<Json>> for Json {
    fn from(val: Vec<Json>) -> Json {
        Json::Array(val)
    }
}

impl From<Vec<Property>> for Json {
    fn from(val: Vec<Property>) -> Json {
        let mut obj = Json::Object(vec![]);
        val.into_iter().for_each(|item| insert(&mut obj, item));
        obj
    }
}

impl From<Json> for bool {
    fn from(val: Json) -> bool {
        match val {
            Json::Null | Json::Bool(false) => false,
            _ => true,
        }
    }
}

impl FromStr for Json {
    type Err = String;

    fn from_str(text: &str) -> Result<Json, String> {
        let mut lex = Lex::new(0, 1, 1);
        parse_value(&text, &mut lex)
    }
}

impl AsRef<str> for Json {
    fn as_ref(&self) -> &str {
        match self {
            Json::String(s) => s,
            _ => panic!("Json is not string"),
        }
    }
}

impl AsRef<Vec<Json>> for Json {
    fn as_ref(&self) -> &Vec<Json> {
        match self {
            Json::Array(arr) => arr,
            _ => panic!("Json is not an array"),
        }
    }
}

impl AsRef<Vec<Property>> for Json {
    fn as_ref(&self) -> &Vec<Property> {
        match self {
            Json::Object(obj) => obj,
            _ => panic!("Json is not an object"),
        }
    }
}

impl AsMut<str> for Json {
    fn as_mut(&mut self) -> &mut str {
        match self {
            Json::String(s) => s,
            _ => panic!("Json is not string"),
        }
    }
}

impl AsMut<Vec<Json>> for Json {
    fn as_mut(&mut self) -> &mut Vec<Json> {
        match self {
            Json::Array(arr) => arr,
            _ => panic!("Json is not an array"),
        }
    }
}

impl AsMut<Vec<Property>> for Json {
    fn as_mut(&mut self) -> &mut Vec<Property> {
        match self {
            Json::Object(obj) => obj,
            _ => panic!("Json is not an object"),
        }
    }
}

impl Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Json::{Array, Bool, Float, Integer, Null, Object, String as S};
        use std::str::from_utf8;

        match self {
            Null => write!(f, "null"),
            Bool(true) => write!(f, "true"),
            Bool(false) => write!(f, "false"),
            Integer(Integral { val: Some(v), .. }) => write!(f, "{}", v),
            Integer(Integral { len, txt, .. }) => write!(f, "{}", from_utf8(&txt[..*len]).unwrap()),
            Float(Floating { val: Some(v), .. }) => write!(f, "{:e}", v),
            Float(Floating { len, txt, .. }) => write!(f, "{}", from_utf8(&txt[..*len]).unwrap()),
            S(val) => {
                encode_string(f, &val)?;
                Ok(())
            }
            Array(val) => {
                if val.is_empty() {
                    write!(f, "[]")
                } else {
                    write!(f, "[")?;
                    for item in val[..val.len() - 1].iter() {
                        write!(f, "{},", item)?;
                    }
                    write!(f, "{}", val[val.len() - 1])?;
                    write!(f, "]")
                }
            }
            Object(val) => {
                let val_len = val.len();
                if val_len == 0 {
                    write!(f, "{{}}")
                } else {
                    write!(f, "{{")?;
                    for (i, prop) in val.iter().enumerate() {
                        encode_string(f, prop.key_ref())?;
                        write!(f, ":{}", prop.value_ref())?;
                        if i < (val_len - 1) {
                            write!(f, ",")?;
                        }
                    }
                    write!(f, "}}")
                }
            }
            Json::__Error(err) => write!(f, "error: {}", err),
        }
    }
}

fn encode_string<W: Write>(w: &mut W, val: &str) -> fmt::Result {
    write!(w, "\"")?;

    let mut start = 0;
    for (i, byte) in val.bytes().enumerate() {
        let escstr = ESCAPE[byte as usize];
        if escstr.is_empty() {
            continue;
        }

        if start < i {
            write!(w, "{}", &val[start..i])?;
        }
        write!(w, "{}", escstr)?;
        start = i + 1;
    }
    if start != val.len() {
        write!(w, "{}", &val[start..])?;
    }
    write!(w, "\"")
}

pub fn insert(json: &mut Json, item: Property) {
    if let Json::Object(obj) = json {
        match property::search_by_key(&obj, item.key_ref()) {
            Ok(off) => obj.insert(off, item),
            Err(off) => obj.insert(off, item),
        }
    }
}

/// Jsons can parse a stream of JSON text supplied by any [Read] instance.
///
/// Any [Read] instance can be converted to Jsons instance and iterated.
/// For Example:
///
/// ```
/// extern crate jsondata;
/// use jsondata::{Json, Jsons};
/// use std::fs::File;
/// let file = File::open("testdata/stream1.jsons").unwrap();
/// let mut iter: Jsons<File> = file.into();
///
/// for json in iter {
///     println!("{:?}", json)
/// }
/// ```
///
/// Note that the iterated value is of type ``Result<Json, std::io::Error>``,
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
///             /* handle std::io::Error returned by the Read instance */
///         },
///     }
/// }
/// ```
///
/// [Read]: std::io::Read
pub struct Jsons<R>
where
    R: io::Read
{
    codes: CodePoints<io::Bytes<R>>,
    quant: String,
}

impl<R> From<R> for Jsons<R>
where
    R: io::Read
{
    fn from(input: R) -> Jsons<R> {
        Jsons{codes: input.into(), quant: String::with_capacity(1024)}
    }
}

impl<R> Iterator for Jsons<R>
where
    R: io::Read
{
    type Item=Result<Json, io::Error>;

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
                                    break
                                }
                            } else if markers.is_empty() {
                                break
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
                },
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
                break res
            }
            let res = self.codes.next();
            if res.is_none() && !self.quant.is_empty() {
                let res = match self.quant.parse() {
                    Ok(json) => Some(Ok(json)),
                    Err(s) => Some(Ok(Json::__Error(s))),
                };
                //println!("quant {:?} {:?}", self.quant.as_bytes(), res);
                self.quant.truncate(0);
                break res
            } else if res.is_none() {
                break None
            }
            ok_ch = res.unwrap();
        }
    }
}

impl<R> Jsons<R>
where
    R: io::Read
{
    fn read_string(&mut self) -> Option<Result<(), io::Error>> {
        let mut escape = false;
        loop {
            match self.codes.next() {
                Some(Ok(ch)) if escape => { self.quant.push(ch); escape = false;},
                Some(Ok('\\')) => { self.quant.push('\\'); escape = true; },
                Some(Ok('"')) => { self.quant.push('"'); break Some(Ok(())); }
                Some(Ok(ch)) => self.quant.push(ch),
                Some(Err(err)) => break Some(Err(err)),
                None => break Some(Ok(())),
            }
        }
    }

    fn read_whitespace(&mut self) -> Option<Result<char, io::Error>> {
        loop {
            match self.codes.next()? {
                Ok(ch) if !ch.is_whitespace() => break Some(Ok(ch)),
                Ok(_) => (),
                Err(err) => break Some(Err(err)),
            }
        }
    }
}

static ESCAPE: [&'static str; 256] = [
    "\\u0000", "\\u0001", "\\u0002", "\\u0003", "\\u0004", "\\u0005", "\\u0006", "\\u0007", "\\b",
    "\\t", "\\n", "\\u000b", "\\f", "\\r", "\\u000e", "\\u000f", "\\u0010", "\\u0011", "\\u0012",
    "\\u0013", "\\u0014", "\\u0015", "\\u0016", "\\u0017", "\\u0018", "\\u0019", "\\u001a",
    "\\u001b", "\\u001c", "\\u001d", "\\u001e", "\\u001f", "", "", "\\\"", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
    "", "", "", "\\\\", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "\\u007f", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
    "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
    "", "",
];
