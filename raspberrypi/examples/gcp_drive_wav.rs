use anyhow::Result;
use serde_json::json;
use yup_oauth2::{read_service_account_key, ServiceAccountAuthenticator};

#[tokio::main]
async fn main() -> Result<()> {
    let creds = read_service_account_key("./auth/service_account.json")
        .await
        .unwrap();
    let auth = ServiceAccountAuthenticator::builder(creds)
        .build()
        .await
        .unwrap();
    let scopes = &["https://www.googleapis.com/auth/drive.file"];

    let token = auth.token(scopes).await.unwrap();

    let file_path = "./sound/sample.wav";

    let reader = hound::WavReader::open(file_path).expect("Failed to open WAV file");

    println!("WAV file information:");
    println!("  Channels: {}", reader.spec().channels);
    println!("  Sample rate (Hz): {}", reader.spec().sample_rate);
    println!("  Bits per sample: {}", reader.spec().bits_per_sample);
    println!("  Sample format: {:?}", reader.spec().sample_format);

    let mut samples_u8 = Vec::new();

    // WAV ファイルのサンプル形式によって処理を分岐
    match reader.spec().sample_format {
        hound::SampleFormat::Int => {
            // サンプルが整数の場合
            let samples: Vec<i32> = reader
                .into_samples::<i32>()
                .map(|s| s.expect("Failed to read sample"))
                .collect();
            // i32 のサンプルを u8 に変換
            samples_u8 = samples.iter().map(|&s| s as u8).collect();
        }
        hound::SampleFormat::Float => {
            // サンプルが浮動小数点数の場合
            let samples: Vec<f32> = reader
                .into_samples::<f32>()
                .map(|s| s.expect("Failed to read sample"))
                .collect();
            // f32 のサンプルを u8 に変換
            samples_u8 = samples.iter().map(|&s| s as u8).collect();
        }
    }

    let metadata = json!(
        {
            "name": "sample.wav",
            "mimeType": "audio/wav",
            "parents": ["1mCkwwOKMaNWwbKEjXY7H8_Nlfsh-Eb3i"],
            "samplingRateHertz": 44100,
        }
    );

    let form = reqwest::multipart::Form::new()
        .part(
            "metadata",
            reqwest::multipart::Part::text(serde_json::to_string(&metadata)?)
                .mime_str("application/json;charset=UTF-8")
                .unwrap(),
        )
        .part(
            "file",
            reqwest::multipart::Part::bytes(samples_u8)
                .mime_str("audio/wav")
                .unwrap(),
        );

    let client = reqwest::Client::new();

    let upload_url = "https://www.googleapis.com/upload/drive/v3/files";
    // let folder_id = "1mCkwwOKMaNWwbKEjXY7H8_Nlfsh-Eb3i";

    let response = client
        .post(upload_url)
        .query(&[("uploadType", "multipart")])
        .bearer_auth(token.token().unwrap())
        .multipart(form)
        .send()
        .await?;

    let value = response.text().await?;
    println!("{:#?}", value);

    println!("File uploaded successfully!");

    Ok(())
}
