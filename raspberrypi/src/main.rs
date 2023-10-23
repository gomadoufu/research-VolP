use anyhow::{Ok, Result};
use chrono::Local;
use reqwest::Response;
use volp_raspberrypi::{
    gcp_auth, get_shared_link, mqtt_pub, record_file, upload_file, RequiredFields,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 現在時刻でファイル名を作成
    let now = Local::now();
    let file_name = now.format("%Y-%m-%d-%H-%M-%S.wav").to_string();
    record(file_name.as_str())?;

    let response = upload(file_name.as_str()).await?;
    let shared_link = get_shared_link(response).await?;

    println!("Shared link: {}", shared_link);

    mqtt_pub(shared_link.as_str()).await?;

    Ok(())
}

fn record(file_name: &str) -> Result<()> {
    // 出力するWAVファイルの設定
    // 入力は自動で設定される
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    record_file(file_name, spec)?;
    Ok(())
}

async fn upload(file_name: &str) -> Result<Response> {
    let token = gcp_auth().await?;
    const PARENT_ID: &str = "1mCkwwOKMaNWwbKEjXY7H8_Nlfsh-Eb3i";
    const MIME_TYPE: &str = "audio/wav";
    const UPLOAD_URL: &str = "https://www.googleapis.com/upload/drive/v3/files";

    let required_fields = RequiredFields::new(
        file_name.to_string(),
        PARENT_ID.to_string(),
        MIME_TYPE.to_string(),
        UPLOAD_URL.to_string(),
    );

    let response = upload_file(required_fields, token).await?;

    println!("File uploaded successfully!");

    Ok(response)
}
