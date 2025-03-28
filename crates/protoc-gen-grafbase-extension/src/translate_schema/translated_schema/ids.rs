use super::*;
use paste::paste;

macro_rules! id_types (
    ($($field:ident[$id_type:ident] -> $record_type:ty,)*) => {
        $(
            mod $field {
                #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
                pub(crate) struct $id_type(usize);

                impl From<usize> for $id_type {
                    fn from(value: usize) -> Self {
                        $id_type(value)
                    }
                }

                impl From<$id_type> for usize {
                    fn from(value: $id_type) -> Self {
                        value.0
                    }
                }
            }

            pub(crate) use $field::*;

            impl std::ops::Index<$id_type> for TranslatedSchema {
                type Output = $record_type;

                fn index(&self, index: $id_type) -> &Self::Output {
                    &self.$field[usize::from(index)]
                }
            }


            impl TranslatedSchema {
                paste! {
                    pub(crate) fn [<push_ $field>](&mut self, record: $record_type) -> $id_type {
                        let idx = self.$field.push(record);
                        $id_type::from(idx)
                    }

                    #[allow(unused)]
                    pub(crate) fn [<iter_ $field>](&self) -> impl Iterator<Item = View<'_, $id_type, $record_type>> {
                        self.$field.iter().enumerate().map(|(id, record)| View { id: $id_type::from(id), record })
                    }
                }
            }
        )*
    };
);

id_types! {
    packages[ProtoPackageId] -> ProtoPackage,

    messages[ProtoMessageId] -> ProtoMessage,
    fields[ProtoFieldId] -> ProtoField,

    enums[ProtoEnumId] -> ProtoEnum,

    services[ProtoServiceId] -> ProtoService,
    methods[ProtoMethodId] -> ProtoMethod,
}

impl ProtoPackageId {
    pub(crate) fn services(
        self,
        schema: &TranslatedSchema,
    ) -> impl Iterator<Item = View<'_, ProtoServiceId, ProtoService>> {
        schema.services.iter_with_prefix(self, |service| service.package_id)
    }
}

impl ProtoServiceId {
    pub(crate) fn methods(
        self,
        schema: &TranslatedSchema,
    ) -> impl Iterator<Item = View<'_, ProtoMethodId, ProtoMethod>> {
        schema.methods.iter_with_prefix(self, |method| method.service_id)
    }
}

impl ProtoMessageId {
    pub(crate) fn fields(self, schema: &TranslatedSchema) -> impl Iterator<Item = View<'_, ProtoFieldId, ProtoField>> {
        schema.fields.iter_with_prefix(self, |field| field.message_id)
    }
}
