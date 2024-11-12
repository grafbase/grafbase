use cynic_parser_deser::ValueDeserialize;

#[derive(ValueDeserialize)]
pub struct CostDirective {
    pub weight: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directives::parse_directive;

    #[test]
    fn test_parsing_cost() {
        let value = parse_directive::<CostDirective>("@cost(weight: 1)").unwrap();

        assert_eq!(value.weight, 1);
    }
}
