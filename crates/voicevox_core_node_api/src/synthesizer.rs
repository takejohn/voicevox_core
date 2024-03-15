use napi::{Error, Result};
use voicevox_core::AccelerationMode;

/// {@link blocking.Synthesizer}および{@link promises.Synthesizer}のオプション。
#[napi(object)]
#[derive(Default)]
pub struct InitializeOptions {
    /// ハードウェアアクセラレーションモード。
    #[napi(ts_type = "'AUTO' | 'CPU' | 'GPU'")]
    pub acceleration_mode: Option<String>,

    /// CPU利用数を指定。0を指定すると環境に合わせたCPUが利用される。
    pub cpu_num_threads: Option<u16>,
}

impl InitializeOptions {
    pub fn convert(self) -> Result<voicevox_core::InitializeOptions> {
        Ok(voicevox_core::InitializeOptions {
            acceleration_mode: match self.acceleration_mode {
                Some(mode_str) => match mode_str.as_str() {
                    "AUTO" => AccelerationMode::Auto,
                    "CPU" => AccelerationMode::Cpu,
                    "GPU" => AccelerationMode::Gpu,
                    unknown_mode => {
                        return Err(Error::from_reason(format!(
                            "不明なハードウェアアクセラレーションモードの設定値: '{}'",
                            unknown_mode
                        )));
                    }
                },
                None => AccelerationMode::default(),
            },
            cpu_num_threads: self.cpu_num_threads.unwrap_or(0),
        })
    }
}
