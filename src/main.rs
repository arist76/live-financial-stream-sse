use std::{convert::Infallible, net::SocketAddr, pin::Pin, time::Duration};

use futures_util::Stream;
use http_body_util::StreamBody;
use hyper::{
    Method, Request, Response,
    body::{Bytes, Frame},
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use serde::Serialize;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_stream::{StreamExt, wrappers::ReceiverStream};
use url::form_urlencoded;
use yahoo_finance_api as yahoo;

#[derive(Debug, Serialize, Clone)]
struct StockQuote {
    symbol: String,
    price: f64,
    timestamp: i64,
}

async fn fetch_stock_data(
    symbol: &str,
) -> Result<StockQuote, Box<dyn std::error::Error + Send + Sync>> {
    let provider = yahoo::YahooConnector::new()?;
    let response = provider.get_latest_quotes(symbol, "1d").await?;
    let quote = response.last_quote()?;
    Ok(StockQuote {
        symbol: symbol.to_string(),
        price: quote.close,
        timestamp: quote.timestamp,
    })
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
) -> Result<
    Response<StreamBody<Pin<Box<dyn Stream<Item = Result<Frame<Bytes>, Infallible>> + Send>>>>,
    Infallible,
> {
    let response_builder = Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type");

    if req.method() == Method::OPTIONS {
        let empty_stream: Pin<Box<dyn Stream<Item = Result<Frame<Bytes>, Infallible>> + Send>> =
            Box::pin(futures_util::stream::empty());
        let response = response_builder
            .body(StreamBody::new(empty_stream))
            .unwrap();
        return Ok(response);
    }

    let query = req.uri().query().unwrap_or("");
    let params: std::collections::HashMap<String, String> =
        form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    let ticker = params
        .get("ticker")
        .map_or("AAPL".to_string(), |s| s.to_string());

    let (tx, rx) = mpsc::channel::<StockQuote>(100);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            eprintln!("Fetching stock data for {}", ticker);
            interval.tick().await;
            match fetch_stock_data(&ticker).await {
                Ok(quote) => {
                    if tx.send(quote).await.is_err() {
                        break;
                    }
                }
                Err(e) => eprintln!("Error fetching stock data for {}: {}", ticker, e),
            }
        }
    });

    let stream = ReceiverStream::new(rx).map(|quote| {
        let data = serde_json::to_string(&quote).unwrap();
        let event = format!("data: {}\n\n", data);
        Ok(Frame::data(Bytes::from(event)))
    });

    let response = response_builder
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(StreamBody::new(Box::pin(stream)
            as Pin<
                Box<dyn Stream<Item = Result<Frame<Bytes>, Infallible>> + Send>,
            >))
        .unwrap();
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req| handle_request(req)))
                .await
            {
                if !err.to_string().contains("IncompleteMessage") {
                    eprintln!("Error serving connection: {:?}", err);
                }
            }
        });
    }
}

