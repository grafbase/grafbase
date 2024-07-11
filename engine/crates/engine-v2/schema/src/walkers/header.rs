use crate::{HeaderRule, HeaderRuleId, NameOrPattern, SchemaWalker};
use regex::Regex;
use std::fmt;

pub type HeaderRuleWalker<'a> = SchemaWalker<'a, HeaderRuleId>;

impl<'a> HeaderRuleWalker<'a> {
    pub fn rule(&self) -> HeaderRuleRef<'a> {
        match &self.schema[self.item] {
            HeaderRule::Forward { name, default, rename } => HeaderRuleRef::Forward {
                name: self.name_or_pattern_ref(name),
                default: default.map(|id| self.schema[id].as_str()),
                rename: rename.map(|id| self.schema[id].as_str()),
            },
            HeaderRule::Insert { name, value } => HeaderRuleRef::Insert {
                name: self.schema[*name].as_str(),
                value: self.schema[*value].as_str(),
            },
            HeaderRule::Remove { name } => HeaderRuleRef::Remove {
                name: self.name_or_pattern_ref(name),
            },
        }
    }

    fn name_or_pattern_ref(&self, name: &'a NameOrPattern) -> NameOrPatternRef<'a> {
        match name {
            NameOrPattern::Pattern(regex_id) => NameOrPatternRef::Pattern(&self.schema[*regex_id]),
            NameOrPattern::Name(name_id) => NameOrPatternRef::Name(self.schema[*name_id].as_str()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NameOrPatternRef<'a> {
    Pattern(&'a Regex),
    Name(&'a str),
}

#[derive(Debug)]
pub enum HeaderRuleRef<'a> {
    Forward {
        name: NameOrPatternRef<'a>,
        default: Option<&'a str>,
        rename: Option<&'a str>,
    },
    Insert {
        name: &'a str,
        value: &'a str,
    },
    Remove {
        name: NameOrPatternRef<'a>,
    },
}

impl<'a> fmt::Debug for HeaderRuleWalker<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubgraphHeaderWalker")
            .field("rule", &self.rule())
            .finish()
    }
}
