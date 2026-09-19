use axum::extract::Query;

pub async fn get(Query(params): Query<Params>) -> impl axum::response::IntoResponse {
    let userlist = tokio::fs::read_to_string("./resources/web/userlist.html")
        .await
        .unwrap_or_default();

    let userlist = userlist.replace("$MY_ID_REPLACED$", &params.myid);

    axum::response::Html(userlist)
}

#[derive(serde::Deserialize)]
pub struct Params {
    myid: String,
}
