use crate::{HeaderRuleId, HeaderRuleRecord, NameOrPattern, SchemaWalker};
use regex::Regex;
use std::fmt;

pub type HeaderRule<'a> = SchemaWalker<'a, HeaderRuleId>;

impl<'a> HeaderRule<'a> {
    pub fn rule(&self) -> HeaderRuleRef<'a> {
        match &self.schema[self.item] {
            HeaderRuleRecord::Forward {
                name_id: name,
                default,
                rename,
            } => HeaderRuleRef::Forward {
                name: self.name_or_pattern_ref(name),
                default: default.map(|id| self.schema[id].as_str()),
                rename: rename.map(|id| self.schema[id].as_str()),
            },
            HeaderRuleRecord::Insert { name_id: name, value } => HeaderRuleRef::Insert {
                name: self.schema[*name].as_str(),
                value: self.schema[*value].as_str(),
            },
            HeaderRuleRecord::Remove { name_id: name } => HeaderRuleRef::Remove {
                name: self.name_or_pattern_ref(name),
            },
            HeaderRuleRecord::RenameDuplicate {
                name_id: name,
                default,
                rename,
            } => HeaderRuleRef::RenameDuplicate {
                name: self.schema[*name].as_str(),
                default: default.map(|id| self.schema[id].as_str()),
                rename: self.schema[*rename].as_str(),
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
    RenameDuplicate {
        name: &'a str,
        default: Option<&'a str>,
        rename: &'a str,
    },
}

impl<'a> fmt::Debug for HeaderRule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubgraphHeaderWalker")
            .field("rule", &self.rule())
            .finish()
    }
}
