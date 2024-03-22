/// Promise版API。
#[napi]
pub mod promises {
    use napi::bindgen_prelude::Buffer;
    use napi::{Env, Result};
    use voicevox_core::tokio::{OpenJtalk, Synthesizer, UserDict, VoiceModel};
    use voicevox_core::{StyleId, VoiceModelId};

    use crate::metas::JsSpeakerMeta;
    use crate::model::{AccentPhrase, AudioQuery};
    use crate::synthesizer::{InitializeOptions, TtsOptions};
    use crate::word::UserDictWord;
    use crate::{convert_result, parse_uuid};

    /// テキスト解析器としてのOpen JTalk。
    #[napi(js_name = "OpenJtalk")]
    pub struct JsOpenJtalk {
        handle: OpenJtalk,
    }

    /// ユーザー辞書を設定する。
    ///
    /// この関数を呼び出した後にユーザー辞書を変更した場合は、再度この関数を呼ぶ必要がある。
    #[napi]
    impl JsOpenJtalk {
        #[napi(factory)]
        pub async fn create(open_jtalk_dict_dir: String) -> Result<JsOpenJtalk> {
            Ok(JsOpenJtalk {
                handle: convert_result(OpenJtalk::new(open_jtalk_dict_dir).await)?,
            })
        }

        /// ユーザー辞書を設定する。
        ///
        /// この関数を呼び出した後にユーザー辞書を変更した場合は、再度この関数を呼ぶ必要がある。
        #[napi]
        pub async fn use_user_dict(&self, user_dict: &JsUserDict) -> Result<()> {
            convert_result(self.handle.use_user_dict(&user_dict.handle).await)
        }
    }

    /// ユーザー辞書。
    ///
    /// 単語はJSONとの相互変換のために挿入された順序を保つ。
    #[napi(js_name = "UserDict")]
    pub struct JsUserDict {
        handle: UserDict,
    }

    #[napi]
    impl JsUserDict {
        /// ユーザー辞書を作成する。
        #[napi(constructor)]
        pub fn new() -> Self {
            JsUserDict {
                handle: UserDict::new(),
            }
        }

        /// このオブジェクトの{@link Record}としての表現
        #[napi(getter, ts_return_type = "Record<string, UserDictWord>")]
        pub fn words(&self, env: Env) -> Result<napi::bindgen_prelude::Object> {
            self.handle.with_words(|map| {
                let mut obj = env.create_object()?;
                for (uuid, word) in map {
                    obj.set(uuid.to_string(), UserDictWord::from(word))?
                }
                Ok(obj)
            })
        }

        /// ユーザー辞書をファイルから読み込む。
        ///
        /// @throws ファイルが読めなかった、または内容が不正だった場合はエラーを返す。
        #[napi]
        pub async fn load(&self, store_path: String) -> Result<()> {
            convert_result(self.handle.load(&store_path).await)
        }

        /// ユーザー辞書に単語を追加する。
        #[napi]
        pub fn add_word(&self, word: UserDictWord) -> Result<String> {
            convert_result(self.handle.add_word(word.convert()?)).map(|uuid| uuid.to_string())
        }

        /// ユーザー辞書の単語を変更する。
        #[napi]
        pub fn update_word(&self, word_uuid: String, new_word: UserDictWord) -> Result<()> {
            convert_result(
                self.handle
                    .update_word(parse_uuid(word_uuid)?, new_word.convert()?),
            )
        }

        /// ユーザー辞書から単語を削除する。
        #[napi]
        pub fn remove_word(&self, word_uuid: String) -> Result<UserDictWord> {
            convert_result(self.handle.remove_word(parse_uuid(word_uuid)?))
                .map(|word| UserDictWord::from(&word))
        }

        /// 他のユーザー辞書をインポートする。
        #[napi]
        pub fn import_dict(&self, other: &JsUserDict) -> Result<()> {
            convert_result(self.handle.import(&other.handle))
        }

        /// ユーザー辞書を保存する。
        #[napi]
        pub async fn save(&self, store_path: String) -> Result<()> {
            convert_result(self.handle.save(&store_path).await)
        }
    }

