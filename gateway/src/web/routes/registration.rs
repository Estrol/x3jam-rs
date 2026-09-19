pub async fn get() -> impl axum::response::IntoResponse {
    if let Ok(registration) = tokio::fs::read_to_string("./resources/web/registration.html").await {
        axum::response::Html(registration)
    } else {
        axum::response::Html("<h1>Registration page not found</h1>".to_string())
    }
}

macro_rules! make_response {
    ($status:expr, $body:expr) => {{
        let code = match $status {
            true => axum::http::StatusCode::OK,
            false => axum::http::StatusCode::BAD_REQUEST,
        };

        let result = RegistrationResponse {
            success: $status,
            message: $body.to_string(),
        };

        super::make_response_json!(code, result)
    }};
}

pub async fn post(
    req: axum::http::Request<axum::body::Body>,
) -> impl axum::response::IntoResponse {
    let res = super::parse_request!(req, RegistrationRequest);

    if res.username.is_empty() || res.password.is_empty() || res.email.is_empty() {
        return make_response!(false, "Username, password, and email cannot be empty.");
    }

    let gender = match res.gender.as_deref() {
        Some("male") => database::CharacterGender::Male,
        Some("female") => database::CharacterGender::Female,
        _ => database::CharacterGender::Male, // Default
    };

    let token = crate::config::get_str("API", "RegistrationToken", "");
    if token.len() > 0 {
        if res.token.is_none() || res.token.as_deref().unwrap() != token {
            return make_response!(false, "Invalid registration token.");
        }
    }

    let database = crate::database::get();
    
    if database.get_user_by_name(&res.username).await.is_some() {
        return make_response!(false, "Username already exists.");
    }

    let hash = bcrypt::hash(&res.password, bcrypt::DEFAULT_COST)
        .expect("Failed to hash password");

    match database.create_user(
        &res.username,
        &hash,
        &res.username,
        &res.email,
        gender,
    ).await {
        Some(_) => make_response!(true, "User registered successfully."),
        None => make_response!(false, "Failed to register user."),
    }
}

#[derive(serde::Deserialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    pub gender: Option<String>,
    pub token: Option<String>,
}

#[derive(serde::Serialize)]
pub struct RegistrationResponse {
    pub success: bool,
    pub message: String,
}