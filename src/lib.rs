use serde::Deserialize;
use serde::Serialize;

trait TraitA {}
trait TraitB {}

pub struct ResolvedType {
    a: Box<dyn TraitA>,
    b: Box<dyn TraitB>,
}

fn default_graph() -> ResolvedType {
    ResolvedType {
        a: Box::new(isA { x: 0, y: 0 }),
        b: Box::new(isB { i: -1 }),
    }
}

#[derive(Default, Serialize, Deserialize)]
struct isA {
    x: i32,
    y: i32,
}

#[derive(Serialize, Deserialize)]
struct isAlsoA {
    x: String,
    weight: f64,
}

#[derive(Default, Serialize, Deserialize)]
struct isB {
    i: i32,
}

#[derive(Serialize, Deserialize)]
struct isAlsoB {
    i: i32,
    j: i64,
}

impl TraitA for isA {}
impl TraitA for isAlsoA {}
impl TraitB for isB {}
impl TraitB for isAlsoB {}

macro_rules! make_allowed_a {
    () => {
        #[derive(Deserialize)]
        enum AllowedA {
            isA(isA),
            isAlsoA(isAlsoA),
        }

        impl AllowedA {
            fn into_boxed(self) -> Box<dyn TraitA> {
                match self {
                    Self::isA(a) => Box::new(a),
                    Self::isAlsoA(a) => Box::new(a)
                }
            }
        }
    };
    ($($tag:tt : $N:ty)+) => {
        #[derive(Deserialize)]
        enum AllowedA {
            isA(isA),
            isAlsoA(isAlsoA),
            $($tag($N),)+
        }
        impl AllowedA {
            fn into_boxed(self) -> Box<dyn TraitA> {
                match self {
                    Self::isA(a) => Box::new(a),
                    Self::isAlsoA(a) => Box::new(a),
                    $(Self::$tag(a) => Box::new(a)),+
                }
            }
        }
    };
}

#[derive(Deserialize)]
enum AllowedB {
    isB(isB),
    isAlsoB(isAlsoB),
}

impl AllowedB {
    fn into_boxed(self) -> Box<dyn TraitB> {
        match self {
            Self::isB(b) => Box::new(b),
            Self::isAlsoB(b) => Box::new(b),
        }
    }
}

macro_rules! make_input_graph {
    () => {
        #[derive(Deserialize)]
        struct InputType {
            a: Option<AllowedA>,
            b: Option<AllowedB>,
        }
        impl From<InputType> for ResolvedType {
            fn from(value: InputType) -> ResolvedType {
                let a = match value.a {
                    Some(a) => a.into_boxed(),
                    None => Box::new(isA::default()),
                };
                let b = match value.b {
                    Some(b) => b.into_boxed(),
                    None => Box::new(isB::default()),
                };
                ResolvedType { a, b }
            }
        }
    };
}

#[test]
fn test_toml() {
    make_allowed_a!();
    make_input_graph!();
    let toml = r"
    [a.isA]
    x = 3
    y = 4
    ";
    let i: InputType = toml::from_str(toml).unwrap();
    assert!(i.a.is_some());
    assert!(i.b.is_none());

    match i.a {
        Some(a) => match a {
            AllowedA::isA(_) => (),
            AllowedA::isAlsoA(_) => panic!("wrong variant"),
        },
        None => panic!("expected Some(...)"),
    }
}

#[test]
fn test_extended_toml() {
    #[derive(Deserialize)]
    struct MyA {
        data: String,
    }
    impl TraitA for MyA {}

    make_allowed_a!(MyA:MyA);
    make_input_graph!();
    let toml = r#"
    [a.MyA]
    data = "foobar"
    "#;
    let i: InputType = toml::from_str(toml).unwrap();
    assert!(i.a.is_some());
    assert!(i.b.is_none());

    match &i.a {
        Some(a) => match a {
            AllowedA::isA(_) => panic!("wrong variant"),
            AllowedA::isAlsoA(_) => panic!("wrong variant"),
            AllowedA::MyA(a) => {
                assert_eq!(a.data, "foobar")
            }
        },
        None => panic!("expected Some(...)"),
    }
    let _ = ResolvedType::from(i);
}

#[test]
fn test_doubly_extended_toml() {
    #[derive(Deserialize)]
    struct MyA {
        data: String,
    }
    impl TraitA for MyA {}

    #[derive(Deserialize)]
    struct MyOtherA {
        datum: f64,
    }
    impl TraitA for MyOtherA {}

    make_allowed_a!(MyA:MyA MyOtherA:MyOtherA);
    make_input_graph!();
    let toml = r#"
    [a.MyOtherA]
    datum = nan
    "#;
    let i: InputType = toml::from_str(toml).unwrap();
    assert!(i.a.is_some());
    assert!(i.b.is_none());

    match &i.a {
        Some(a) => match a {
            AllowedA::isA(_) => panic!("wrong variant"),
            AllowedA::isAlsoA(_) => panic!("wrong variant"),
            AllowedA::MyA(_) => {
                panic!("wrong variant")
            }
            AllowedA::MyOtherA(a) => {
                assert!(a.datum.is_nan())
            }
        },
        None => panic!("expected Some(...)"),
    }
    let _ = ResolvedType::from(i);
}
