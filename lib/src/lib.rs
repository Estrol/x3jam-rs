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
    fn create_client(socket: TcpStream, run: Arc<AtomicBool>, id: u64) -> Arc<Mutex<Self>>;
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
            ) -> std::sync::Arc<tokio::sync::Mutex<Self>> {
                std::sync::Arc::new(tokio::sync::Mutex::new($new_callback(socket, run, id)))
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

                self.socket.write_all(data).await?;
                Ok(())
            }

            async fn read(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
                self.data.clear();

                loop {
                    self.socket.readable().await?;

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
    pub async fn new(addr: AddressType<'_>, port: u16) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
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

        loop {
            tokio::select! {
                result = self.listener.accept() => {
                    let (socket, _) = result?;
                    let run = self.run.clone();
                    let id = self.gen_next_id();

                    let client = C::create_client(socket, run, id);
                    let sender = self.mpsc.0.clone();
                    let cb = cb.clone();

                    self.socket.insert(id, client.clone());

                    tokio::spawn(async move {
                        let id = {
                            let mut client = client.lock().await;
                            client.id();

                            let result = std::panic::AssertUnwindSafe(cb(&mut client)).catch_unwind().await;
                            if let Err(e) = result {
                                eprintln!("Error in client {}: {:?}", id, e);
                            }

                            client.id()
                        };

                        let _ = sender.send(id);
                    });
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