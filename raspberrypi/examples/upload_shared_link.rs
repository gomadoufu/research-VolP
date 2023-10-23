use anyhow::{Ok, Result};
use reqwest::Response;
use serde_json::json;
use yup_oauth2::{read_service_account_key, AccessToken, ServiceAccountAuthenticator};

#[derive(Debug)]
struct RequiredFields {
    file_name: String,
    parent_id: String,
    mime_type: String,
    upload_url: String,
}

impl RequiredFields {
    fn new(file_name: String, parent_id: String, mime_type: String, upload_url: String) -> Self {
        Self {
            file_name,
            parent_id,
            mime_type,
            upload_url,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let token = auth().await?;
    let file_name = "sine.wav";
    let parent_id = "1mCkwwOKMaNWwbKEjXY7H8_Nlfsh-Eb3i";
    let mime_type = "audio/wav";
    let upload_url = "https://www.googleapis.com/upload/drive/v3/files";

    let required_fields = RequiredFields::new(
        file_name.to_string(),
        parent_id.to_string(),
        mime_type.to_string(),
        upload_url.to_string(),
    );

    let response = upload_file(required_fields, token).await?;

    println!("File uploaded successfully!");

    let shared_link = get_shared_link(response).await?;

    println!("Shared link: {}", shared_link);

    Ok(())
}

/// get auth
/// auth -> token
async fn auth() -> Result<AccessToken> {
    let creds = read_service_account_key("./auth/service_account.json")
        .await
        .unwrap();
    let auth = ServiceAccountAuthenticator::builder(creds)
        .build()
        .await
        .unwrap();

    let scopes = &["https://www.googleapis.com/auth/drive.file"];

    let token = auth.token(scopes).await.unwrap();
    Ok(token)
}

/// upload file
/// required_fields, token -> response
async fn upload_file(required_fields: RequiredFields, token: AccessToken) -> Result<Response> {
    let metadata = json!(
        {
            "name": required_fields.file_name,
            "mimeType": required_fields.mime_type,
            "parents": [required_fields.parent_id],
        }
    );

    let file_path = format!("./sound/{}", required_fields.file_name);

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
                .mime_str(required_fields.mime_type.as_str())
                .unwrap(),
        );

    let client = reqwest::Client::new();

    let response = client
        .post(required_fields.upload_url)
        .query(&[("uploadType", "multipart")])
        .bearer_auth(token.token().unwrap())
        .multipart(form)
        .send()
        .await?;

    Ok(response)
}

/// get shared link
/// response -> shared_link
async fn get_shared_link(response: Response) -> Result<String> {
    let response = response.text().await?;
    let response_value = serde_json::from_str::<serde_json::Value>(response.as_str()).unwrap();

    let file_id = response_value
        .get("id")
        .unwrap()
        .as_str()
        .unwrap()
        .trim_matches('"');

    let shared_link = format!("https://drive.google.com/uc?id={}", file_id);
    Ok(shared_link)
}
