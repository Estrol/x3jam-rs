use axum::body::to_bytes;

macro_rules! make_response {
    ($code:expr, $body:expr) => {{
        use axum::response::{IntoResponse};

        (
            $code,
            $body,
        )
            .into_response()
    }};
}

pub async fn post(
    req: axum::http::Request<axum::body::Body>,
) -> impl axum::response::IntoResponse {
    let query_params = super::parse_query_params!(req);

    match query_params.get("gameid") {
        Some(gameid) => {
            let mut items = Vec::new();

            for i in 1..=50 {
                let tid_key = format!("tid{}", i);
                if let Some(tid) = query_params.get(&tid_key) {
                    let tid_value = tid.parse::<u32>().unwrap_or(0);
                    if tid_value != 0 {
                        items.push(tid_value);
                    }
                }
            }

            for i in 1..=50 {
                let aid_key = format!("aid{}", i);
                if let Some(tid) = query_params.get(&aid_key) {
                    let tid_value = tid.parse::<u32>().unwrap_or(0);
                    if tid_value != 0 {
                        items.push(tid_value);
                    }
                }
            }

            if items.is_empty() {
                return make_response!(
                    axum::http::StatusCode::BAD_REQUEST,
                    "No items provided for payment"
                );
            }

            let Some(user) = crate::database::get().get_user_by_name(gameid).await else {
                return make_response!(
                    axum::http::StatusCode::NOT_FOUND,
                    format!("User {} not found", gameid)
                );
            };

            let item_list = crate::itemlist::get();

            let my_id = user.nickname.clone();
            let my_data = items
                .iter()
                .map(|&tid| Data {
                    id: tid,
                    name: item_list
                        .get_item(tid)
                        .map_or("Unknown Item".to_string(), |item| unsafe {
                            String::from_utf8_unchecked(item.name.to_vec())
                        }),
                    amount: 1,
                    price: 100, // Placeholder price
                })
                .collect::<Vec<Data>>();

            let html = tokio::fs::read_to_string("./resources/payment.html")
                .await
                .unwrap_or_default();

            let html = html.replace("$MY_ID_REPLACED$", &my_id);
            let html = html.replace(
                "$MY_DATA_REPLACED$",
                &serde_json::to_string(&my_data).unwrap_or_default(),
            );

            make_response!(axum::http::StatusCode::OK, axum::response::Html(html))
        }
        None => make_response!(axum::http::StatusCode::BAD_REQUEST, "Missing gameid parameter"),
    }
}

pub async fn post2(
    req: axum::http::Request<axum::body::Body>,
) -> impl axum::response::IntoResponse {
    let (_parts, body) = req.into_parts();

    let body_str = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => "<Failed to read body>".to_string(),
    };

    // Parse body as JSON PaymentRequest
    let payment_request: Result<PaymentRequest, _> = serde_json::from_str(&body_str);

    match payment_request {
        Ok(request) => {
            let database = crate::database::get();

            let Some(user) = database.get_user_by_name(&request.gameid).await else {
                return make_response!(
                    axum::http::StatusCode::NOT_FOUND,
                    format!("User {} not found", request.gameid)
                );
            };

            let item_list = crate::itemlist::get();

            let mut inventory = database.get_inventory(user.id).await;

            let find_empty_slot = |inventory: &[database::Item]| -> Option<usize> {
                (0..30).find(|slot| !inventory.iter().any(|item| item.slot as usize == *slot))
            };

            for item in request.items {
                if let Some(_) = item_list.get_item(item.id) {
                    let slot = find_empty_slot(&inventory);

                    if let Some(slot) = slot {
                        let new_item = database::Item {
                            item_id: item.id,
                            slot: slot as u8,
                            quantity: item.amount,
                        };

                        inventory.push(new_item);
                    } else {
                        log::info!("No empty slot available for user {}", user.nickname);
                    }
                }
            }

            let _ = database.save_inventory(user.id, &inventory).await;

            return make_response!(
                axum::http::StatusCode::OK,
                "Payment processed successfully"
            );
        }
        Err(e) => {
            log::info!("Failed to parse payment request: {}", e);

            return make_response!(
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid payment request format"
            );
        }
    }
}

#[derive(serde::Serialize)]
pub struct Data {
    pub id: u32,
    pub name: String,
    pub amount: u32,
    pub price: u32,
}

#[derive(serde::Deserialize)]
pub struct PaymentRequest {
    pub gameid: String,
    pub items: Vec<Item>,
}

#[derive(serde::Deserialize)]
pub struct Item {
    pub id: u32,
    pub amount: u32,
}
