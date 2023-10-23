use anyhow::{Ok, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use reqwest::Response;
use serde_json::json;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use yup_oauth2::{read_service_account_key, AccessToken, ServiceAccountAuthenticator};

pub fn record_file(file_name: &str, spec: hound::WavSpec) -> Result<()> {
    let host = cpal::default_host();

    let device = host.default_input_device().unwrap();

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");
    println!("Default input config: {:?}", config);

    let path = format!("./sound/{}", file_name);

    let writer = hound::WavWriter::create(path, spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    println!("Begin recording...");

    let writer_2 = writer.clone();

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let stream = device.build_input_stream(
        &config.into(),
        move |data, _: &_| write_input_data::<f32, i16>(data, &writer_2),
        err_fn,
        None,
    )?;

    stream.play()?;

    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(stream);
    writer.lock().unwrap().take().unwrap().finalize()?;
    println!("Recording stopped.");
    Ok(())
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let core::result::Result::Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

pub struct RequiredFields {
    pub file_name: String,
    pub parent_id: String,
    pub mime_type: String,
    pub upload_url: String,
}

impl RequiredFields {
    pub fn new(
        file_name: String,
        parent_id: String,
        mime_type: String,
        upload_url: String,
    ) -> Self {
        Self {
            file_name,
            parent_id,
            mime_type,
            upload_url,
        }
    }
}

/// get auth
/// auth -> token
pub async fn gcp_auth() -> Result<AccessToken> {
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
pub async fn upload_file(required_fields: RequiredFields, token: AccessToken) -> Result<Response> {
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
pub async fn get_shared_link(response: Response) -> Result<String> {
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
