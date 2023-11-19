use anyhow::{Ok, Result};
use chrono::Local;
use reqwest::Response;
use rppal::gpio::{Gpio, InputPin};
use volp_raspberrypi::{
    gcp_auth, mqtt_pub, record_and_create_file, share_file, upload_file, RequiredFields, SharedLink,
};

const GPIO_BUTTON: u8 = 22;
const GPIO_LED: u8 = 24;

#[tokio::main]
async fn main() -> Result<()> {
    loop {
        let button = Gpio::new()?.get(GPIO_BUTTON)?.into_input_pulldown();
        let mut led = Gpio::new()?.get(GPIO_LED)?.into_output();

        led.set_low();

        if button.is_low() {
            continue;
        }

        led.set_high();

        // 今の時間を取得して、ファイル名にする
        let now = Local::now();
        let file_name: String = now.format("%Y-%m-%d-%H-%M-%S.wav").to_string();

        // 録音して、ファイルを作成する
        record(file_name.as_str(), &button)?;

        // ファイルをアップロードする
        let response: Response = upload(file_name.as_str()).await?;

        println!("{:#?}", response);

        // 共有リンクを取得する
        let shared_link: SharedLink = share_file(response).await?;

        println!("Shared link: {}", shared_link);

        // MQTTで共有リンクを送信する
        mqtt_pub(shared_link).await?;
    }
}

fn record(file_name: &str, button: &InputPin) -> Result<()> {
    // 出力するWAVファイルの設定
    // Int16じゃないと、Drive側で再生できない
    // 入力は自動で設定される
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    println!("Start recording");
    record_and_create_file(file_name, spec, &button)?;
    println!("Finish recording");
    Ok(())
}

async fn upload(file_name: &str) -> Result<Response> {
    let token = gcp_auth().await?;
    let parent_id: &str = include_str!("../secrets/parent_id").trim_end_matches('\n');
    const MIME_TYPE: &str = "audio/wav";
    const UPLOAD_URL: &str = "https://www.googleapis.com/upload/drive/v3/files";

    let required_fields = RequiredFields::new(
        file_name.to_string(),
        parent_id.to_string(),
        MIME_TYPE.to_string(),
        UPLOAD_URL.to_string(),
    );

    println!("{:#?}", required_fields);

    let response = upload_file(required_fields, token).await?;

    Ok(response)
}
