use anyhow::Result;
use async_prost::AsyncProstStream;
use futures::SinkExt;
use futures::StreamExt;
use tokio::net::TcpListener;
use tracing::info;
use v4::CommandRequest;
use v4::CommandResponse;
use v4::MemTable;
use v4::Service;
use v4::ServiceInner;
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "127.0.0.1:9527";
    let listener = TcpListener::bind(addr).await?;
    let service: Service = ServiceInner::new(MemTable::default()).into();
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
