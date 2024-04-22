use std::collections::{BTreeMap, HashMap};

use anyhow::anyhow;
use gateway_v2_auth_config::v1::AuthConfig;
use indexmap::IndexSet;
use postgres_connector_types::database_definition::DatabaseDefinition;

use crate::{
    ids::*,
    resolvers::Resolver,
    storage::{self, *},
    ConnectorHeaders, CorsConfig, FederationEntity, IdRange, MongoDBConfiguration, OperationLimits, Registry,
    TrustedDocuments, TypeWrappers,
};

/// Writes to a registry.
///
/// Note that this is a very low level interface.  We'd want some sort of friendly builders
/// built on top of this if we were ever to use it directly in parsers
#[derive(Default)]
pub struct RegistryWriter {
    strings: IndexSet<Box<str>>,

    // Optional so we can preallocate
    types: Vec<Option<storage::MetaTypeRecord>>,

    objects: Vec<storage::ObjectTypeRecord>,
    object_fields: Vec<storage::MetaFieldRecord>,

    input_objects: Vec<storage::InputObjectTypeRecord>,
    input_values: Vec<storage::MetaInputValueRecord>,
    input_validators: Vec<storage::InputValidatorRecord>,

    enums: Vec<storage::EnumTypeRecord>,
    enum_values: Vec<storage::MetaEnumValueRecord>,

    interfaces: Vec<storage::InterfaceTypeRecord>,
    scalars: Vec<storage::ScalarTypeRecord>,
    unions: Vec<storage::UnionTypeRecord>,

    directives: Vec<storage::MetaDirectiveRecord>,

    pub implements: HashMap<MetaTypeId, Vec<MetaTypeId>>,
    pub query_type: Option<MetaTypeId>,
    pub mutation_type: Option<MetaTypeId>,
    pub subscription_type: Option<MetaTypeId>,
    pub disable_introspection: bool,
    pub enable_federation: bool,
    pub federation_subscription: bool,

    pub auth: AuthConfig,
    pub mongodb_configurations: HashMap<String, MongoDBConfiguration>,
    pub http_headers: BTreeMap<String, ConnectorHeaders>,
    pub postgres_databases: HashMap<String, DatabaseDefinition>,
    pub enable_caching: bool,
    pub enable_kv: bool,
    pub federation_entities: BTreeMap<String, FederationEntity>,
    pub enable_codegen: bool,
    pub is_federated: bool,
    pub operation_limits: OperationLimits,
    pub trusted_documents: Option<TrustedDocuments>,
    pub cors_config: Option<CorsConfig>,
}

impl RegistryWriter {
    pub fn new() -> Self {
        RegistryWriter::default()
    }

    pub fn preallocate_type_ids(&mut self, capacity: usize) -> impl ExactSizeIterator<Item = MetaTypeId> {
        let starting_id = MetaTypeId::new(self.types.len());
        self.types.extend(std::iter::repeat_with(|| None).take(capacity));

        IdRange::new(starting_id, MetaTypeId::new(self.types.len())).iter()
    }

    pub fn populate_preallocated_type(&mut self, id: MetaTypeId, record: MetaTypeRecord) {
        let index = id.to_index();
        if self.types[index].is_some() {
            panic!("Tried to populate an already populated index");
        }
        self.types[index] = Some(record);
    }

    #[must_use]
    pub fn insert_scalar(&mut self, details: ScalarTypeRecord) -> MetaTypeRecord {
        let id = ScalarTypeId::new(self.scalars.len());
        self.scalars.push(details);
        MetaTypeRecord::Scalar(id)
    }

    #[must_use]
    pub fn insert_object(&mut self, details: ObjectTypeRecord) -> MetaTypeRecord {
        let id = ObjectTypeId::new(self.objects.len());
        self.objects.push(details);
        MetaTypeRecord::Object(id)
    }

    #[must_use]
    pub fn insert_interface(&mut self, details: InterfaceTypeRecord) -> MetaTypeRecord {
        let id = InterfaceTypeId::new(self.interfaces.len());
        self.interfaces.push(details);
        MetaTypeRecord::Interface(id)
    }

    #[must_use]
    pub fn insert_union(&mut self, details: UnionTypeRecord) -> MetaTypeRecord {
        let id = UnionTypeId::new(self.unions.len());
        self.unions.push(details);
        MetaTypeRecord::Union(id)
    }

    #[must_use]
    pub fn insert_enum(&mut self, details: EnumTypeRecord) -> MetaTypeRecord {
        let id = EnumTypeId::new(self.enums.len());
        self.enums.push(details);
        MetaTypeRecord::Enum(id)
    }

    #[must_use]
    pub fn insert_enum_values(&mut self, mut values: Vec<MetaEnumValueRecord>) -> IdRange<MetaEnumValueId> {
        let starting_id = MetaEnumValueId::new(self.enum_values.len());

        // Sort the values so we can binary search later
        values.sort_by_key(|val| &self.strings[val.name.to_index()]);

        self.enum_values.append(&mut values);

        IdRange::new(starting_id, MetaEnumValueId::new(self.enum_values.len()))
    }

    #[must_use]
    pub fn insert_input_object(&mut self, details: InputObjectTypeRecord) -> MetaTypeRecord {
        let id = InputObjectTypeId::new(self.input_objects.len());
        self.input_objects.push(details);
        MetaTypeRecord::InputObject(id)
    }

