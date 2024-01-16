use super::*;

/// Removing a field is breaking iff it is used in any operation. This is easy to determine
/// for output fields, but for input fields in most cases we can only know if the input
/// object is used (through a variable) and not the specific field. We may want to go
/// further in the future and check input object literals in queries, but most of the time,
/// it will be variables.
pub(super) fn remove_field(
    CheckArgs {
        check_params,
        change,
        used_input_types,
        ..
    }: CheckArgs<'_, '_>,
) -> Option<CheckDiagnostic> {
    let (type_name, field_name) = change.path.split_once('.').unwrap();
    let is_input_object = check_params.source.input_objects.contains(type_name);

    if is_input_object {
        let type_is_used = used_input_types
            .get_or_insert_with(|| find_used_input_types(check_params))
            .contains(type_name);

        if !type_is_used {
            return None;
        }

        let field_is_required = check_params
            .source
            .find_field(type_name, field_name)
            .map(|field_id| check_params.source[field_id].is_required())
            .unwrap_or_default();

        let diagnostic = if field_is_required {
            CheckDiagnostic {
                message: format!(
                    "The field `{}` was removed but it is still used by clients.",
                    change.path
                ),
                severity: Severity::Error,
            }
        } else {
            CheckDiagnostic {
                message: format!(
                    "The field `{}` was removed but it may still be used by clients.",
                    change.path
                ),
                severity: Severity::Warning,
            }
        };

        Some(diagnostic)
    } else if check_params.field_is_used(&change.path) {
        Some(CheckDiagnostic {
            message: format!(
                "The field `{}` was removed but it is still used by clients.",
                change.path
            ),
            severity: Severity::Error,
        })
    } else {
        None
    }
}

/// Adding an argument to a field is safe iff the new argument is optional.
pub(super) fn add_field_argument(
    CheckArgs {
        change, check_params, ..
    }: CheckArgs<'_, '_>,
) -> Option<CheckDiagnostic> {
    if !check_params.field_is_used(trim_to_field_path(&change.path)) {
        return None;
    }

    let mut path = change.path.split('.');
    let type_name = path.next().unwrap();
    let field_name = path.next().unwrap();
    let argument_name = path.next().unwrap();

    let argument_id = check_params
        .target
        .find_argument((type_name, field_name, argument_name))
        .expect("Broken invariant: added argument not found in target schema.");

    if !check_params.target[argument_id].is_required_without_default_value() {
        return None;
    }

    Some(CheckDiagnostic {
        message: format!(
            "The new required argument at `{}` would break clients that are not providing it.",
            change.path
        ),
        severity: Severity::Error,
    })
}

/// Adding a field is breaking iff it is a required input object field. The input object has
/// to be used, too.
pub(super) fn add_field(
    CheckArgs {
        change,
        check_params,
        used_input_types,
        ..
    }: CheckArgs<'_, '_>,
) -> Option<CheckDiagnostic> {
    let (type_name, field_name) = change.path.split_once('.').unwrap();

    let Some(field_id) = check_params.target.find_field(type_name, field_name) else {
        return None;
    };

    if !check_params.target[field_id].is_required() {
        return None;
    }

    let type_name = change.path.split_once('.').unwrap().0;

    let used_input_types = used_input_types.get_or_insert_with(|| find_used_input_types(check_params));

    if used_input_types.contains(type_name) {
        Some(CheckDiagnostic {
            message: format!(
                "The new required field at `{}` would break clients that are not providing it.",
                change.path
            ),
            severity: Severity::Error,
        })
    } else {
        None
    }
}

/// Changing the type of an argument or removing an argument is safe iff the argument is not in
/// use or if it was required and became optional (keeping the same inner type).
pub(super) fn change_field_argument_type(
    CheckArgs {
        change, check_params, ..
    }: CheckArgs<'_, '_>,
) -> Option<CheckDiagnostic> {
    if !check_params.argument_is_used(&change.path) {
        return None;
    }

    let (type_name, field_name_and_arg_name) = change.path.split_once('.').unwrap();
    let (field_name, arg_name) = field_name_and_arg_name.split_once('.').unwrap();

    let arg_in_src = check_params.source.find_argument((type_name, field_name, arg_name));
    let arg_in_target = check_params.source.find_argument((type_name, field_name, arg_name));

    'refine: {
        if let Some((src_id, target_id)) = arg_in_src.zip(arg_in_target) {
            let src_arg = &check_params.source[src_id];
            let target_arg = &check_params.target[target_id];

            if src_arg.base_type != target_arg.base_type {
                break 'refine; // type change overrides arity change
            }

            match src_arg.wrappers.compare(&target_arg.wrappers) {
                crate::schema::WrapperTypesComparison::RemovedNonNull => return None,
                crate::schema::WrapperTypesComparison::AddedNonNull => {
                    return Some(CheckDiagnostic {
                        message: format!(
                            "The argument `{}` became required, but clients are not providing it.",
                            change.path
                        ),
                        severity: Severity::Error,
                    });
                }
                crate::schema::WrapperTypesComparison::NoChange
                | crate::schema::WrapperTypesComparison::NotCompatible => break 'refine,
            }
        }
    }

    Some(CheckDiagnostic {
        message: format!("The argument `{}` changed type but it is used by clients.", change.path),
        severity: Severity::Error,
    })
}

