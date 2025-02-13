use std::borrow::Cow;

use walker::Walk;

use crate::Schema;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct TemplateId(u32);

pub struct TemplateRecord {
    pub inner: ramhorns::Template<'static>,
    pub escaping: TemplateEscaping,
}

impl TemplateRecord {
    pub(crate) fn new(source: String, escaping: TemplateEscaping) -> Result<Self, ramhorns::Error> {
        Ok(Self {
            inner: ramhorns::Template::new(source)?,
            escaping,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum TemplateEscaping {
    Json,
    Url,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SerdeTemplate<'a> {
    source: Cow<'a, str>,
    escaping: TemplateEscaping,
}

impl serde::Serialize for TemplateRecord {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SerdeTemplate {
            source: self.inner.source().into(),
            escaping: self.escaping,
        }
        .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for TemplateRecord {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let SerdeTemplate::<'static> { source, escaping } = SerdeTemplate::deserialize(deserializer)?;
        let inner = ramhorns::Template::new(source.into_owned()).map_err(serde::de::Error::custom)?;
        Ok(Self { inner, escaping })
    }
}

#[derive(Clone, Copy)]
pub struct Template<'a> {
    pub(crate) schema: &'a Schema,
    pub id: TemplateId,
}

impl std::ops::Deref for Template<'_> {
    type Target = TemplateRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> Template<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a TemplateRecord {
        &self.schema[self.id]
    }
}

impl<'a> Walk<&'a Schema> for TemplateId {
    type Walker<'w>
        = Template<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        Template {
            schema: schema.into(),
            id: self,
        }
    }
}
