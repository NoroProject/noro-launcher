//! Аргументы запуска из version.json вместе с их условиями.
//!
//! Аргумент бывает либо просто строкой, либо объектом с `rules` — и правила
//! доезжают до клиента невычисленными. Мастер раздаёт один манифест на все
//! платформы и не знает, куда он уедет: свернуть правила у себя значит
//! отфильтровать их под ОС сервера. Считает их `backend::game_runner::rules`.

use serde::{Deserialize, Serialize};

/// Условие по ОС из `rules`.
///
/// Поля описаны все, какие встречаются, даже если клиент смотрит не на каждое:
/// мастер перекладывает правила через эту структуру в БД, и незнакомое поле
/// потерялось бы там навсегда. `version` — regex на версию ОС (`^10\.`), живёт
/// в манифестах 1.16–1.19 и к 1.20 из них исчез.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestRuleOs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestRule {
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<ManifestRuleOs>,
    /// Условия вида `is_demo_user` — мы их не включаем, см. модуль правил.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub features: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ManifestArg {
    String(String),
    Conditional {
        rules: Vec<ManifestRule>,
        #[serde(deserialize_with = "deserialize_arg_value")]
        value: Vec<String>,
    },
}

impl ManifestArg {
    pub fn new_string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }
}

/// `value` у Mojang то строка, то массив строк — принимаем оба вида, наружу
/// отдаём всегда массив, чтобы вызывающему не приходилось различать.
fn deserialize_arg_value<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct ValueVisitor;
    impl<'de> serde::de::Visitor<'de> for ValueVisitor {
        type Value = Vec<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("строку или массив строк")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            Ok(vec![value.to_owned()])
        }

        fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
            Ok(vec![value])
        }

        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(item) = seq.next_element::<String>()? {
                out.push(item);
            }
            Ok(out)
        }
    }
    deserializer.deserialize_any(ValueVisitor)
}
