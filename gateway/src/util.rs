use futures::FutureExt as _;

#[inline]
pub fn spawn_named(
    name: impl Into<String>,
    future: impl std::future::Future<Output = ()> + Send + 'static,
) -> tokio::task::JoinHandle<()> {
    let name = name.into();
    let named = name.clone();

    tokio::task::Builder::new()
        .name(&name)
        .spawn(async move {
            let result = std::panic::AssertUnwindSafe(future).catch_unwind().await;
            if let Err(e) = result {
                log::info!("Task {} panicked: {:?}", named, e);
            }
        })
        .expect("Failed to spawn task")
}

#[inline]
pub fn spawn_named_blocking(
    name: impl Into<String>,
    future: impl FnOnce() + Send + 'static,
) -> tokio::task::JoinHandle<()> {
    let name = name.into();
    let named = name.clone();

    tokio::task::Builder::new()
        .name(&name)
        .spawn_blocking(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(future));
            if let Err(e) = result {
                log::info!("Blocking task {} panicked: {:?}", named, e);
            }
        })
        .expect("Failed to spawn blocking task")
}
