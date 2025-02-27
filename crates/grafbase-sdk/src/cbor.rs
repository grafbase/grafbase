//! Remove this when this PR is merged and a new version of minicbor-serde is released.
//! https://github.com/twittner/minicbor/pull/20

use minicbor::encode::{Encoder, Write};
use minicbor_serde::error::EncodeError;
use serde::ser::{self, SerializeSeq, SerializeTuple, SerializeTupleStruct};
use serde::ser::{SerializeMap, SerializeStruct, SerializeStructVariant, SerializeTupleVariant};
use serde::Serialize;

/// Serialise a type implementing [`serde::Serialize`] and return the encoded byte vector.
pub fn to_vec<T: Serialize>(val: T) -> Result<Vec<u8>, EncodeError<core::convert::Infallible>> {
    let mut v = Vec::new();
    val.serialize(&mut Serializer::new(&mut v))?;
    Ok(v)
}

/// An implementation of [`serde::Serializer`] using a [`minicbor::Encoder`].
#[derive(Debug, Clone)]
pub struct Serializer<W> {
    encoder: Encoder<W>,
}

impl<W: Write> Serializer<W> {
    pub fn new(w: W) -> Self {
        Self::from(Encoder::new(w))
    }

    pub fn encoder(&self) -> &Encoder<W> {
        &self.encoder
    }

    pub fn encoder_mut(&mut self) -> &mut Encoder<W> {
        &mut self.encoder
    }

    pub fn into_encoder(self) -> Encoder<W> {
        self.encoder
    }
}

impl<W: Write> From<Encoder<W>> for Serializer<W> {
    fn from(e: Encoder<W>) -> Self {
        Self { encoder: e }
    }
}

impl<'a, W: Write> ser::Serializer for &'a mut Serializer<W>
where
    <W as Write>::Error: core::error::Error + 'static,
{
    type Ok = ();
    type Error = EncodeError<W::Error>;

    type SerializeSeq = SeqSerializer<'a, W>;
    type SerializeTuple = SeqSerializer<'a, W>;
    type SerializeTupleStruct = SeqSerializer<'a, W>;
    type SerializeTupleVariant = SeqSerializer<'a, W>;
    type SerializeMap = SeqSerializer<'a, W>;
    type SerializeStruct = SeqSerializer<'a, W>;
    type SerializeStructVariant = SeqSerializer<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.encoder.bool(v)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.encoder.i8(v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.encoder.i16(v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.encoder.i32(v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.encoder.i64(v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.encoder.u8(v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.encoder.u16(v)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.encoder.u32(v)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.encoder.u64(v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.encoder.f32(v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.encoder.f64(v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.encoder.char(v)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.encoder.str(v)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.encoder.bytes(v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.null()?;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.encoder.null()?;

        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        variant.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.encoder.map(1)?.str(variant)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if let Some(n) = len {
            self.encoder.array(n as u64)?;
        } else {
            self.encoder.begin_array()?;
        }
        Ok(SeqSerializer {
            serializer: self,
            indefinite: len.is_none(),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.encoder.array(len as u64)?;
        Ok(SeqSerializer {
            serializer: self,
            indefinite: false,
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.encoder.map(1)?.str(variant)?;
        self.serialize_tuple(len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        if let Some(n) = len {
            self.encoder.map(n as u64)?;
        } else {
            self.encoder.begin_map()?;
        }
        Ok(SeqSerializer {
            serializer: self,
            indefinite: len.is_none(),
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        self.encoder.map(len as u64)?;
        Ok(SeqSerializer {
            serializer: self,
            indefinite: false,
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.encoder.map(1)?.str(variant)?;
        self.serialize_struct(name, len)
    }

    fn collect_str<T: core::fmt::Display + ?Sized>(self, _val: &T) -> Result<Self::Ok, Self::Error> {
        Err(minicbor::encode::Error::message("collect_str requires features `alloc` or `std`").into())
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

pub struct SeqSerializer<'a, W> {
    serializer: &'a mut Serializer<W>,
    indefinite: bool,
}

impl<W: Write> SerializeSeq for SeqSerializer<'_, W>
where
    <W as Write>::Error: core::error::Error + 'static,
{
    type Ok = ();
    type Error = minicbor_serde::error::EncodeError<W::Error>;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, x: &T) -> Result<(), Self::Error> {
        x.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.indefinite {
            self.serializer.encoder.end()?;
        }
        Ok(())
    }
}

impl<W: Write> SerializeTuple for SeqSerializer<'_, W>
where
    <W as Write>::Error: core::error::Error + 'static,
{
    type Ok = ();
    type Error = minicbor_serde::error::EncodeError<W::Error>;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, x: &T) -> Result<(), Self::Error> {
        x.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: Write> SerializeTupleStruct for SeqSerializer<'_, W>
where
    <W as Write>::Error: core::error::Error + 'static,
{
    type Ok = ();
    type Error = EncodeError<W::Error>;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, x: &T) -> Result<(), Self::Error> {
        x.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: Write> SerializeTupleVariant for SeqSerializer<'_, W>
where
    <W as Write>::Error: core::error::Error + 'static,
{
    type Ok = ();
    type Error = EncodeError<W::Error>;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, x: &T) -> Result<(), Self::Error> {
        x.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: Write> SerializeMap for SeqSerializer<'_, W>
where
    <W as Write>::Error: core::error::Error + 'static,
{
    type Ok = ();
    type Error = EncodeError<W::Error>;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, k: &T) -> Result<(), Self::Error> {
        k.serialize(&mut *self.serializer)
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Self::Error> {
        v.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.indefinite {
            self.serializer.encoder.end()?;
        }
        Ok(())
    }
}

impl<W: Write> SerializeStruct for SeqSerializer<'_, W>
where
    <W as Write>::Error: core::error::Error + 'static,
{
    type Ok = ();
    type Error = EncodeError<W::Error>;

    fn serialize_field<T>(&mut self, key: &'static str, val: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        key.serialize(&mut *self.serializer)?;
        val.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: Write> SerializeStructVariant for SeqSerializer<'_, W>
where
    <W as Write>::Error: core::error::Error + 'static,
{
    type Ok = ();
    type Error = EncodeError<W::Error>;

    fn serialize_field<T>(&mut self, key: &'static str, val: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        key.serialize(&mut *self.serializer)?;
        val.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
