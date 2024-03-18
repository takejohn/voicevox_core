use napi::{Error, Result};
use voicevox_core::UserDictWordType;

use crate::convert_result;

/// ユーザー辞書の単語。
#[napi(object, js_name = "UserDictWord")]
pub struct UserDictWord {
    /// 単語の表記。
    pub surface: String,
    /// 単語の読み。
    pub pronunciation: String,
    /// アクセント型。
    pub accent_type: Option<i64>,
    /// 単語の種類。
    #[napi(
        ts_type = "'PROPER_NOUN' | 'COMMON_NOUN' | 'VERB' | 'ADJECTIVE' | 'SUFFIX' | undefined | null"
    )]
    pub word_type: Option<String>,
    /// 単語の優先度。
    pub priority: Option<u32>,
}

impl UserDictWord {
    pub fn convert(self) -> Result<voicevox_core::UserDictWord> {
        convert_result(voicevox_core::UserDictWord::new(
            &self.surface,
            self.pronunciation,
            self.accent_type.unwrap_or(0) as usize,
            match self.word_type {
                Some(value) => match value.as_str() {
                    "PROPER_NOUN" => UserDictWordType::ProperNoun,
                    "COMMON_NOUN" => UserDictWordType::CommonNoun,
                    "VERB" => UserDictWordType::Verb,
                    "ADJECTIVE" => UserDictWordType::Adjective,
                    "SUFFIX" => UserDictWordType::Suffix,
                    unknown_type => {
                        return Err(Error::from_reason(format!(
                            "不明な単語の種類: '{}'",
                            unknown_type
                        )));
                    }
                },
                None => UserDictWordType::CommonNoun,
            },
            self.priority.unwrap_or(0),
        ))
    }
}

impl From<voicevox_core::UserDictWord> for UserDictWord {
    fn from(value: voicevox_core::UserDictWord) -> Self {
        UserDictWord {
            surface: value.surface,
            pronunciation: value.pronunciation,
            accent_type: Some(value.accent_type as i64),
            word_type: Some(match value.word_type {
                UserDictWordType::ProperNoun => String::from("PROPER_NOUN"),
                UserDictWordType::CommonNoun => String::from("COMMON_NOUN"),
                UserDictWordType::Verb => String::from("VERB"),
                UserDictWordType::Adjective => String::from("ADJECTIVE"),
                UserDictWordType::Suffix => String::from("SUFFIX"),
            }),
            priority: Some(value.priority as u32),
        }
    }
}