    /// 音声シンセサイザ。
    #[napi(js_name = "Synthesizer")]
    pub struct JsSynthesizer {
        handle: Synthesizer<OpenJtalk>,
    }

    #[napi]
    impl JsSynthesizer {
        /// `Synthesizer`をコンストラクトする。
        #[napi(constructor)]
        pub fn new(open_jtalk: &JsOpenJtalk, options: Option<InitializeOptions>) -> Result<Self> {
            Ok(JsSynthesizer {
                handle: convert_result(Synthesizer::new(
                    open_jtalk.handle.clone(),
                    &(options.unwrap_or_default().convert()?),
                ))?,
            })
        }

        /// ハードウェアアクセラレーションがGPUモードかどうか。
        #[napi(getter)]
        pub fn is_gpu_mode(&self) -> bool {
            self.handle.is_gpu_mode()
        }

        /// 音声モデルを読み込む。
        #[napi]
        pub async fn load_voice_model(&self, model: &JsVoiceModel) -> Result<()> {
            convert_result(self.handle.load_voice_model(&model.handle).await)
        }

        /// 音声モデルの読み込みを解除する。
        #[napi]
        pub fn unload_voice_model(&self, voice_model_id: String) -> Result<()> {
            convert_result(
                self.handle
                    .unload_voice_model(&VoiceModelId::new(voice_model_id)),
            )
        }

        /// 指定したIDの音声モデルが読み込まれているか判定する。
        #[napi]
        pub fn is_loaded_voice_model(&self, voice_model_id: String) -> bool {
            self.handle
                .is_loaded_voice_model(&VoiceModelId::new(voice_model_id))
        }

        /// 今読み込んでいる音声モデルのメタ情報。
        #[napi(getter)]
        pub fn metas(&self) -> Vec<JsSpeakerMeta> {
            self.handle
                .metas()
                .into_iter()
                .map(|meta| JsSpeakerMeta::from(meta))
                .collect()
        }

        /// AudioQueryから音声合成を行う。
        #[napi]
        pub async fn synthesis(
            &self,
            audio_query: AudioQuery,
            style_id: u32,
            options: Option<crate::synthesizer::SynthesisOptions>,
        ) -> Result<Buffer> {
            convert_result(
                self.handle
                    .synthesis(
                        &(audio_query.convert()?),
                        StyleId::new(style_id),
                        &(options.unwrap_or_default().into()),
                    )
                    .await,
            )
            .map(|vec| vec.into())
        }

        /// AquesTalk風記法からAccentPhrase (アクセント句)の配列を生成する。
        #[napi]
        pub async fn create_accent_phrases_from_kana(
            &self,
            kana: String,
            style_id: u32,
        ) -> Result<Vec<AccentPhrase>> {
            let models = convert_result(
                self.handle
                    .create_accent_phrases_from_kana(kana.as_str(), StyleId::new(style_id))
                    .await,
            )?;
            AccentPhrase::convert_from_slice(&models).map_err(|err| err.into())
        }

        /// AccentPhraseの配列の音高・音素長を、特定の声で生成しなおす。
        #[napi]
        pub async fn replace_mora_data(
            &self,
            accent_phrases: Vec<AccentPhrase>,
            style_id: u32,
        ) -> Result<Vec<AccentPhrase>> {
            let models = AccentPhrase::convert_slice(&accent_phrases)?;
            let result = convert_result(
                self.handle
                    .replace_mora_data(&models, StyleId::new(style_id))
                    .await,
            )?;
            AccentPhrase::convert_from_slice(&result).map_err(|err| err.into())
        }

        /// AccentPhraseの配列の音素長を、特定の声で生成しなおす。
        #[napi]
        pub async fn replace_phoneme_length(
            &self,
            accent_phrases: Vec<AccentPhrase>,
            style_id: u32,
        ) -> Result<Vec<AccentPhrase>> {
            let models = AccentPhrase::convert_slice(&accent_phrases)?;
            let result = convert_result(
                self.handle
                    .replace_phoneme_length(&models, StyleId::new(style_id))
                    .await,
            )?;
            AccentPhrase::convert_from_slice(&result).map_err(|err| err.into())
        }

