mod helper;

#[tokio::test]
async fn add_user_returns_200_for_valid_form() {
    let app;
    let client;
    let response;
    let body;
    let saved;

    app = helper::spawn_app().await;
    client = reqwest::Client::new();

    body = "username=hello&email=world%40gmail.com";
    response = client
        .post(&format!("{}/user", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    saved = sqlx::query!("select username, email from axumstarter.user",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch new user.");

    assert_eq!(saved.username, "hello");
    assert_eq!(saved.email, "world@gmail.com")
}

#[tokio::test]
async fn add_user_returns_400_for_valid_form() {
    let app;
    let client;
    let test_cases;

    app = helper::spawn_app().await;
    client = reqwest::Client::new();
    test_cases = vec![
        ("username=hello", "missing the email"),
        ("email=john%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, err_msg) in test_cases {
        let response;

        response = client
            .post(&format!("{}/user", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            err_msg
        )
    }
}
