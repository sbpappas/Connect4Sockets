use futures::StreamExt;
use warp::filters::query;
use std::env;
use std::fs;
use std::hash::Hash;
use std::net::SocketAddr;
use std::os::macos::raw::stat;
use std::thread::current;
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
type Players = HashMap<usize, Player>;

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

    // for key in users.keys() {

    // }


    let files = warp::fs::dir("./static");

    let res_404 = warp::any().map(|| {
        warp::http::Response::builder()
            .status(warp::http::StatusCode::NOT_FOUND)
            .body(fs::read_to_string("./static/404.html").expect("404 404?"))
    });

    let routes = chat.or(hello).or(files).or(res_404);

    let server = warp::serve(routes).try_bind(socket_address);

    println!("Running server at {}", addr);

    server.await
}

async fn connect(ws: WebSocket, users: Users) {
    // Bookkeeping
    let my_id = NEXT_USERID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    println!("Welcome User {}", my_id);
    let mut players: Players = HashMap::new();
    if my_id % 2 == 0 {
    players.insert(my_id, Player::Yellow);
    } else if my_id % 2 == 1 {
        players.insert(my_id, Player::Red);
    }
    // let mut game: GameState = GameState {board: vec![vec![0;7];6], move_col: 9, won: false};
    println!("players: {:?} ", players.get(&my_id).unwrap());
    
    
    // Establishing a connection
    let (user_tx, mut user_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();

    let rx = UnboundedReceiverStream::new(rx);

    tokio::spawn(rx.forward(user_tx));
    users.write().await.insert(my_id, tx);


    // Reading and broadcasting messages
    while let Some(result) = user_rx.next().await {
        //println!("Received message in serv: {:?}", result);
        //add user id to msg 
        
        broadcast_msg(result.expect("Failed to fetch message"), &users,  players.get(&my_id).unwrap()).await;
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

// use serde_json::Value;
/* 
async fn broadcast_msg(msg: Message, users: &Users, my_id: usize) {
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
}*/
use serde_json::json;

async fn broadcast_msg(msg: Message, users: &Users, user_id: &Player) {
    println!("in broadcast");
    let mut iwon = 0;
    // let mut current_player: Player = Player::Red;

    let player: usize = match user_id {
        Player::Red => 1,
        Player::Yellow => 2,
    };

   


    if let Ok(json_str) = msg.to_str() {
        let mut state: GameState = serde_json::from_str(&json_str).unwrap();
        let mut new_board: Vec<Vec<usize>> =  Vec::new();
        println!("state: {:?}", state);
        
        // Add my_id to the JSON object
        let update = to_board(state.board);
        // println!("col: {}", state.move_col);
        // *players.get(&user_id).unwrap(),
        if player == state.current_player {
            new_board = play(*user_id, update, state.move_col);
            // current_player = match current_player {
            //     Player::Red => Player::Yellow,
            //     Player::Yellow => Player::Red,
            // };
        } else {
            new_board = update.display();
        }
        let win_board = to_board(new_board.clone());
        let winner = win_board.check_winner();
        match winner {
            Some(Player::Red) => { iwon = 1;
                                state.won = true;
                                },
            Some(Player::Yellow) => { iwon = 2;
                                state.won = true;
                                },
            None => if !win_board.is_full() {
                        iwon = 0;
                    } else {
                        state.won = true;
                        iwon = 3;
                    },
        }


        let msg = json!({
            "board": new_board,
            "won": state.won,
            "winner": iwon,
            "currentPlayer": player,
        });

        // println!("{:?}", serde_json::to_string(&new_board));
        
        // game.updateBoard()
        // Broadcast the updated JSON object to all users
        for (&_uid, tx) in users.read().await.iter() {

            
            tx.send(Ok(Message::text(serde_json::to_string(&msg)
                .expect("Failed to serialize GameState."))))
                .expect("Failed to send message");
        }
        return;
    
    }
    // If the message does not contain valid JSON data, simply broadcast it as is
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


//commandline Connect4 game parts
#[derive(Debug, Deserialize, Serialize)]
struct GameState {
    board: Vec<Vec<usize>>,  // 2D vector representing the game board
    move_col: usize,
    won: bool,
    current_player: usize,
}




#[derive(Clone, PartialEq, Copy, Debug)]
enum Player {
    Red,
    Yellow,
}


struct Move {
    player: Player,
    column: i32,
}

impl Move {
    // reads a move from a column string
    fn read_move(c: usize, player: &Player) -> Option<Move>{

        if c < 7{
            return Some(Move{
                player: player.clone(),
                column: c as i32,
            })
                
        } else {
            return None
        }

    
    }
}

struct Board {
    game_board: Vec<Vec<Option<Player>>>,
}

impl Board {
    fn display(&self) -> Vec<Vec<usize>> { //turns into json format
        let mut json_board: Vec<Vec<usize>> = vec![vec![0;7];6];
        for i in 0..6{ //iterate through rows
            // println!("rows: {:?}", self.game_board[i]);
            for j in 0..7{//iterate thru cols
                match self.game_board[i][j] {
                    Some(Player::Red) => json_board[i][j] = 1,
                    Some(Player::Yellow) => json_board[i][j] = 2,
                    None => json_board[i][j] = 0,
                }
            }
        }
        return json_board
    }

    fn update_board(&mut self, m: Move) {
        for i in (0..6).rev() {
            // use m.column-1 because user inputs a num from 1-7, we need 0-6
            let j: usize = m.column as usize; 
            if self.game_board[i][j] == None {
                self.game_board[i][j] = Some(m.player);
                break;
            }
        }
    }
    

    fn is_full(&self) -> bool {
        for j in 0..=6{//move horizontally
            if self.game_board[0][j] == None { //go through the top row and find if any spot is open
                return false
            }
        }
        true
    }
    
    
    fn check_winner(&self) -> Option<Player> {
        let mut winner: Option<Player> = None;
        //horizontal check
        
        for i in 0..=3{
            for j in 0..=5{
                if self.game_board[j][i]== self.game_board[j][i+1] && self.game_board[j][i]== self.game_board[j][i+2] && self.game_board[j][i]== self.game_board[j][i+3] {
                    if self.game_board[j][i] != None {
                        winner = self.game_board[j][i]
                    }
                }
            }
        }
    
        //vertical check

        for i in 0..=2{
            for j in 0..=6{
                if self.game_board[i][j]== self.game_board[i+1][j] && self.game_board[i][j]== self.game_board[i+2][j] && self.game_board[i][j]== self.game_board[i+3][j] {
                    if self.game_board[i][j] != None {
                        winner = self.game_board[i][j]
                    }
                }
            }
        }

        //ascending diagonal check

        for i in 3..=5{
            for j in 0..=3{
                if self.game_board[i][j]== self.game_board[i-1][j+1] && self.game_board[i][j]== self.game_board[i-2][j+2] && self.game_board[i][j]== self.game_board[i-3][j+3] {
                    if self.game_board[i][j] != None {
                        winner = self.game_board[i][j]
                    }
                }
            }
        }

        //descending diagonal check

        for i in 3..=5{
            for j in 3..=6{
                if self.game_board[i][j]== self.game_board[i-1][j-1] && self.game_board[i][j]== self.game_board[i-2][j-2] && self.game_board[i][j]== self.game_board[i-3][j-3] {
                    if self.game_board[i][j] != None {
                        winner = self.game_board[i][j]
                    }
                }
            }
        }
        
        winner
    }

}

fn to_board(json: Vec<Vec<usize>>) -> Board {
    let mut board: Vec<Vec<Option<Player>>> = vec![vec![None; 7]; 6];
    for i in 0..6{ //iterate through rows
        for j in 0..7{//iterate thru cols
            match json[i][j] {
                1 => board[i][j] = Some(Player::Red),
                2 => board[i][j] = Some(Player::Yellow),
                _ => board[i][j] = None,   
            }
        }
    }
    let from_json =Board { game_board: board};
    return from_json
}

fn play(current_player: Player, mut game: Board, col: usize) -> Vec<Vec<usize>> {
    let mut current_move: Option<Move> = None;
    current_move = Move::read_move(col, &current_player);
    let current_move = current_move;
    match current_move {
        Some(e) => { game.update_board(e);
                    return game.display();},
        None => return game.display(),
    }
}

async fn get_player(id: usize) -> usize {
    match id%2 {
        1 => return 1,
        0 => return 2,
        _ => panic!("does not divide")
    }
}