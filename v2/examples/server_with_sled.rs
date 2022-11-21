use anyhow::Result;
use async_prost::AsyncProstStream;
use futures::SinkExt;
use futures::StreamExt;
use tokio::net::TcpListener;
use tracing::info;
use v2::CommandRequest;
use v2::CommandResponse;
use v2::MemTable;
use v2::Service;
use v2::ServiceInner;
use v2::SledDb;
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "127.0.0.1:9527";
    let listener = TcpListener::bind(addr).await?;
    // let service: Service = ServiceInner::new(MemTable::new()).into();
    let service: Service<SledDb> = ServiceInner::new(SledDb::new("/tmp/kvserver"))
        .fn_before_send(|res| match res.message.as_ref() {
            "" => res.message = "altered. Original message is empty.".into(),
            s => res.message = format!("altered: {}", s),
        })
        .into();
    info! {"Start listening on {}", addr};
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Client {:?} connected", addr);
        let svc = service.clone();
        tokio::spawn(async move {
            let mut stream =
                AsyncProstStream::<_, CommandRequest, CommandResponse, _>::from(stream).for_async();
            while let Some(Ok(cmd)) = stream.next().await {
                info!("Got a new command: {:?}", cmd);
                let resp = svc.execute(cmd);
                stream.send(resp).await.unwrap();
            }
            info!("Client {:?} disconnected", addr);
        });
    }
}
// RUST_LOG=info cargo run --example server_with_sled --quiet
// RUST_LOG=info cargo run --example client --quiet
