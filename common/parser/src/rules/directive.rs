pub trait Directive {
    fn definition() -> String;
}

pub struct Directives(Vec<String>);

impl Directives {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn to_definition(&self) -> String {
        self.0.join("\n")
    }

    pub fn with<D: Directive>(self) -> Directives {
        let mut v = self.0;
        v.push(D::definition());
        Self(v)
    }
}
