use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Privacy {
    Public,
    Protected,
    Private,
}

impl fmt::Display for Privacy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let privacy = match self {
            Privacy::Public => "public",
            Privacy::Protected => "protected",
            Privacy::Private => "private",
        };

        f.write_str(privacy)
    }
}
