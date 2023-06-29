use std::fmt::{self, Write};

use crate::{block::Block, common::Identifier, r#type::TypeKind};

#[derive(Default)]
pub struct Closure<'a> {
    params: Vec<Identifier<'a>>,
    input_types: Vec<TypeKind<'a>>,
    return_type: Option<TypeKind<'a>>,
    body: Block<'a>,
}

#[allow(dead_code)]
impl<'a> Closure<'a> {
    #[must_use]
    pub fn new(body: Block<'a>) -> Self {
        Self {
            body,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn params(mut self, params: Vec<Identifier<'a>>) -> Self {
        self.params = params;
        self
    }

    pub fn returns(mut self, return_type: impl Into<TypeKind<'a>>) -> Self {
        self.return_type = Some(return_type.into());
        self
    }

    #[must_use]
    pub fn typed_params(mut self, params: Vec<(Identifier<'a>, impl Into<TypeKind<'a>>)>) -> Self {
        let (params, input_types): (Vec<_>, Vec<_>) = params.into_iter().map(|(a, b)| (a, b.into())).unzip();

        self.params = params;
        self.input_types = input_types;

        self
    }
}

impl<'a> fmt::Display for Closure<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('(')?;

        for (i, param) in self.params.iter().enumerate() {
            write!(f, "{param}")?;

            if !self.input_types.is_empty() {
                write!(f, ": {}", self.input_types[i])?;
            }

            f.write_str(", ")?;
        }

        f.write_char(')')?;

        if let Some(ref return_type) = self.return_type {
            write!(f, ": {return_type}")?;
        }

        write!(f, " => {}", self.body)?;

        Ok(())
    }
}
