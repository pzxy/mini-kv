use anyhow::Result;
use futures::prelude::*;
use prost::Message;
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;
use v5::{CommandRequest, MemTable, Service, ServiceInner};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let service: Service = ServiceInner::new(MemTable::new()).into();
    let addr = "127.0.0.1:9527";
    let listender = TcpListener::bind(addr).await?;
    info!("Start listening on {}", addr);
    loop {
        let (stream, addr) = listender.accept().await?;
        info!("Clietn {:?} connected", addr);
        let svc = service.clone();
        tokio::spawn(async move {
            // let mut stream = AsyncProstStream::<_, CommandRequest, CommandResponse, _>::from(stream).for_async();
            let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
            while let Some(Ok(mut buf)) = stream.next().await {
              let cmd = CommandRequest::decode(&buf[..]).unwrap();
              info!("Got a new command: {:?}", cmd);
              let res = svc.execute(cmd);
              buf.clear();
              res.encode(&mut buf).unwrap();
              stream.send(buf.freeze()).await.unwrap();
            }
            info!("Client {:?} disconnected", addr);
        });
    }
}