pub(super) fn remove_field_argument(
    CheckArgs {
        change, check_params, ..
    }: CheckArgs<'_, '_>,
) -> Option<CheckDiagnostic> {
    if !check_params.argument_is_used(&change.path) {
        return None;
    }

    Some(CheckDiagnostic {
        message: format!(
            "The argument `{}` was removed but it is still used by clients.",
            change.path
        ),
        severity: Severity::Error,
    })
}

/// Changing field types is safe if the field is not in use or:
///
/// - In input fields: if the field was required, and becomes optional.
/// - In output fields: if the field was optional, and becomes required.
pub(super) fn change_field_type(
    CheckArgs {
        change,
        check_params,
        used_input_types,
        ..
    }: CheckArgs<'_, '_>,
) -> Option<CheckDiagnostic> {
    let (type_name, field_name) = change.path.split_once('.').unwrap();
    let source_field_id = check_params.source.find_field(type_name, field_name)?;
    let target_field_id = check_params.target.find_field(type_name, field_name)?;
    let source_field = &check_params.source[source_field_id];
    let target_field = &check_params.target[target_field_id];
    let is_input_object = check_params.source.input_objects.contains(type_name);
    let field_is_used = if is_input_object {
        used_input_types
            .get_or_insert_with(|| find_used_input_types(check_params))
            .contains(type_name)
    } else {
        check_params.field_is_used(&change.path)
    };

    if !field_is_used {
        return None;
    }

    let wrappers_comparison = source_field.wrappers.compare(&target_field.wrappers);

    if source_field.base_type != target_field.base_type
        || matches!(
            wrappers_comparison,
            crate::schema::WrapperTypesComparison::NotCompatible,
        )
    {
        return Some(CheckDiagnostic {
            message: format!(
                "The type of the field `{}` changed from `{}` to `{}`.",
                change.path,
                source_field.render_type(),
                target_field.render_type()
            ),
            severity: Severity::Error,
        });
    }

    match (is_input_object, wrappers_comparison) {
        (true, crate::schema::WrapperTypesComparison::AddedNonNull) => {
            Some(CheckDiagnostic {
                message: format!(
                    "The field `{}` became required, but clients may not be providing it.",
                    change.path
                ),
                severity: Severity::Warning, // warning because we can't tell if they are providing it or not
            })
        }
        (false, crate::schema::WrapperTypesComparison::RemovedNonNull) => Some(CheckDiagnostic {
            message: format!(
                "The field `{}` became optional, but clients do not expect null.",
                change.path
            ),
            severity: Severity::Error,
        }),
        _ => None,
    }
}

/// Removing an `implements` is safe iff there is no inline fragment making use of the
/// implementer on selections on the interface.
pub(super) fn remove_interface_implementation(
    CheckArgs {
        change, check_params, ..
    }: CheckArgs<'_, '_>,
) -> Option<CheckDiagnostic> {
    if !&check_params
        .field_usage
        .type_condition_counts
        .contains_key(&change.path)
    {
        return None;
    }

    Some(CheckDiagnostic {
        message: format!(
            "The interface implementation `{}` was removed but it is still used by clients.",
            change.path
        ),
        severity: Severity::Error,
    })
}

/// Removing an union member is safe iff there is no inline fragment making use of the member on
/// selections on the union.
pub(super) fn remove_union_member(
    CheckArgs {
        change, check_params, ..
    }: CheckArgs<'_, '_>,
) -> Option<CheckDiagnostic> {
    if !&check_params
        .field_usage
        .type_condition_counts
        .contains_key(&change.path)
    {
        return None;
    }

    Some(CheckDiagnostic {
        message: format!(
            "The union member `{}` was removed but it is still used by clients.",
            change.path
        ),
        severity: Severity::Error,
    })
}

/// If you remove a regular object type, it registers as a field type change
/// for the fields that return it. But that isn't the case for the root
/// query, mutation and subscription types, so we need to check for that
/// case separately.
pub(crate) fn remove_object_type(args: CheckArgs<'_, '_>) -> Option<CheckDiagnostic> {
    let type_name = &args.change.path;
    let src = args.check_params.source;

    if ![
        &src.query_type_name,
        &src.mutation_type_name,
        &src.subscription_type_name,
    ]
    .contains(&type_name)
    {
        return None;
    }

    let type_is_used = args.check_params.source.iter_fields(type_name).any(|field| {
        args.check_params
            .field_is_used(&format!("{}.{}", type_name, field.field_name))
    });

    if !type_is_used {
        return None;
    }

    Some(CheckDiagnostic {
        message: format!(
            "The root type `{}` was removed but it is still used by clients.",
            type_name
        ),
        severity: Severity::Error,
    })
}

// Removing an enum value is safe iff it is not used explicitly as argument in any query.
pub(crate) fn remove_enum_value(args: CheckArgs<'_, '_>) -> Option<CheckDiagnostic> {
    if !args.check_params.enum_value_is_used(&args.change.path) {
        return None;
    }

    Some(CheckDiagnostic {
        message: format!(
            "The enum value `{}` was removed but it is still used by clients.",
            args.change.path
        ),
        severity: Severity::Error,
    })
}
