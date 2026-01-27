use bson::oid::ObjectId;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(oid: &Option<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match oid {
        Some(oid) => serializer.serialize_str(&oid.to_hex()),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ObjectId>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let opt: Option<bson::Bson> = Option::deserialize(deserializer)?;
    match opt {
        Some(bson::Bson::ObjectId(oid)) => Ok(Some(oid)),
        None => Ok(None),
        _ => Err(D::Error::custom("expected ObjectId")),
    }
}
