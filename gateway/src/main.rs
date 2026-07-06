pub mod channel;
pub mod commandline;
pub mod config;
pub mod database;
pub mod gateway;
pub mod itemlist;
pub mod room;
pub mod user;
pub mod util;
pub mod web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    print_logo();

    console_subscriber::init();
    commandline::init();
    config::init();

    crate::database::init().await;
    crate::itemlist::init().await;

    let main_token = tokio_util::sync::CancellationToken::new();

    let main_task = util::spawn_named("Main Task", {
        let main_token = main_token.clone();
        async move {
            let _ = tokio::join!(
                crate::web::run(main_token.clone()),
                crate::gateway::run(main_token.clone())
            );
        }
    });

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");

    main_token.cancel();
    main_task.await.expect("Failed to wait for main task");

    Ok(())
}

const LOGO_ZIP_BASE64: &str = concat!(
    "H4sIAAAAAAAEAJWSUQ6AIAxD/028g8f",
    "1ExKniYmX4yQGDFspQ6PZj1spj8IyT0v+0r7mkiPtsZT+rlyme",
    "S8Uuw6uoTUrmZxMI1uSK/clWP2Ae5QR3MhhzFoWambR4BSLztG",
    "MPoNsyVgPxGTukPGsboax6YiyVFZ+D6PMQGDXBWcGsvYG394Zx",
    "gmFkXs7DYjx8BcK4J3ZDKScTfCx+nRV+eHcudX+DWQRQ5+dAwA",
    "A"
);

pub fn print_logo() {
    use base64::Engine as _;
    use std::io::Read;

    let logo_bytes = base64::engine::general_purpose::STANDARD
        .decode(LOGO_ZIP_BASE64)
        .expect("Failed to decode base64 logo");

    let mut decoder = flate2::read::GzDecoder::new(&logo_bytes[..]);
    let mut logo_string = String::new();
    decoder
        .read_to_string(&mut logo_string)
        .expect("Failed to decompress logo");

    println!("{}", logo_string);
    println!();
}
