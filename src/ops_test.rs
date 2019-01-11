use std::ops::{Add, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};
use std::ops::{BitAnd, BitOr, BitXor};

use property::Property;
use json::Json;

#[test]
fn test_ops_add() {
    // Null as lhs
    assert_eq!(Json::Null, Json::Null + Json::Null);
    let refval: Json = true.into();
    assert_eq!(refval, Json::Null + true.into());
    let refval: Json = false.into();
    assert_eq!(refval, Json::Null + false.into());
    let refval: Json = 10.into();
    assert_eq!(refval, Json::Null + 10.into());
    let refval: Json = 10.2.into();
    assert_eq!(refval, Json::Null + 10.2.into());
    let refval: Json = "hello".into();
    assert_eq!(refval, Json::Null + "hello".into());
    let rhs: Json = vec![Json::new(1), 2.into()].into();
    let refval: Json = vec![Json::new(1), 2.into()].into();
    assert_eq!(refval, Json::Null + rhs);
    let rhs: Json = vec![Property::new("a", 10.into())].into();
    let refval: Json = vec![Property::new("a", 10.into())].into();
    assert_eq!(refval, Json::Null + rhs);

    // Null as rhs
    assert_eq!(Json::Null, Json::Null + Json::Null);
    let refval: Json = true.into();
    assert_eq!(refval, Json::new(true) + Json::Null);
    let refval: Json = false.into();
    assert_eq!(refval, Json::new(false) + Json::Null);
    let refval: Json = 10.into();
    assert_eq!(refval, Json::new(10) + Json::Null);
    let refval: Json = 10.2.into();
    assert_eq!(refval, Json::new(10.2) + Json::Null);
    let refval: Json = "hello".into();
    assert_eq!(refval, Json::new("hello") + Json::Null);
    let lhs: Json = vec![Json::new(1), 2.into()].into();
    let refval: Json = vec![Json::new(1), 2.into()].into();
    assert_eq!(refval, lhs + Json::Null);
    let lhs: Json = vec![Property::new("a", 10.into())].into();
    let refval: Json = vec![Property::new("a", 10.into())].into();
    assert_eq!(refval, lhs + Json::Null);

    // Integers and floats
    assert_eq!(Json::new(20), Json::new(10) + 10.into());
    assert_eq!(Json::new(20.2), Json::new(10.1) + 10.1.into());
    assert_eq!(Json::new(20.2), Json::new(10) + 10.2.into());
    assert_eq!(Json::new(20.2), Json::new(10.2) + 10.into());

    // String addition
    assert_eq!(Json::new("helloworld"), Json::new("hello") + "world".into());

    // Array addition
    let lhs: Json = vec![Json::new(1), 2.into()].into();
    let rhs: Json = vec![Json::new(2), 1.into()].into();
    let refval: Json = vec![Json::new(1), 2.into(), 2.into(), 1.into()].into();
    assert_eq!(refval, lhs + rhs);

    // Object addition
    let lhs: Json = vec![Property::new("a", 10.into()), Property::new("b", 11.into())].into();
    let rhs: Json = vec![Property::new("b", 20.into())].into();
    let refval: Json = vec![Property::new("a", 10.into()), Property::new("b", 20.into())].into();
    assert_eq!(refval, lhs + rhs);
}

