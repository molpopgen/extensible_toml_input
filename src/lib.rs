use serde::Deserialize;
use serde::Serialize;

trait TraitA {}
trait TraitB {}

pub struct Graph<A: TraitA, B: TraitB> {
    a: A,
    b: B,
}

fn default_graph() -> Graph<isA, isB> {
    Graph {
        a: isA { x: 0, y: 0 },
        b: isB { i: -1 },
    }
}

// NOTE: struct update syntax is not
// possible b/c the generic types change!
impl<A, B> Graph<A, B>
where
    A: TraitA,
    B: TraitB,
{
    fn set_a<NewA: TraitA>(self, newa: NewA) -> Graph<NewA, B> {
        Graph { a: newa, b: self.b }
    }
    fn set_b<NewB: TraitB>(self, newb: NewB) -> Graph<A, NewB> {
        Graph { a: self.a, b: newb }
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

#[derive(Deserialize)]
enum AllowedA {
    isA(isA),
    isAlsoA(isAlsoA),
}

#[derive(Deserialize)]
enum AllowedB {
    isB(isB),
    isAlsoB(isAlsoB),
}

#[derive(Deserialize)]
struct InputGraph {
    a: Option<AllowedA>,
    b: Option<AllowedB>,
}

#[test]
fn test_toml() {
    let toml = r"
    [a.isA]
    x = 3
    y = 4
    ";
    let i: InputGraph = toml::from_str(toml).unwrap();
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
