use std::{thread, time::Duration};
use tokio::{runtime::Runtime, time::sleep};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    WebSocketStream, connect_async,
    tungstenite::{Message, Utf8Bytes},
};
use tracing::{info, warn, error};

#[cfg(target_os = "linux")]
use nix::unistd::Pid;
#[cfg(target_os = "linux")]
use nix::sched::{CpuSet, sched_setaffinity};


use crate::candle::{CandleStickBuilder, candle_stick};

#[cfg(target_os = "macos")]
fn set_affinity(pin_id: usize) {
    println!("CPU affinity is not supported on macOS.");
}

#[cfg(target_os = "macos")]
fn get_affinity() {
    println!("Get CPU affinity is not supported on macOS.");
}

#[cfg(target_os = "linux")]
fn set_affinity(pin_id: usize) {
    // let num_cores = sysconf(nix::unistd::SysconfVar::_NPROCESSORS_ONLN);
    // info!("Number of cores: {}", num_cores);

    let mut cpuset = CpuSet::new();
    cpuset.set(pin_id).unwrap(); // Pin to CPU 0

    let pid = nix::unistd::Pid::this();
    sched_setaffinity(pid, &cpuset).unwrap();
    info!("Affinity set on Linux on {:?}", pin_id);
}

#[cfg(target_os = "linux")]
fn get_affinity() {
    // let cpuset = sched_getaffinity(Pid::this()).unwrap();
    // info!("Thread is running on cores: {:?}", cpuset);
}

pub trait Exchange {
    fn new(config_path: &str) -> Self
    where
        Self: Sized;
}

pub fn create_exchange<T: Exchange>(config_path: &str) -> T {
    T::new(config_path)
}

#[async_trait]
pub trait ExchangeFeed: Service {
    
    fn create_single_thread_runtime() -> Result<Runtime, Box<dyn std::error::Error>> {
        // let rt = tokio::runtime::Builder::new_multi_thread()
        let rt = tokio::runtime::Builder::new_current_thread()
            // .worker_threads(1)
            .enable_all()
            .build()?;
        Ok(rt)
    }

    async fn connect(&self, on_success: impl FnOnce(String) + Send) -> Result<String, String> {
        let (symbol, channel) = Self::get_sub_channel(self.symbol());

        // let connection_str = format!("{}{}", self.base_url(), self.symbol());
        let connection_str = Self::get_connection_str(self.base(), symbol);
        println!("connecting... {:?}", connection_str);

        // Connect to WebSocket
        let mut ws_stream = None;
        match connect_async(connection_str).await {
            Ok((stream, _)) => {
                // Connection was successful
                ws_stream = Some(stream);
                println!("Connected to WebSocket server");
            }
            Err(e) => {
                // Handle the error if the connection fails
                return Err(format!("Error Connecting: {:?}", e));
            }
        }

        on_success(format!("{} connected.", self.name()));

        let mut ws_stream = ws_stream.unwrap();

        Self::send_subscribe(channel, symbol, &mut ws_stream).await;

        let mut candle_stick_data = CandleStickBuilder::new().build();
        candle_stick_data.source = self.name().to_string();

        while let Some(result) = ws_stream.next().await {
            match result {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        Self::parse_text(text, &mut candle_stick_data);
                    }
                    _ => (),
                },
                Err(e) => {
                    // Return error message in case of connection failure
                    return Err(format!("Error receiving message: {:?}", e));
                }
            }
        }

        Ok(format!("Finished. {}", self.name()))
    }

    fn connect_on_thread(feed: Arc<Mutex<Self>>, pin_id: usize)
    where
        Self: Send + Sync + 'static,
    {
        println!("connecting... on thread");
        let success_callback = |message: String| {
            println!("Success callback received message: {}", message);
        };

        let self_clone = Arc::clone(&feed);

        thread::spawn(move || {
            if pin_id > 0 {
                set_affinity(pin_id);
            }

            // get_affinity();

            let rt = match Self::create_single_thread_runtime() {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("Error building runtime: {}", e);
                    return;
                }
            };

            rt.block_on(async {
                println!(
                    "Started {} thread: {:?}",
                    self_clone.lock().unwrap().name(),
                    std::thread::current().id()
                );
                let max_retries = 5; // Maximum retry attempts
                let mut retries = 0;

                while retries < max_retries {
                    let callback = success_callback.clone();
                    match self_clone.lock().unwrap().connect(callback).await {
                        Ok(msg) => {
                            println!("{:?}", msg);
                            return; // If connection is successful, exit the loop
                        }
                        Err(err) => {
                            eprint!("Error: {:?}. Retrying...\n", err);
                            retries += 1;
                            if retries < max_retries {
                                // Introduce a delay before retrying
                                let delay = Duration::from_secs(30);
                                sleep(delay).await;
                            } else {
                                eprintln!("Max retries reached. Exiting...");
                                self_clone.lock().unwrap().stop();
                                return; // Exit after max retries
                            }
                        }
                    }
                }
            });
        });
    }

    fn parse_text(text: Utf8Bytes, candle_stick_data: &mut candle_stick);

    fn get_sub_channel(_: &str) -> (&str, &str) {
        ("", "")
    }

    fn get_connection_str<'a>(_: &'a str, _: &'a str) -> String {
        "".to_string()
    }

    async fn send_subscribe(
        _: &str,
        _: &str,
        _: &mut WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
    ) {
    }
}

pub trait Service {
    fn name(&self) -> &str;
    fn symbol(&self) -> &str;
    fn base(&self) -> &str;
    fn enable(&self) -> bool;

    // async fn connect(&self, on_success: impl FnOnce(String) + Send) -> Result<String, String>;

    fn start(&self) {
        println!("Starting Serivce: {:?}", self.name());
    }

    fn stop(&self) {
        println!("Stopping Serivce: {:?}", self.name());
    }
}

pub struct BasicService {
    base_url: String,
    symbol: String,
    enable: bool,
    name: String,
}

impl BasicService {
    pub fn new(base_url: String, symbol: String, enable: bool, name: String) -> Self {
        BasicService {
            base_url,
            symbol,
            enable,
            name,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn enable(&self) -> bool {
        self.enable
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}
