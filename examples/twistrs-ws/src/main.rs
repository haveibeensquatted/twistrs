// #![deny(warnings)]
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use warp::ws::{Message, WebSocket};
use warp::Filter;

use twistrs::enrich::DomainMetadata;
use twistrs::permutate::Domain;

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let users = Users::default();
    let users = warp::any().map(move || users.clone());

    let chat = warp::path("chat")
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| ws.on_upgrade(move |socket| user_connected(socket, users)));

    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));
    let routes = index.or(chat);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn user_connected(ws: WebSocket, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
    }));

    // Save the sender in our list of connected users.
    users.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Make an extra clone to give to our disconnection handler...
    let user_disconnection_handler = users.clone();

    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &user_disconnection_handler).await;
}

async fn user_message(my_id: usize, msg: Message, users: &Users) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if my_id == uid {
            eprintln!("initiating dns resolution checks for user: {}", my_id);

            let domain = Domain::new(&msg).unwrap();
            let mut domain_permutations = domain.all().collect::<HashSet<String>>();
            domain_permutations.insert(String::from(domain.fqdn.clone()));

            for v in domain_permutations.into_iter() {
                let domain_metadata = DomainMetadata::new(v.clone());
                let tx = tx.clone();

                tokio::spawn(async move {
                    match domain_metadata.dns_resolvable().await {
                        Ok(metadata) => match metadata.ips {
                            Some(ips) => {
                                if let Err(_) = tx.send(Ok(Message::text(format!("{:?}", ips)))) {
                                    println!("received dropped");
                                    return;
                                }

                                drop(tx);
                            }
                            None => return,
                        },
                        Err(_) => return,
                    }
                });
            }
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Twistrs WebSocket DNS Resolution Example</title>
    </head>
    <body>
        <h1>Twistrs WebSocket DNS Resolution Example</h1>

        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        
        <script type="text/javascript">
        
        const chat = document.getElementById('chat');
        const text = document.getElementById('text');
        const uri = 'ws://' + location.host + '/chat';
        const ws = new WebSocket(uri);
        function message(data) {
            const line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }
        ws.onopen = function() {
            chat.innerHTML = '<p><em>Connected!</em></p>';
        };
        ws.onmessage = function(msg) {
            message(msg.data);
        };
        ws.onclose = function() {
            chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
        };
        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            text.value = '';
            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;
