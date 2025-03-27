pub mod hex_list {
    use core::fmt;
    use std::marker::PhantomData;

    use blueprint_sdk::tangle::extract::List;
    use serde::Serialize;
    use serde::de::{SeqAccess, Visitor};

    pub fn serialize<S, T>(list: &List<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: serde::Serialize + Default + AsRef<[u8]>,
    {
        let mut out = Vec::with_capacity(list.0.len());
        for item in &list.0 {
            out.push(hex::encode(item));
        }
        List(out).serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<List<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: serde::Deserialize<'de> + Default + hex::FromHex,
        <T as hex::FromHex>::Error: fmt::Display,
    {
        struct HexStrVisitor<T>(PhantomData<T>);
        impl<'de, T> Visitor<'de> for HexStrVisitor<T>
        where
            T: serde::Deserialize<'de> + Default + hex::FromHex,
            <T as hex::FromHex>::Error: fmt::Display,
        {
            type Value = List<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a list of hex-encoded strings")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(item) = seq.next_element::<String>()? {
                    let bytes = hex::FromHex::from_hex(item).map_err(serde::de::Error::custom)?;
                    out.push(bytes);
                }
                Ok(List(out))
            }
        }

        deserializer.deserialize_seq(HexStrVisitor::<T>(PhantomData))
    }
}
