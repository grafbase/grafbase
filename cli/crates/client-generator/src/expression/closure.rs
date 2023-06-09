use std::fmt::{self, Write};

use crate::{block::Block, common::Identifier, r#type::TypeKind};

#[derive(Debug, Default)]
pub struct Closure {
    params: Vec<Identifier>,
    input_types: Vec<TypeKind>,
    return_type: Option<TypeKind>,
    body: Block,
}

impl Closure {
    #[must_use]
    pub fn new(body: Block) -> Self {
        Self {
            body,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn params(mut self, params: Vec<Identifier>) -> Self {
        self.params = params;
        self
    }

    pub fn returns(mut self, return_type: impl Into<TypeKind>) -> Self {
        self.return_type = Some(return_type.into());
        self
    }

    #[must_use]
    pub fn typed_params(mut self, params: Vec<(Identifier, impl Into<TypeKind>)>) -> Self {
        let (params, input_types): (Vec<_>, Vec<_>) = params.into_iter().map(|(a, b)| (a, b.into())).unzip();

        self.params = params;
        self.input_types = input_types;

        self
    }
}

impl fmt::Display for Closure {
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
