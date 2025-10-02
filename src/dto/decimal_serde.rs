use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let f = value.to_string().parse::<f64>().unwrap_or(0.0);
    serializer.serialize_f64(f)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let num = serde_json::Number::deserialize(deserializer)?;
    Decimal::from_str(&num.to_string()).map_err(serde::de::Error::custom)
}

pub mod option {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(value: &Option<Decimal>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(d) => {
                let f = d.to_string().parse::<f64>().unwrap_or(0.0);
                serializer.serialize_some(&f)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<serde_json::Number>::deserialize(deserializer)?;
        opt.map(|num| Decimal::from_str(&num.to_string()).map_err(serde::de::Error::custom))
            .transpose()
    }
}

pub mod vec_array3 {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(value: &[[Decimal; 3]], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let f64_vec: Vec<[f64; 3]> = value
            .iter()
            .map(|[a, b, c]| {
                [
                    a.to_string().parse::<f64>().unwrap_or(0.0),
                    b.to_string().parse::<f64>().unwrap_or(0.0),
                    c.to_string().parse::<f64>().unwrap_or(0.0),
                ]
            })
            .collect();
        serializer.collect_seq(f64_vec)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<[Decimal; 3]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<[serde_json::Number; 3]>::deserialize(deserializer)?;
        vec
            .into_iter()
            .map(|[a, b, c]| {
                Ok([
                    Decimal::from_str(&a.to_string()).map_err(serde::de::Error::custom)?,
                    Decimal::from_str(&b.to_string()).map_err(serde::de::Error::custom)?,
                    Decimal::from_str(&c.to_string()).map_err(serde::de::Error::custom)?,
                ])
            })
            .collect()
    }
}

pub mod option_vec_array3 {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(
        value: &Option<Vec<[Decimal; 3]>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => {
                let f64_vec: Vec<[f64; 3]> = v
                    .iter()
                    .map(|[a, b, c]| {
                        [
                            a.to_string().parse::<f64>().unwrap_or(0.0),
                            b.to_string().parse::<f64>().unwrap_or(0.0),
                            c.to_string().parse::<f64>().unwrap_or(0.0),
                        ]
                    })
                    .collect();
                serializer.serialize_some(&f64_vec)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<[Decimal; 3]>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<Vec<[serde_json::Number; 3]>>::deserialize(deserializer)?;
        opt.map(|vec| {
            vec
                .into_iter()
                .map(|[a, b, c]| {
                    Ok([
                        Decimal::from_str(&a.to_string()).map_err(serde::de::Error::custom)?,
                        Decimal::from_str(&b.to_string()).map_err(serde::de::Error::custom)?,
                        Decimal::from_str(&c.to_string()).map_err(serde::de::Error::custom)?,
                    ])
                })
                .collect()
        })
        .transpose()
    }
}

pub mod option_vec_vec_decimal {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(
        value: &Option<Vec<Vec<Decimal>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => {
                let f64_vec: Vec<Vec<f64>> = v
                    .iter()
                    .map(|inner| {
                        inner
                            .iter()
                            .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
                            .collect()
                    })
                    .collect();
                serializer.serialize_some(&f64_vec)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<Vec<Decimal>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<Vec<Vec<serde_json::Number>>>::deserialize(deserializer)?;
        opt.map(|vec| {
            vec
                .into_iter()
                .map(|inner| {
                    inner
                        .into_iter()
                        .map(|num| Decimal::from_str(&num.to_string()).map_err(serde::de::Error::custom))
                        .collect()
                })
                .collect::<Result<Vec<Vec<Decimal>>, _>>()
        })
        .transpose()
    }
}
