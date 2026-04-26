use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};
use thiserror::Error;
use uuid::Uuid;

pub trait Entity {
    fn key_name() -> &'static str;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<T: Entity> {
    value: Uuid,
    _marker: PhantomData<T>,
}

impl<T: Entity> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Entity> Copy for Id<T> {}

impl<T: Entity> Id<T> {
    pub fn new() -> Self {
        Self {
            value: Uuid::new_v4(),
            _marker: PhantomData,
        }
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self {
            value: uuid,
            _marker: PhantomData,
        }
    }

    pub fn parse(input: &str) -> Result<Self, IdError<T>> {
        Uuid::from_str(input)
            .map(Self::from_uuid)
            .map_err(|_| IdError::Invalid {
                entity: T::key_name(),
                value: input.to_string(),
                _marker: PhantomData,
            })
    }
}

impl<T: Entity> FromStr for Id<T> {
    type Err = IdError<T>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl<T: Entity> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T: Entity> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de, T: Entity> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Uuid::deserialize(deserializer)?;
        Ok(Self::from_uuid(value))
    }
}

impl<T: Entity> PgHasArrayType for Id<T> {
    fn array_type_info() -> PgTypeInfo {
        <Uuid as PgHasArrayType>::array_type_info()
    }
}

#[derive(Debug, Error)]
pub enum IdError<T: Entity> {
    #[error("Invalid id for '{entity}': '{value}'")]
    Invalid {
        entity: &'static str,
        value: String,
        _marker: PhantomData<T>,
    },
}

impl<T: Entity> Type<Postgres> for Id<T> {
    fn type_info() -> PgTypeInfo {
        <Uuid as Type<Postgres>>::type_info()
    }
}

impl<'q, T: Entity> Encode<'q, Postgres> for Id<T> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        <Uuid as Encode<Postgres>>::encode_by_ref(&self.value, buf)
    }
}

impl<'r, T: Entity> Decode<'r, Postgres> for Id<T> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(Self {
            value: <Uuid as Decode<Postgres>>::decode(value)?,
            _marker: PhantomData,
        })
    }
}

impl<T: Entity> Deref for Id<T> {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEntity;

    impl Entity for TestEntity {
        fn key_name() -> &'static str {
            "test"
        }
    }

    #[test]
    fn serializes_id_as_uuid_string() {
        let id = Id::<TestEntity>::from_uuid(Uuid::nil());
        let json = serde_json::to_string(&id).expect("id should serialize to JSON");

        assert_eq!(json, "\"00000000-0000-0000-0000-000000000000\"");
    }

    #[test]
    fn deserializes_id_from_uuid_string() {
        let json = "\"550e8400-e29b-41d4-a716-446655440000\"";
        let id: Id<TestEntity> =
            serde_json::from_str(json).expect("id should deserialize from JSON string");

        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn fails_deserializing_invalid_uuid() {
        let json = "\"not-a-uuid\"";
        let result = serde_json::from_str::<Id<TestEntity>>(json);

        assert!(result.is_err());
    }
}
