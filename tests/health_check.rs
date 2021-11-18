mod helper;

#[tokio::test]
async fn health_check_works() {
    let app;
    let client;
    let response;

    app = helper::spawn_app().await;
    client = reqwest::Client::new();
    response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