#[test]
fn test_ops_sub() {
    // Null as lhs
    assert_eq!(Json::Null, Json::Null - Json::Null);
    let refval: Json = true.into();
    assert_eq!(refval, Json::Null - true.into());
    let refval: Json = false.into();
    assert_eq!(refval, Json::Null - false.into());
    let refval: Json = 10.into();
    assert_eq!(refval, Json::Null - 10.into());
    let refval: Json = 10.2.into();
    assert_eq!(refval, Json::Null - 10.2.into());
    let refval: Json = "hello".into();
    assert_eq!(refval, Json::Null - "hello".into());
    let rhs: Json = vec![Json::new(1), 2.into()].into();
    let refval: Json = vec![Json::new(1), 2.into()].into();
    assert_eq!(refval, Json::Null - rhs);
    let rhs: Json = vec![Property::new("a", 10.into())].into();
    let refval: Json = vec![Property::new("a", 10.into())].into();
    assert_eq!(refval, Json::Null - rhs);

    // Null as rhs
    assert_eq!(Json::Null, Json::Null - Json::Null);
    let refval: Json = true.into();
    assert_eq!(refval, Json::new(true) - Json::Null);
    let refval: Json = false.into();
    assert_eq!(refval, Json::new(false) - Json::Null);
    let refval: Json = 10.into();
    assert_eq!(refval, Json::new(10) - Json::Null);
    let refval: Json = 10.2.into();
    assert_eq!(refval, Json::new(10.2) - Json::Null);
    let refval: Json = "hello".into();
    assert_eq!(refval, Json::new("hello") - Json::Null);
    let lhs: Json = vec![Json::new(1), 2.into()].into();
    let refval: Json = vec![Json::new(1), 2.into()].into();
    assert_eq!(refval, lhs - Json::Null);
    let lhs: Json = vec![Property::new("a", 10.into())].into();
    let refval: Json = vec![Property::new("a", 10.into())].into();
    assert_eq!(refval, lhs - Json::Null);

    // Integers and floats
    assert_eq!(Json::new(10), Json::new(20) - 10.into());
    assert_eq!(Json::new(20.1-10.1), Json::new(20.1) - 10.1.into());
    assert_eq!(Json::new(9.8), Json::new(20) - 10.2.into());
    assert_eq!(Json::new(10.2-(10 as f64)), Json::new(10.2) - 10.into());

    // Array substraction
    let lhs: Json = vec![Json::new(1), 1.into(), 2.into(), 2.into(), 2.into()].into();
    let rhs: Json = vec![Json::new(2), 2.into(), 1.into()].into();
    let refval: Json = vec![Json::new(1), 2.into()].into();
    assert_eq!(refval, lhs - rhs);

    // Object substraction
    let lhs: Json = vec![Property::new("a", 10.into()), Property::new("b", 20.into())].into();
    let rhs: Json = vec![Property::new("b", 20.into())].into();
    let refval: Json = vec![Property::new("a", 10.into())].into();
    assert_eq!(refval, lhs - rhs);
}

#[test]
fn test_ops_mul() {
    // Null as lhs
    assert_eq!(Json::Null, Json::Null * Json::Null);
    assert_eq!(Json::Null, Json::Null * true.into());
    assert_eq!(Json::Null, Json::Null * false.into());
    assert_eq!(Json::Null, Json::Null * 10.into());
    assert_eq!(Json::Null, Json::Null * 10.2.into());
    assert_eq!(Json::Null, Json::Null * "hello".into());
    let rhs: Json = vec![Json::new(1), 2.into()].into();
    assert_eq!(Json::Null, Json::Null * rhs);
    let rhs: Json = vec![Property::new("a", 10.into())].into();
    assert_eq!(Json::Null, Json::Null * rhs);

    // Null as rhs
    assert_eq!(Json::Null, Json::Null * Json::Null);
    assert_eq!(Json::Null, Json::new(true) * Json::Null);
    assert_eq!(Json::Null, Json::new(false) * Json::Null);
    assert_eq!(Json::Null, Json::new(10) * Json::Null);
    assert_eq!(Json::Null, Json::new(10.2) * Json::Null);
    assert_eq!(Json::Null, Json::new("hello") * Json::Null);
    let lhs: Json = vec![Json::new(1), 2.into()].into();
    assert_eq!(Json::Null, lhs * Json::Null);
    let lhs: Json = vec![Property::new("a", 10.into())].into();
    assert_eq!(Json::Null, lhs * Json::Null);

    // Integers and floats
    assert_eq!(Json::new(200), Json::new(20) * 10.into());
    assert_eq!(Json::new(20.1*10.1), Json::new(20.1) * 10.1.into());
    assert_eq!(Json::new((20 as f64)*10.2), Json::new(20) * 10.2.into());
    assert_eq!(Json::new(10.2*(10 as f64)), Json::new(10.2) * 10.into());

    // String multiplication
    assert_eq!(Json::new("okokok"), Json::new("ok") * 3.into());
    assert_eq!(Json::new("okokok"), Json::new(3) * "ok".into());
}

