#[derive(serde::Serialize, Debug)]
pub(crate) struct Binding {
    r#type: &'static str,
    value: String,
}

impl Binding {
    pub(crate) fn fixed(value: String) -> Binding {
        Binding { r#type: "FIXED", value }
    }

    pub(crate) fn real(value: String) -> Binding {
        Binding { r#type: "REAL", value }
    }

    pub(crate) fn text(value: String) -> Binding {
        Binding { r#type: "TEXT", value }
    }

    pub(crate) fn boolean(value: String) -> Binding {
        Binding {
            r#type: "BOOLEAN",
            value,
        }
    }
}
