use yup_oauth2::ServiceAccountAuthenticator;

#[tokio::main]
async fn main() {
    let creds = yup_oauth2::read_service_account_key("./auth/service_account.json")
        .await
        .unwrap();
    let sa = ServiceAccountAuthenticator::builder(creds)
        .build()
        .await
        .unwrap();
    let scopes = &["https://www.googleapis.com/auth/pubsub"];

    let tok = sa.token(scopes).await.unwrap();
    println!("token is: {:?}", tok);
    let tok = sa.token(scopes).await.unwrap();
    println!("cached token is {:?} and should be identical", tok);
}
