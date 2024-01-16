mod rules;

use crate::{FieldUsage, Schema};
use graphql_schema_diff::{Change, ChangeKind};
use std::collections::HashSet;

/// A diagnostic produced by [check()].
#[derive(Debug)]
pub struct CheckDiagnostic {
    /// The message of the diagnostic.
    pub message: String,
    /// See [Severity].
    pub severity: Severity,
}

/// The severity of a [CheckDiagnostic].
#[derive(Debug)]
pub enum Severity {
    /// Could be breaking.
    Warning,
    /// Is breaking.
    Error,
}

/// The arguments to [check()].
pub struct CheckParams<'a> {
    /// The source (old, previous) [Schema].
    pub source: &'a Schema,
    /// The target (new, next) [Schema].
    pub target: &'a Schema,
    /// The diff between source and target.
    pub diff: &'a [Change],
    /// Field usage from operations.
    pub field_usage: &'a FieldUsage,
}

impl CheckParams<'_> {
    fn field_is_used(&self, type_and_field: &str) -> bool {
        let (type_name, field_name) = type_and_field.split_once('.').unwrap();

        let Some(field_id) = self.source.find_field(type_name, field_name) else {
            return false;
        };

        self.field_usage.count_per_field.contains_key(&field_id)
    }

    fn argument_is_used(&self, path: &str) -> bool {
        let Some(argument_id) = self.find_argument(path) else {
            return false;
        };

        self.field_usage.count_per_field_argument.contains_key(&argument_id)
    }

    fn enum_value_is_used(&self, path: &str) -> bool {
        self.field_usage.count_per_enum_value.contains_key(path)
    }

    fn argument_is_left_out(&self, path: &str) -> bool {
        let Some(argument_id) = self.find_argument(path) else {
            return false;
        };

        self.field_usage
            .arguments_with_defaults_left_out_count
            .contains_key(&argument_id)
    }

    fn find_argument(&self, path: &str) -> Option<crate::schema::ArgumentId> {
        let mut path = path.split('.');
        let type_name = path.next().unwrap();
        let field_name = path.next().unwrap();
        let argument_name = path.next().unwrap();

        self.source.find_argument((type_name, field_name, argument_name))
    }
}

/// Perform operation checks.
pub fn check(params: &CheckParams<'_>) -> Vec<CheckDiagnostic> {
    let mut used_input_types = None;

    params
        .diff
        .iter()
        .filter_map(|change| {
            check_change(CheckArgs {
                change,
                check_params: params,
                used_input_types: &mut used_input_types,
            })
        })
        .collect()
}

struct CheckArgs<'a, 'b> {
    change: &'a Change,
    check_params: &'a CheckParams<'a>,
    used_input_types: &'b mut Option<HashSet<&'a str>>,
}

fn check_change(args: CheckArgs<'_, '_>) -> Option<CheckDiagnostic> {
    match args.change.kind {
        // Not relevant for federated graphs.
        ChangeKind::ChangeMutationType
        | ChangeKind::ChangeSubscriptionType
        | ChangeKind::ChangeQueryType

        // Not relevant for federated graphs.
        | ChangeKind::AddSchemaDefinition
        | ChangeKind::RemoveSchemaDefinition

        // Directives do not directly affect the shape of the API.
        | ChangeKind::AddDirectiveDefinition
        | ChangeKind::RemoveDirectiveDefinition

        // Adding or changing the default on an argument will not break clients.
        | ChangeKind::AddFieldArgumentDefault
        | ChangeKind::ChangeFieldArgumentDefault

        // Making an object or an interface implement a new interface is safe.
        | ChangeKind::AddInterfaceImplementation

        // Adding a member to a union is safe.
        | ChangeKind::AddUnionMember

        // Adding a value to an enum is safe.
        | ChangeKind::AddEnumValue

        // Adding types is always safe.
        | ChangeKind::AddInputObject
        | ChangeKind::AddInterface
        | ChangeKind::AddObjectType
        | ChangeKind::AddScalar
        | ChangeKind::AddUnion
        | ChangeKind::AddEnum

        // Removing a type means it isn't used anymore. That may translate to field type changes,
        // which are the actual breaking changes.
        | ChangeKind::RemoveEnum
        | ChangeKind::RemoveScalar
        | ChangeKind::RemoveInterface
        | ChangeKind::RemoveInputObject
        | ChangeKind::RemoveUnion => None,

        ChangeKind::RemoveObjectType => rules::remove_object_type(args),

        ChangeKind::RemoveField => rules::remove_field(args),

        ChangeKind::AddFieldArgument => rules::add_field_argument(args),

        ChangeKind::AddField => rules::add_field(args),

        ChangeKind::ChangeFieldArgumentType => rules::change_field_argument_type(args),

        ChangeKind::RemoveFieldArgument => rules::remove_field_argument(args),

        ChangeKind::ChangeFieldType => rules::change_field_type(args),

        ChangeKind::RemoveInterfaceImplementation => rules::remove_interface_implementation(args),

        ChangeKind::RemoveUnionMember => rules::remove_union_member(args),

        ChangeKind::RemoveEnumValue => rules::remove_enum_value(args),

        ChangeKind::RemoveFieldArgumentDefault  => rules::remove_field_argument_default(args),
    }
}

fn trim_to_field_path(path: &str) -> &str {
    let mut positions = path.match_indices('.');

    positions
        .next()
        .expect("Expected field path, but no dot separator found.");

    match positions.next() {
        Some((idx, _)) => path.split_at(idx).0,
        None => path,
    }
}

fn find_used_input_types<'a>(params: &CheckParams<'a>) -> HashSet<&'a str> {
    fn find_used_input_types_rec<'a>(
        root_type: &'a str,
        params: &CheckParams<'a>,
        used_input_types: &mut HashSet<&'a str>,
    ) {
        if !used_input_types.insert(root_type) {
            return;
        }

        for field in params.source.iter_fields(root_type) {
            find_used_input_types_rec(&field.base_type, params, used_input_types);
        }
    }

    let mut used_input_types = HashSet::new();

    for arg in params.field_usage.count_per_field_argument.keys() {
        let arg = &params.source[*arg];
        let arg_type = &arg.base_type;

        find_used_input_types_rec(arg_type, params, &mut used_input_types);
    }

    used_input_types
}
