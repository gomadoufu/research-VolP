use anyhow::Result;
use reqwest::Client;
use yup_oauth2::ServiceAccountAuthenticator;

#[tokio::main]
async fn main() -> Result<()> {
    let creds = yup_oauth2::read_service_account_key("./auth/service_account.json")
        .await
        .unwrap();
    let sa = ServiceAccountAuthenticator::builder(creds)
        .build()
        .await
        .unwrap();
    let scopes = &["https://www.googleapis.com/auth/drive"];

    let access_token = sa.token(scopes).await.unwrap();

    let response = Client::new()
        .get("https://www.googleapis.com/drive/v3/files/1mCkwwOKMaNWwbKEjXY7H8_Nlfsh-Eb3i?fields=*&key=")
        .bearer_auth(access_token.token().unwrap())
        .send()
        .await?;

    let value = response.text().await?;
    println!("{:#?}", value);

    Ok(())
}
