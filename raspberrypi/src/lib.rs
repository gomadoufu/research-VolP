use anyhow::{Ok, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, Stream};
use reqwest::Response;
use rumqttc::{self, Key, QoS, TlsConfiguration, Transport};
use rumqttc::{AsyncClient, MqttOptions};
use serde_json::json;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{task, time};
use yup_oauth2::{read_service_account_key, AccessToken, ServiceAccountAuthenticator};

pub fn record_and_create_file(
    file_name: &str,
    spec: hound::WavSpec,
    button: &InputPin,
) -> Result<()> {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow::anyhow!("Failed to get default input device"))?;

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");
    println!("Default input config: {:?}", config);

    let path = format!("./sound/{}", file_name);

    let writer = hound::WavWriter::create(path, spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

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

    // ここでボタンの状態を確認し続ける
    while button.is_high() {
        // ボタンが押されている間は何もしない
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    drop(stream);
    writer.lock().unwrap().take().unwrap().finalize()?;
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

#[derive(Debug)]
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

pub async fn gcp_auth() -> Result<AccessToken> {
    let creds = read_service_account_key("./auth/service_account.json").await?;
    let auth = ServiceAccountAuthenticator::builder(creds).build().await?;

    let scopes = &["https://www.googleapis.com/auth/drive.file"];

    let token = auth.token(scopes).await?;
    Ok(token)
}

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
                .mime_str("application/json;charset=UTF-8")?,
        )
        .part(
            "file",
            reqwest::multipart::Part::bytes(std::fs::read(file_path)?)
                .mime_str(required_fields.mime_type.as_str())?,
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

pub type SharedLink = String;

pub async fn share_file(response: Response) -> Result<SharedLink> {
    let response = response.text().await?;
    let response_value = serde_json::from_str::<serde_json::Value>(response.as_str()).unwrap();

    let file_id = response_value
        .get("id")
        .ok_or_else(|| anyhow::anyhow!("Failed to get file id"))?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert &Value(file id) into &str"))?
        .trim_matches('"');

    let shared_link = format!("https://drive.google.com/uc?id={}", file_id);
    Ok(shared_link)
}

pub async fn mqtt_pub(url: SharedLink) -> Result<()> {
    let mut mqtt_options = MqttOptions::new(
        include_str!("../secrets/thing_name").trim_end_matches('\n'),
        include_str!("../secrets/aws_endpoint").trim_end_matches('\n'),
        8883,
    );
    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let ca = include_bytes!("../certs/AmazonRootCA1.pem");
    let cert = include_bytes!("../certs/certificate.pem.crt");
    let key = include_bytes!("../certs/private.pem.key");

    let transport = Transport::Tls(TlsConfiguration::Simple {
        ca: ca.to_vec(),
        alpn: None,
        client_auth: Some((cert.to_vec(), Key::RSA(key.to_vec()))),
    });

    mqtt_options.set_transport(transport);

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 5);

    task::spawn(async move {
        client
            .publish(
                "volp/share/link",
                QoS::AtMostOnce,
                false,
                format!("{{ \"link\": \"{}\"}}", url),
            )
            .await
            .expect("Failed to publish message");
        time::sleep(Duration::from_secs(10)).await;
    });

    loop {
        let notification = eventloop.poll().await;
        if let core::result::Result::Ok(rumqttc::Event::Outgoing(rumqttc::Outgoing::Publish(_))) =
            notification
        {
            break;
        }
    }
    Ok(())
}
