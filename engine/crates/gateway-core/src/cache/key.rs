use std::{
    collections::BTreeSet,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use common_types::auth::ExecutionAuth;
use engine_value::ConstValue;

#[derive(Debug, Hash)]
pub enum CacheAccess<'a> {
    Scoped(BTreeSet<String>),
    Default(&'a ExecutionAuth),
}

#[derive(Debug)]
pub struct CacheKey<'a, H: Hasher + Default> {
    access: CacheAccess<'a>,
    gql_request: &'a engine::Request,
    subdomain: &'a str,
    _hasher_builder: PhantomData<H>,
}

impl<'a, H: Hasher + Default> CacheKey<'a, H> {
    pub fn new(access: CacheAccess<'a>, gql_request: &'a engine::Request, subdomain: &'a str) -> Self {
        CacheKey {
            access,
            gql_request,
            subdomain,
            _hasher_builder: PhantomData,
        }
    }

    pub fn to_hash_string(&self) -> String {
        let mut hasher = H::default();
        self.hash(&mut hasher);
        hasher.finish().to_string()
    }
}

impl<HB: Hasher + Default> Hash for CacheKey<'_, HB> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.gql_request.query.hash(state);
        self.gql_request.operation_name.hash(state);
        self.subdomain.hash(state);

        fn hash_const_value<H: Hasher + Default>(value: &ConstValue) -> u64 {
            let mut hasher = H::default();
            match value {
                ConstValue::Object(map) => {
                    hasher.write_usize(map.len());
                    let mut inner_hash: u64 = 0;
                    // opted for XORing the hashes of the elements instead of sorting the keys
                    for (name, value) in map {
                        let mut inner_hasher = H::default();
                        name.hash(&mut inner_hasher);
                        inner_hasher.write_u64(hash_const_value::<H>(value));
                        inner_hash ^= inner_hasher.finish();
                    }
                    hasher.write_u64(inner_hash);
                }
                // Since ConstValue::Number is a enum for 64 variations it will have a different memory representation than 0_u8.
                // The hash of u64 and u8 relies on the method to_ne_bytes() which returns the memory representation.
                // Hashing memory representations of different sizes will yield different hashes.
                // Therefore, its safe to rely on 0_u8 here
                ConstValue::Null => 0_u8.hash(&mut hasher),
                ConstValue::Number(n) => n.hash(&mut hasher),
                ConstValue::String(s) => s.hash(&mut hasher),
                ConstValue::Boolean(b) => b.hash(&mut hasher),
                ConstValue::Binary(bin) => bin.hash(&mut hasher),
                ConstValue::Enum(e) => e.hash(&mut hasher),
                ConstValue::List(l) => {
                    hasher.write_usize(l.len());
                    // not XORing or sorting on purpose
                    // e.g: orderBy: [NAME, EMAIL] != orderBy: [EMAIL, NAME]
                    for v in l {
                        hasher.write_u64(hash_const_value::<H>(v));
                    }
                }
            }

            hasher.finish()
        }

        // hash request variables
        state.write_usize(self.gql_request.variables.len());
        // variables is a BTreeMap behind the scenes so ordering is guaranteed
        // simply hashing its elements in the main hasher is enough
        for (name, value) in &*self.gql_request.variables {
            name.hash(state);
            state.write_u64(hash_const_value::<HB>(value));
        }

        // hash access
        self.access.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_types::auth::{ExecutionAuth, Operations};
    use engine::indexmap::IndexMap;
    use engine_value::{ConstValue, Name, Variables};
    use std::collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet};

    #[test]
    fn should_have_equal_cache_key_hashes_when_request_variables_are_equal_with_different_ordering() {
        // prepare
        let auth = ExecutionAuth::new_from_token(
            Operations::all(),
            BTreeSet::from(["test".to_string()]),
            None,
            BTreeMap::new(),
        );
        let test_subdomain = "test-subdomain".to_string();
        let variable_1 = ConstValue::List(vec![
            ConstValue::Enum(Name::new("hello")),
            ConstValue::Object(IndexMap::from([(Name::new("hash_fun"), ConstValue::Null)])),
        ]);
        let variable_2 = ConstValue::String("hello".to_string());

        let gql_request =
            engine::Request::new("{ query { test { id } } }").variables(Variables::from_value(ConstValue::List(vec![
                variable_1.clone(),
                variable_2.clone(),
            ])));

        let gql_request_2 = engine::Request::new("{ query { test { id } } }")
            .variables(Variables::from_value(ConstValue::List(vec![variable_2, variable_1])));

        let cache_key = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request, &test_subdomain);
        let cache_key_2 = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request_2, &test_subdomain);

        assert_eq!(cache_key.to_hash_string(), cache_key_2.to_hash_string());
    }

    #[test]
    fn should_have_equal_cache_key_hashes_when_request_variables_have_equal_lists() {
        // prepare
        let auth = ExecutionAuth::new_from_token(
            Operations::all(),
            BTreeSet::from(["test".to_string()]),
            None,
            BTreeMap::new(),
        );
        let test_subdomain = "test-subdomain".to_string();
        let variable_list = ConstValue::List(vec![
            ConstValue::Object(IndexMap::from([(Name::new("hash_fun"), ConstValue::Null)])),
            ConstValue::Boolean(false),
            ConstValue::Enum(Name::new("hello")),
        ]);

        let gql_request = engine::Request::new("{ query { test { id } } }").variables(Variables::from_value(
            ConstValue::Object(IndexMap::from([(Name::new("test"), variable_list.clone())])),
        ));

        let gql_request_2 = engine::Request::new("{ query { test { id } } }").variables(Variables::from_value(
            ConstValue::Object(IndexMap::from([(Name::new("test"), variable_list)])),
        ));

        let cache_key = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request, &test_subdomain);
        let cache_key_2 = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request_2, &test_subdomain);

        assert_eq!(cache_key.to_hash_string(), cache_key_2.to_hash_string());
    }

    #[test]
    fn should_not_have_equal_cache_key_hashes_when_request_variables_have_equal_lists_with_different_ordering() {
        // prepare
        let auth = ExecutionAuth::new_from_token(
            Operations::all(),
            BTreeSet::from(["test".to_string()]),
            None,
            BTreeMap::new(),
        );
        let test_subdomain = "test-subdomain".to_string();

        let gql_request = engine::Request::new("{ query { test { id } } }").variables(Variables::from_value(
            ConstValue::Object(IndexMap::from([(
                Name::new("test"),
                ConstValue::List(vec![
                    ConstValue::Enum(Name::new("hello")),
                    ConstValue::Object(IndexMap::from([(Name::new("hash_fun"), ConstValue::Null)])),
                    ConstValue::Boolean(true),
                ]),
            )])),
        ));

        let gql_request_2 = engine::Request::new("{ query { test { id } } }").variables(Variables::from_value(
            ConstValue::Object(IndexMap::from([(
                Name::new("test"),
                ConstValue::List(vec![
                    ConstValue::Object(IndexMap::from([(Name::new("hash_fun"), ConstValue::Null)])),
                    ConstValue::Boolean(true),
                    ConstValue::Enum(Name::new("hello")),
                ]),
            )])),
        ));

        let cache_key = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request, &test_subdomain);
        let cache_key_2 = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request_2, &test_subdomain);

        assert_ne!(cache_key.to_hash_string(), cache_key_2.to_hash_string());
    }

    #[test]
    fn should_have_equal_cache_key_hashes_when_request_variables_have_equal_maps_with_different_ordering() {
        let auth = ExecutionAuth::new_from_token(
            Operations::all(),
            BTreeSet::from(["test".to_string()]),
            None,
            BTreeMap::new(),
        );
        let test_subdomain = "test-subdomain".to_string();

        let gql_request = engine::Request::new("{ query { test { id } } }").variables(Variables::from_value(
            ConstValue::Object(IndexMap::from([
                (
                    Name::new("test"),
                    ConstValue::List(vec![ConstValue::Enum(Name::new("hello"))]),
                ),
                (
                    Name::new("test_2"),
                    ConstValue::List(vec![ConstValue::Enum(Name::new("hello"))]),
                ),
            ])),
        ));

        let gql_request_2 = engine::Request::new("{ query { test { id } } }").variables(Variables::from_value(
            ConstValue::Object(IndexMap::from([
                (
                    Name::new("test_2"),
                    ConstValue::List(vec![ConstValue::Enum(Name::new("hello"))]),
                ),
                (
                    Name::new("test"),
                    ConstValue::List(vec![ConstValue::Enum(Name::new("hello"))]),
                ),
            ])),
        ));

        let cache_key = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request, &test_subdomain);
        let cache_key_2 = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request_2, &test_subdomain);

        assert_eq!(cache_key.to_hash_string(), cache_key_2.to_hash_string());
    }

    #[test]
    fn should_not_have_equal_cache_keys_hashes_due_to_query() {
        // prepare
        let auth = ExecutionAuth::new_from_token(
            Operations::all(),
            BTreeSet::from(["test".to_string()]),
            None,
            BTreeMap::new(),
        );
        let test_subdomain = "test-subdomain".to_string();
        let gql_variables = Variables::from_value(ConstValue::Object(IndexMap::from([(
            Name::new("test"),
            ConstValue::Null,
        )])));

        let gql_request = engine::Request::new("{ query { test { id, id2 } } }").variables(gql_variables.clone());

        let gql_request_2 = engine::Request::new("{ query { test { id, name } } }").variables(gql_variables);

        let cache_key = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request, &test_subdomain);
        let cache_key_2 = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request_2, &test_subdomain);

        assert_ne!(cache_key.to_hash_string(), cache_key_2.to_hash_string());
    }

    #[test]
    fn should_have_equal_cache_key_hashes_when_auth_groups_are_equal_with_different_ordering() {
        let auth = ExecutionAuth::new_from_token(
            Operations::all(),
            BTreeSet::from(["test".to_string(), "test_2".to_string()]),
            Some(("test".to_string(), Operations::all())),
            BTreeMap::new(),
        );
        let auth_2 = ExecutionAuth::new_from_token(
            Operations::all(),
            BTreeSet::from(["test_2".to_string(), "test".to_string()]),
            Some(("test".to_string(), Operations::all())),
            BTreeMap::new(),
        );
        let test_subdomain = "test-subdomain".to_string();

        let gql_request = engine::Request::new("{ query { test { id } } }")
            .variables(Variables::from_value(ConstValue::Enum(Name::new("hello"))));

        let cache_key = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request, &test_subdomain);
        let cache_key_2 = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth_2), &gql_request, &test_subdomain);

        assert_eq!(cache_key.to_hash_string(), cache_key_2.to_hash_string());
    }

    #[test]
    fn should_not_have_equal_cache_keys_hashes_due_to_domain() {
        let auth = ExecutionAuth::new_from_token(
            Operations::all(),
            BTreeSet::from(["test".to_string()]),
            Some(("test".to_string(), Operations::all())),
            BTreeMap::new(),
        );
        let test_subdomain_1 = "test-subdomain".to_string();
        let test_subdomain_2 = "test-subdomain-2".to_string();
        let gql_variables = Variables::from_value(ConstValue::Object(IndexMap::from([(
            Name::new("test"),
            ConstValue::Null,
        )])));

        let gql_request = engine::Request::new("{ query { test { id, id2 } } }").variables(gql_variables.clone());

        let gql_request_2 = engine::Request::new("{ query { test { id, name } } }").variables(gql_variables);

        let cache_key = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request, &test_subdomain_1);
        let cache_key_2 =
            CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request_2, &test_subdomain_2);

        assert_ne!(cache_key.to_hash_string(), cache_key_2.to_hash_string());
    }

    #[test]
    fn should_not_have_equal_cache_keys_hashes_when_using_null_and_0() {
        let gql_query = "{ query { test { id } } }";
        let auth = ExecutionAuth::new_from_token(
            Operations::all(),
            BTreeSet::from(["test".to_string()]),
            None,
            BTreeMap::new(),
        );
        let test_subdomain = "test-subdomain".to_string();

        let gql_variables = Variables::from_value(ConstValue::Object(IndexMap::from([(
            Name::new("test"),
            ConstValue::Null,
        )])));

        let gql_variables_2 = Variables::from_value(ConstValue::Object(IndexMap::from([(
            Name::new("test"),
            ConstValue::Number(engine::Number::from(0)),
        )])));

        let gql_request = engine::Request::new(gql_query.to_string()).variables(gql_variables);
        let gql_request_2 = engine::Request::new(gql_query.to_string()).variables(gql_variables_2);

        let cache_key = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request, &test_subdomain);
        let cache_key_2 = CacheKey::<DefaultHasher>::new(CacheAccess::Default(&auth), &gql_request_2, &test_subdomain);

        assert_ne!(cache_key.to_hash_string(), cache_key_2.to_hash_string());
    }
}
