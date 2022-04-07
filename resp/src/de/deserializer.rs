use serde::de::{self, Unexpected};

use crate::parser::{parse_array, parse_bytes, parse_err, parse_int_loose, parse_str_loose};

use super::{Enum, Error, WithLen};

pub struct Deserializer<'de> {
	pub input: &'de [u8],
}

impl<'de> Deserializer<'de> {
	fn parse_error(&mut self) -> Result<&'de str, Error<'de>> {
		let (rem, str) = parse_err(self.input)?;
		self.input = rem;

		Ok(str)
	}

	fn parse_str(&mut self) -> Result<&'de str, Error<'de>> {
		self.check_error()?;

		let (rem, str) = parse_str_loose(self.input)?;
		self.input = rem;

		Ok(str)
	}

	fn parse_int(&mut self) -> Result<i64, Error<'de>> {
		self.check_error()?;

		let (rem, int) = parse_int_loose(self.input)?;
		self.input = rem;

		Ok(int)
	}

	fn parse_bytes(&mut self) -> Result<Option<&'de [u8]>, Error<'de>> {
		self.check_error()?;

		let (rem, bytes) = parse_bytes(self.input)?;
		self.input = rem;

		Ok(bytes)
	}

	fn parse_array(&mut self) -> Result<i64, Error<'de>> {
		self.check_error()?;

		let (rem, len) = parse_array(self.input)?;
		self.input = rem;

		Ok(len)
	}

	fn parse_array_len(&mut self, exp: usize) -> Result<i64, Error<'de>> {
		let len = self.parse_array()?;
		if len != exp.try_into()? {
			Err(de::Error::invalid_length(
				len as usize,
				&format!("an array of length {}", exp).as_str(),
			))
		} else {
			Ok(len)
		}
	}

	fn check_error(&mut self) -> Result<(), Error<'de>> {
		if self.input.get(0).copied() == Some(b'-') {
			Err(Error::RedisError(self.parse_error()?))
		} else {
			Ok(())
		}
	}
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
	type Error = Error<'de>;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self.input.get(0) {
			Some(b'+') => self.deserialize_str(visitor),
			Some(b'-') => Err(Error::RedisError(self.parse_error()?)),
			Some(b':') => self.deserialize_i64(visitor),
			Some(b'$') => self.deserialize_bytes(visitor),
			Some(b'*') => self.deserialize_seq(visitor),
			Some(b) => Err(de::Error::invalid_value(
				Unexpected::Unsigned(*b as u64),
				&"a valid RESP prefix",
			)),
			None => Err(de::Error::invalid_length(0, &"length of at least 1")),
		}
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_bool(self.parse_str()?.parse()?)
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_i8(self.parse_int()?.try_into()?)
	}

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_i16(self.parse_int()?.try_into()?)
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_i32(self.parse_int()?.try_into()?)
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_i64(self.parse_int()?)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_u8(self.parse_int()?.try_into()?)
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_u16(self.parse_int()?.try_into()?)
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_u32(self.parse_int()?.try_into()?)
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_u64(self.parse_int()?.try_into()?)
	}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_f32(self.parse_str()?.parse()?)
	}

	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_f64(self.parse_str()?.parse()?)
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_char(self.parse_str()?.parse()?)
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_borrowed_str(self.parse_str()?)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self.parse_bytes()? {
			Some(d) => visitor.visit_borrowed_bytes(d),
			None => visitor.visit_none(),
		}
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_bytes(visitor)
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		match self.parse_bytes() {
			Ok(Some(b)) => visitor.visit_borrowed_bytes(b),
			Ok(None) => visitor.visit_none(),
			Err(_) => match self.parse_array() {
				Ok(-1) => visitor.visit_none(),
				Ok(_) => visitor.visit_some(self),
				Err(e) => Err(e),
			},
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.check_error()?;
		visitor.visit_none()
	}

	fn deserialize_unit_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_unit(visitor)
	}

	fn deserialize_newtype_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.check_error()?;
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		let len = self.parse_array()?;

		visitor.visit_seq(WithLen {
			de: self,
			cur: 0,
			len,
		})
	}

	fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		let len = self.parse_array_len(len)?;

		visitor.visit_seq(WithLen {
			de: self,
			cur: 0,
			len,
		})
	}

	fn deserialize_tuple_struct<V>(
		self,
		_name: &'static str,
		len: usize,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		let len = self.parse_array_len(len)?;

		visitor.visit_seq(WithLen {
			de: self,
			cur: 0,
			len,
		})
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		let len = self.parse_array()?;

		visitor.visit_map(WithLen {
			de: self,
			cur: 0,
			len: len / 2,
		})
	}

	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_map(visitor)
	}

	fn deserialize_enum<V>(
		self,
		_name: &'static str,
		_variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_enum(Enum { de: self })
	}

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_any(visitor)
	}
}
