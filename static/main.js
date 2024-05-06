const canvas = document.getElementById("connectFourCanvas");
const ctx = canvas.getContext("2d");
const whichPlayer = document.getElementById("player");

const ROWS = 6;
const COLS = 7;
const CELL_SIZE = 50;
const PLAYER_ONE_COLOR = "red";
const PLAYER_TWO_COLOR = "yellow";
let move_col = 69;
let won = false;
let winner = '';
let currentPlayer = 1;

let board = [];
for (let i = 0; i < ROWS; i++) {
    board.push(new Array(COLS).fill(0));
}

canvas.addEventListener("click", function (event) {
  const col = Math.floor(event.offsetX / CELL_SIZE);
  move_col = col;
  dropPiece();
});

let log = console.log;

const wsUri =
  ((window.location.protocol == "https:" && "wss://") || "ws://") +
  window.location.host +
  "/ws";
conn = new WebSocket(wsUri);

log("Connecting...");

conn.onopen = function () {
  log("Connected.");
};

conn.onmessage = function (e) {
  const gameState = JSON.parse(e.data);
  board = gameState.board;
  won = gameState.won;
  winner = gameState.winner;
  currentPlayer = gameState.currentPlayer;
  console.log("onmessage cur: " + currentPlayer);

  drawBoard();
  
  checkForWin();
  
};


conn.onclose = function () {
  log("Disconnected.");
  conn = null;
};



function send() {
  const gameState = {
      board: board,
      move_col: move_col,
      won: won,
      current_player: currentPlayer
  };
  console.log(gameState);
  conn.send(JSON.stringify(gameState));
  //conn.send(document.getElementById("input").value);
  //conn.send(document.getElementById("connectFourCanvas"))
}

function drawBoard() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "blue";
    ctx.fillRect(0, 0, COLS * CELL_SIZE, ROWS * CELL_SIZE);
    for (let row = 0; row < ROWS; row++) {
        for (let col = 0; col < COLS; col++) {
            const cellValue = board[row][col];
            if (cellValue === 1) {
                ctx.fillStyle = PLAYER_ONE_COLOR;
            } else if (cellValue === 2) {
                ctx.fillStyle = PLAYER_TWO_COLOR;
            } else {
                ctx.fillStyle = "white";
            }
            ctx.beginPath();
            ctx.arc(col * CELL_SIZE + CELL_SIZE / 2, row * CELL_SIZE + CELL_SIZE / 2, CELL_SIZE / 2 - 5, 0, Math.PI * 2);
            ctx.fill();
        }
    }
}

function checkForWin() {
  if (won) {
    if(winner === 1) {
    alert("Red wins!");
    resetGame();
    } else if (winner === 2) {
      alert("Red wins!");
      resetGame();
    } else if (winner === 3) {
      alert("It's a tie!");
      resetGame();
    } else {
      alert("You broke the game!!!");
      resetGame();
    }
  } 
    return;  
}

function dropPiece() {
  currentPlayer = currentPlayer === 1 ? 2 : 1;
  send();
  return;
}

function resetGame() {
    board = [];
    for (let i = 0; i < ROWS; i++) {
        board.push(new Array(COLS).fill(0));
    }
    won = false;
    winner = '';
    currentPlayer = 1;
    drawBoard();
}
