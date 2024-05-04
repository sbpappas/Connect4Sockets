use futures::StreamExt;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
use warp::reject::{self, Reject};
use serde::{Deserialize, Serialize};

//use warp::http::StatusCode; // Import StatusCode


static MAX_USERS: usize = 2;
static NEXT_USERID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;

#[tokio::main]
async fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8081".to_string());
    let socket_address: SocketAddr = addr.parse().expect("valid socket Address");

    let users = Users::default();
    let users = warp::any().map(move || users.clone());

    let opt = warp::path::param::<String>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<String>,), std::convert::Infallible>((None,)) });

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path("hello")
        .and(opt)
        .and(warp::path::end())
        .map(|name: Option<String>| {
            format!("Hello, {}!", name.unwrap_or_else(|| "world".to_string()))
        });

    // GET /ws
    /*let chat = warp::path("ws")
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| ws.on_upgrade(move |socket| connect(socket, users)));
*/
    let chat = warp::path("ws")
    .and(warp::ws())
    .and(users.clone())
    .and_then(|ws: warp::ws::Ws, users: Users| {
        async move {
            if users.read().await.len() < MAX_USERS {
                Ok(ws.on_upgrade(move |socket| connect(socket, users)))
            } else {
                println!("Third user attempted joining, was rejected.");
                Err(reject::custom(TooManyRequests))           }
        }
    });

    let files = warp::fs::dir("./static");

    let res_404 = warp::any().map(|| {
        warp::http::Response::builder()
            .status(warp::http::StatusCode::NOT_FOUND)
            .body(fs::read_to_string("./static/404.html").expect("404 404?"))
    });

    let routes = chat.or(hello).or(files).or(res_404);

    let server = warp::serve(routes).try_bind(socket_address);

    println!("Running server at {}!", addr);

    server.await
}

async fn connect(ws: WebSocket, users: Users) {
    // Bookkeeping
    let my_id = NEXT_USERID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    println!("Welcome User {}", my_id);

    // Establishing a connection
    let (user_tx, mut user_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();

    let rx = UnboundedReceiverStream::new(rx);

    tokio::spawn(rx.forward(user_tx));
    users.write().await.insert(my_id, tx);

    // Reading and broadcasting messages
    while let Some(result) = user_rx.next().await {
        //println!("Received message in serv: {:?}", result);

        broadcast_msg(result.expect("Failed to fetch message"), &users).await;
    }

    // Disconnect
    disconnect(my_id, &users).await;
}

/*async fn broadcast_msg(msg: Message, users: &Users) {
    println!("in broadcast");
    if let Ok(_) = msg.to_str() {
        for (&_uid, tx) in users.read().await.iter() {
            tx.send(Ok(msg.clone())).expect("Failed to send message");
        }
    }
}*/

use serde_json::Value;

async fn broadcast_msg(msg: Message, users: &Users) {
    if let Ok(json_str) = msg.to_str() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
            if let Some(board) = json.get("board") {
                //println!("in broadcast mess");
                for (&_uid, tx) in users.read().await.iter() {
                    tx.send(Ok(Message::text(json_str.to_owned()))).expect("Failed to send message");
                }
                return;
            }
        }
    }
    for (&_uid, tx) in users.read().await.iter() {
        tx.send(Ok(msg.clone())).expect("Failed to send message");
    }
}

async fn disconnect(my_id: usize, users: &Users) {
    println!("Good bye user {}", my_id);
    users.write().await.remove(&my_id);
}



// Custom rejection type for too many requests
#[derive(Debug)]
struct TooManyRequests;

impl Reject for TooManyRequests {}