    #[must_use]
    pub fn insert_fields(&mut self, mut fields: Vec<MetaFieldRecord>) -> IdRange<MetaFieldId> {
        let starting_id = MetaFieldId::new(self.object_fields.len());

        // Sort the fields so we can binary search later
        fields.sort_by_key(|val| &self.strings[val.name.to_index()]);

        self.object_fields.append(&mut fields);

        IdRange::new(starting_id, MetaFieldId::new(self.object_fields.len()))
    }

    #[must_use]
    pub fn insert_input_values(&mut self, mut values: Vec<MetaInputValueRecord>) -> IdRange<MetaInputValueId> {
        let starting_id = MetaInputValueId::new(self.input_values.len());

        // Sort the values so we can binary search later
        values.sort_by_key(|val| &self.strings[val.name.to_index()]);

        self.input_values.append(&mut values);

        IdRange::new(starting_id, MetaInputValueId::new(self.input_values.len()))
    }

    #[must_use]
    pub fn insert_input_validators(&mut self, mut values: Vec<InputValidatorRecord>) -> IdRange<InputValidatorId> {
        let starting_id = InputValidatorId::new(self.input_validators.len());

        self.input_validators.append(&mut values);

        IdRange::new(starting_id, InputValidatorId::new(self.input_validators.len()))
    }

    pub fn insert_directive(&mut self, details: MetaDirectiveRecord) -> MetaDirectiveId {
        let id = MetaDirectiveId::new(self.directives.len());
        self.directives.push(details);
        id
    }

    #[must_use]
    pub fn intern_str(&mut self, string: &str) -> StringId {
        let (id, _) = self.strings.insert_full(string.into());
        StringId::new(id)
    }

    #[must_use]
    pub fn intern_string(&mut self, string: String) -> StringId {
        let (id, _) = self.strings.insert_full(string.into());
        StringId::new(id)
    }

    pub fn finish(mut self) -> anyhow::Result<Registry> {
        let typename_index = self.insert_typename_field();

        let RegistryWriter {
            strings,
            types,
            objects,
            object_fields,
            input_objects,
            input_values,
            input_validators,
            enums,
            enum_values,
            interfaces,
            scalars,
            unions,
            directives,
            implements,
            query_type,
            mutation_type,
            subscription_type,
            disable_introspection,
            enable_federation,
            federation_subscription,
            auth,
            mongodb_configurations,
            http_headers,
            postgres_databases,
            enable_caching,
            enable_kv,
            federation_entities,
            enable_codegen,
            is_federated,
            operation_limits,
            trusted_documents,
            cors_config,
        } = self;

        let types = types
            .into_iter()
            .map(|ty| ty.ok_or_else(|| anyhow!("All preallocated types must be allocated")))
            .collect::<Result<Vec<_>, _>>()?;

        let query_type = query_type.ok_or_else(|| anyhow!("Root query type was not defined"))?;

        Ok(Registry {
            strings,
            types,
            objects,
            object_fields,
            input_objects,
            input_values,
            input_validators,
            enums,
            enum_values,
            interfaces,
            scalars,
            unions,
            directives,
            typename_index,
            implements,
            query_type,
            mutation_type,
            subscription_type,
            disable_introspection,
            enable_federation,
            federation_subscription,
            auth,
            mongodb_configurations,
            http_headers,
            postgres_databases,
            enable_caching,
            enable_kv,
            federation_entities,
            enable_codegen,
            is_federated,
            operation_limits,
            trusted_documents,
            cors_config,
        })
    }

    // __typename doesn't usually exist in the schema, but it's handy if we have a
    // MetaFieldRecord for it, so we insert a fake field here and don't associated
    // it with any particular object
    fn insert_typename_field(&mut self) -> MetaFieldId {
        let name = self.intern_str("__typename");
        let index = self
            .types
            .binary_search_by_key(&"String", |key| {
                self.meta_type_name(key.as_ref().expect("to be prepopulated"))
            })
            .expect("String to be a defined type");
        let string = MetaTypeId::new(index);
        let ty = MetaFieldTypeRecord {
            wrappers: TypeWrappers::none().wrap_non_null(),
            target: string,
        };

        let range = self.insert_fields(vec![MetaFieldRecord {
            name,
            mapped_name: None,
            description: None,
            args: IdRange::default(),
            ty,
            deprecation: None,
            cache_control: None,
            requires: None,
            federation: None,
            resolver: Resolver::Typename,
            required_operation: None,
            auth: None,
        }]);

        range.start
    }

    #[allow(dead_code)]
    fn meta_type_name_by_id(&self, id: MetaTypeId) -> &str {
        self.meta_type_name(
            self.types[id.to_index()]
                .as_ref()
                .expect("to be preopulated before this call"),
        )
    }

    fn meta_type_name(&self, record: &MetaTypeRecord) -> &str {
        let string_id = match record {
            MetaTypeRecord::Object(inner) => self.objects[inner.to_index()].name,
            MetaTypeRecord::Interface(inner) => self.interfaces[inner.to_index()].name,
            MetaTypeRecord::Union(inner) => self.unions[inner.to_index()].name,
            MetaTypeRecord::InputObject(inner) => self.input_objects[inner.to_index()].name,
            MetaTypeRecord::Enum(inner) => self.enums[inner.to_index()].name,
            MetaTypeRecord::Scalar(inner) => self.scalars[inner.to_index()].name,
        };

        &self.strings[string_id.to_index()]
    }
}
