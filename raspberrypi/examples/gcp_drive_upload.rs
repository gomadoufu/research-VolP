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

    let metadata = json!(
        {
            "name": "sample.wav",
            "mimeType": "audio/wav",
            "parents": ["1mCkwwOKMaNWwbKEjXY7H8_Nlfsh-Eb3i"],
            "samplingRateHertz": 44100,
        }
    );

    let file_path = "./sound/sample.wav";

    let form = reqwest::multipart::Form::new()
        .part(
            "metadata",
            reqwest::multipart::Part::text(serde_json::to_string(&metadata)?)
                .mime_str("application/json;charset=UTF-8")
                .unwrap(),
        )
        .part(
            "file",
            reqwest::multipart::Part::bytes(std::fs::read(file_path)?)
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