#[test]
fn test_ops_div() {
    // Null as lhs
    assert_eq!(Json::Null, Json::Null / Json::Null);
    assert_eq!(Json::Null, Json::Null / true.into());
    assert_eq!(Json::Null, Json::Null / false.into());
    assert_eq!(Json::Null, Json::Null / 10.into());
    assert_eq!(Json::Null, Json::Null / 10.2.into());
    assert_eq!(Json::Null, Json::Null / "hello".into());
    let rhs: Json = vec![Json::new(1), 2.into()].into();
    assert_eq!(Json::Null, Json::Null / rhs);
    let rhs: Json = vec![Property::new("a", 10.into())].into();
    assert_eq!(Json::Null, Json::Null / rhs);

    // Null as rhs
    assert_eq!(Json::Null, Json::Null / Json::Null);
    assert_eq!(Json::Null, Json::new(true) / Json::Null);
    assert_eq!(Json::Null, Json::new(false) / Json::Null);
    assert_eq!(Json::Null, Json::new(10) / Json::Null);
    assert_eq!(Json::Null, Json::new(10.2) / Json::Null);
    assert_eq!(Json::Null, Json::new("hello") / Json::Null);
    let lhs: Json = vec![Json::new(1), 2.into()].into();
    assert_eq!(Json::Null, lhs / Json::Null);
    let lhs: Json = vec![Property::new("a", 10.into())].into();
    assert_eq!(Json::Null, lhs / Json::Null);

    // Integers and floats
    assert_eq!(Json::new(2), Json::new(20) / 10.into());
    assert_eq!(Json::new(20.1/10.1), Json::new(20.1) / 10.1.into());
    assert_eq!(Json::new((20 as f64)/10.2), Json::new(20) / 10.2.into());
    assert_eq!(Json::new(10.2/(10 as f64)), Json::new(10.2) / 10.into());
}

#[test]
fn test_ops_rem() {
    // Null as lhs
    assert_eq!(Json::Null, Json::Null % Json::Null);
    assert_eq!(Json::Null, Json::Null % true.into());
    assert_eq!(Json::Null, Json::Null % false.into());
    assert_eq!(Json::Null, Json::Null % 10.into());
    assert_eq!(Json::Null, Json::Null % 10.2.into());
    assert_eq!(Json::Null, Json::Null % "hello".into());
    let rhs: Json = vec![Json::new(1), 2.into()].into();
    assert_eq!(Json::Null, Json::Null % rhs);
    let rhs: Json = vec![Property::new("a", 10.into())].into();
    assert_eq!(Json::Null, Json::Null % rhs);

    // Null as rhs
    assert_eq!(Json::Null, Json::Null % Json::Null);
    assert_eq!(Json::Null, Json::new(true) % Json::Null);
    assert_eq!(Json::Null, Json::new(false) % Json::Null);
    assert_eq!(Json::Null, Json::new(10) % Json::Null);
    assert_eq!(Json::Null, Json::new(10.2) % Json::Null);
    assert_eq!(Json::Null, Json::new("hello") % Json::Null);
    let lhs: Json = vec![Json::new(1), 2.into()].into();
    assert_eq!(Json::Null, lhs % Json::Null);
    let lhs: Json = vec![Property::new("a", 10.into())].into();
    assert_eq!(Json::Null, lhs % Json::Null);

    // Integers and floats
    assert_eq!(Json::new(2), Json::new(202) % 10.into());
    assert_eq!(Json::new(20.1%10.1), Json::new(20.1) % 10.1.into());
    assert_eq!(Json::new((20 as f64)%10.2), Json::new(20) % 10.2.into());
    assert_eq!(Json::new(10.2%(10 as f64)), Json::new(10.2) % 10.into());
}

#[test]
fn test_ops_neg() {
    // Null as lhs
    assert_eq!(Json::Null, -Json::Null);

    // Integers and floats
    assert_eq!(Json::new(-202), -Json::new(202));
    assert_eq!(Json::new(-20.1), -Json::new(20.1));
}
