use futures::FutureExt;
use std::collections::HashMap;
use std::sync::{Arc, atomic::AtomicBool};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};

#[trait_variant::make(IClient: Send + Sync)]
pub trait ILocalClient {
    fn create_client(
        socket: TcpStream,
        run: Arc<AtomicBool>,
        id: u64,
        token: tokio_util::sync::CancellationToken,
    ) -> Arc<Mutex<Self>>;
    fn id(&self) -> u64;
    fn is_running(&self) -> bool;
    fn data(&self) -> &[u8];

    async fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn read(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}

#[macro_export]
macro_rules! pin {
    ($callback:ident) => {{
        |client| {
            Box::pin(async move {
                $callback(client).await;
            })
        }
    }};
}

#[macro_export]
macro_rules! client {
    ($client:ty, $new_callback:expr) => {
        impl tcpserver::IClient for $client {
            fn create_client(
                socket: tokio::net::TcpStream,
                run: std::sync::Arc<std::sync::atomic::AtomicBool>,
                id: u64,
                token: tokio_util::sync::CancellationToken,
            ) -> std::sync::Arc<tokio::sync::Mutex<Self>> {
                std::sync::Arc::new(tokio::sync::Mutex::new($new_callback(socket, run, id, token)))
            }

            fn id(&self) -> u64 {
                self.id
            }

            fn is_running(&self) -> bool {
                self.run.load(std::sync::atomic::Ordering::SeqCst)
            }

            fn data(&self) -> &[u8] {
                &self.data
            }

            async fn send(
                &mut self,
                data: &[u8],
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                use tokio::io::AsyncWriteExt;
                use tokio::time::{timeout, Duration};

                let dur = Duration::from_secs(5);

                match timeout(dur, self.socket.write_all(data)).await {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(e)) => Err(Box::new(e)),
                    Err(_) => Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "send timed out",
                    ))),
                }
            }

            async fn read(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
                self.data.clear();
                let timeout = tokio::time::Duration::from_secs(5);

                loop {
                    tokio::select! {
                        // Timeout used, since .readable() might hang forever
                        // if the client disconnects without closing the connection properly.
                        // like client crashes, or network issues, etc.
                        result = tokio::time::timeout(timeout, self.socket.readable()) => {
                            match result {
                                Ok(Ok(())) => {
                                    match self.socket.try_read(&mut self.buffer) {
                                        Ok(0) => {
                                            // Connection closed by client.
                                            return Ok(0);
                                        }
                                        Ok(n) => {
                                            self.data.extend_from_slice(&self.buffer[..n]);
                                        }
                                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                            // No more data to read.
                                            break;
                                        }
                                        Err(e) => {
                                            return Err(Box::new(e));
                                        }
                                    }
                                }
                                Ok(Err(e)) => {
                                    return Err(Box::new(e));
                                }
                                Err(_) => {
                                    continue // Timeout occurred, continue to the next iteration to check for cancellation
                                }
                            }
                        }

                        _ = self.token.cancelled() => {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Interrupted,
                                "Client read cancelled",
                            )));
                        }
                    }
                }

                Ok(self.data.len())
            }
        }
    };
}

pub enum AddressType<'a> {
    Any,
    Address(&'a str),
}

pub struct Server<C: IClient + Send + Sync + 'static> {
    pub run: Arc<AtomicBool>,
    pub listener: TcpListener,
    pub socket: HashMap<u64, Arc<Mutex<C>>>,
    pub mpsc: (UnboundedSender<u64>, UnboundedReceiver<u64>),
    pub counter: u64,
}

impl<C: IClient + Send + Sync> Server<C> {
    pub async fn new(
        addr: AddressType<'_>,
        port: u16,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let bind_addr = match addr {
            AddressType::Any => format!("0.0.0.0:{}", port),
            AddressType::Address(a) => format!("{}:{}", a, port),
        };

        let listener = TcpListener::bind(bind_addr).await?;
        let run = Arc::new(AtomicBool::new(true));
        let channel = unbounded_channel::<u64>();

        Ok(Self {
            run,
            listener,
            socket: HashMap::new(),
            mpsc: channel,
            counter: 0,
        })
    }

    pub(crate) fn gen_next_id(&mut self) -> u64 {
        let id = self.counter;
        self.counter += 1;
        id
    }

    pub async fn run<F>(
        mut self,
        token: tokio_util::sync::CancellationToken,
        callback: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: for<'a> Fn(
                &'a mut C,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>>
            + Send
            + Sync
            + 'static,
    {
        self.run.store(true, std::sync::atomic::Ordering::SeqCst);
        let cb = Arc::new(callback);

        let client_token = tokio_util::sync::CancellationToken::new();

        loop {
            tokio::select! {
                result = self.listener.accept() => {
                    let (socket, _) = result?;
                    let run = self.run.clone();
                    let id = self.gen_next_id();

                    let client = C::create_client(socket, run, id, client_token.child_token());
                    let sender = self.mpsc.0.clone();
                    let cb = cb.clone();

                    println!("New client connected: ID={:?}", id);

                    self.socket.insert(id, client.clone());

                    tokio::task::Builder::new()
                        .name(&format!("client-{}", id))
                        .spawn(async move {
                            let id = {
                                let mut client = client.lock().await;
                                client.id();

                                let result = std::panic::AssertUnwindSafe(cb(&mut client)).catch_unwind().await;
                                if let Err(e) = result {
                                    println!("Client {} panicked: {:?}", client.id(), e);
                                }

                                client.id()
                            };

                            let _ = sender.send(id);
                        })?;
                }

                result = self.mpsc.1.recv() => {
                    if let Some(id) = result {
                        self.socket.remove(&id);
                    }
                }

                _ = token.cancelled() => {
                    self.run.store(false, std::sync::atomic::Ordering::SeqCst);
                    self.socket.clear();

                    break;
                }
            }
        }

        Ok(())
    }
}
