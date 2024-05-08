mod arguments_of_correct_type;
mod default_values_of_correct_type;
mod directives_unique;
mod fields_on_correct_type;
mod fragments_on_composite_types;
mod known_argument_names;
mod known_directives;
mod known_fragment_names;
mod known_type_names;
mod no_fragment_cycles;
mod no_undefined_variables;
mod no_unused_fragments;
mod no_unused_variables;
mod overlapping_fields_can_be_merged;
mod possible_fragment_spreads;
mod provided_non_null_arguments;
mod scalar_leafs;
mod unique_argument_names;
mod unique_variable_names;
mod variables_are_input_types;
mod variables_in_allowed_position;

pub use arguments_of_correct_type::ArgumentsOfCorrectType;
pub use default_values_of_correct_type::DefaultValuesOfCorrectType;
pub use directives_unique::DirectivesUnique;
pub use fields_on_correct_type::FieldsOnCorrectType;
pub use fragments_on_composite_types::FragmentsOnCompositeTypes;
pub use known_argument_names::KnownArgumentNames;
pub use known_directives::KnownDirectives;
pub use known_fragment_names::KnownFragmentNames;
pub use known_type_names::KnownTypeNames;
pub use no_fragment_cycles::NoFragmentCycles;
pub use no_undefined_variables::NoUndefinedVariables;
pub use no_unused_fragments::NoUnusedFragments;
pub use no_unused_variables::NoUnusedVariables;
pub use overlapping_fields_can_be_merged::OverlappingFieldsCanBeMerged;
pub use possible_fragment_spreads::PossibleFragmentSpreads;
pub use provided_non_null_arguments::ProvidedNonNullArguments;
pub use scalar_leafs::ScalarLeafs;
pub use unique_argument_names::UniqueArgumentNames;
pub use unique_variable_names::UniqueVariableNames;
pub use variables_are_input_types::VariablesAreInputTypes;
pub use variables_in_allowed_position::VariableInAllowedPosition;

fn concrete_type_name_from_parsed_type(query_type: &engine_parser::types::Type) -> &str {
    match &query_type.base {
        engine_parser::types::BaseType::Named(name) => name.as_str(),
        engine_parser::types::BaseType::List(ty) => concrete_type_name_from_parsed_type(ty),
    }
}