        /// AccentPhraseの配列の音高を、特定の声で生成しなおす。
        #[napi]
        pub async fn replace_mora_pitch(
            &self,
            accent_phrases: Vec<AccentPhrase>,
            style_id: u32,
        ) -> Result<Vec<AccentPhrase>> {
            let models = AccentPhrase::convert_slice(&accent_phrases)?;
            let result = convert_result(
                self.handle
                    .replace_mora_pitch(&models, StyleId::new(style_id))
                    .await,
            )?;
            AccentPhrase::convert_from_slice(&result).map_err(|err| err.into())
        }

        /// AquesTalk風記法から{@link AudioQuery}を生成する。
        #[napi]
        pub async fn audio_query_from_kana(
            &self,
            kana: String,
            style_id: u32,
        ) -> Result<AudioQuery> {
            let result = convert_result(
                self.handle
                    .audio_query_from_kana(kana.as_str(), StyleId::new(style_id))
                    .await,
            )?;
            AudioQuery::convert_from(&result).map_err(|err| err.into())
        }

        /// AquesTalk風記法から音声合成を行う。
        #[napi]
        pub async fn tts_from_kana(
            &self,
            kana: String,
            style_id: u32,
            options: Option<TtsOptions>,
        ) -> Result<Buffer> {
            convert_result(
                self.handle
                    .tts_from_kana(
                        kana.as_str(),
                        StyleId::new(style_id),
                        &options.unwrap_or_default().into(),
                    )
                    .await,
            )
            .map(|vec| vec.into())
        }

        /// 日本語のテキストからAccentPhrase (アクセント句)の配列を生成する。
        #[napi]
        pub async fn create_accent_phrases(
            &self,
            text: String,
            style_id: u32,
        ) -> Result<Vec<AccentPhrase>> {
            let models = convert_result(
                self.handle
                    .create_accent_phrases(text.as_str(), StyleId::new(style_id))
                    .await,
            )?;
            AccentPhrase::convert_from_slice(&models).map_err(|err| err.into())
        }

        /// 日本語のテキストから{@link AudioQuery}を生成する。
        #[napi]
        pub async fn audio_query(&self, text: String, style_id: u32) -> Result<AudioQuery> {
            let model = convert_result(
                self.handle
                    .audio_query(text.as_str(), StyleId::new(style_id))
                    .await,
            )?;
            AudioQuery::convert_from(&model).map_err(|err| err.into())
        }

        /// 日本語のテキストから音声合成を行う。
        #[napi]
        pub async fn tts(
            &self,
            text: String,
            style_id: u32,
            options: Option<TtsOptions>,
        ) -> Result<Buffer> {
            convert_result(
                self.handle
                    .tts(
                        text.as_str(),
                        StyleId::new(style_id),
                        &options.unwrap_or_default().into(),
                    )
                    .await,
            )
            .map(|vec| vec.into())
        }
    }

    /// 音声モデル。
    ///
    /// VVMファイルと対応する。
    #[napi(js_name = "VoiceModel")]
    pub struct JsVoiceModel {
        pub(super) handle: VoiceModel,
    }

    #[napi]
    impl JsVoiceModel {
        /// VVMファイルから`VoiceModel`をコンストラクトする。
        #[napi(factory)]
        pub async fn from_path(path: String) -> Result<JsVoiceModel> {
            convert_result(
                VoiceModel::from_path(&path)
                    .await
                    .map(|handle| JsVoiceModel { handle }),
            )
        }

        /// ID。
        #[napi(getter)]
        pub fn id(&self) -> String {
            self.handle.id().to_string()
        }

        /// メタ情報。
        #[napi(getter)]
        pub fn metas(&self) -> Vec<JsSpeakerMeta> {
            self.handle
                .metas()
                .into_iter()
                .map(|handle| JsSpeakerMeta::from(handle.to_owned()))
                .collect()
        }
    }
}