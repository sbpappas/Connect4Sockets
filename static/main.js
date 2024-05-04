/*let log = console.log;

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
  log("Received: " + e.data);
  document.getElementById("log").textContent =
    document.getElementById("log").textContent + "\n" + e.data;
};

conn.onclose = function () {
  log("Disconnected.");
  conn = null;
};

function send() {
  conn.send(document.getElementById("input").value);
  //conn.send(document.getElementById("connectFourCanvas"))
}

document.getElementById("btn").addEventListener("click", send);
*/
// connectFour.js

const canvas = document.getElementById("connectFourCanvas");
const ctx = canvas.getContext("2d");

const ROWS = 6;
const COLS = 7;
const CELL_SIZE = 50;
const PLAYER_ONE_COLOR = "red";
const PLAYER_TWO_COLOR = "yellow";

let currentPlayer = 1;
let board = [];
for (let i = 0; i < ROWS; i++) {
    board.push(new Array(COLS).fill(0));
}

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

/*conn.onmessage = function (e) {
  log("Received: " + e.data);
  document.getElementById("log").textContent =
    document.getElementById("log").textContent + "\n" + e.data;
};*/
conn.onmessage = function (e) {
  const gameState = JSON.parse(e.data);
  board = gameState.board;
  currentPlayer = gameState.currentPlayer;
  drawBoard();
};


conn.onclose = function () {
  log("Disconnected.");
  conn = null;
};

function send() {
  
  const gameState = {
      board: board,
      currentPlayer: currentPlayer
  };
  
  conn.send(JSON.stringify(gameState));
  //console.log(gameState);
  //conn.send(document.getElementById("input").value);
  //conn.send(document.getElementById("connectFourCanvas"))
}

//document.getElementById("connectFourCanvas").addEventListener("click", send);

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

function dropPiece(col) {
    for (let row = ROWS - 1; row >= 0; row--) {
        if (board[row][col] === 0) {
            board[row][col] = currentPlayer;
            drawBoard();
            if (checkForWin(row, col)) {
                alert("Player " + currentPlayer + " wins!");
                resetGame();
            } else {
                currentPlayer = currentPlayer === 1 ? 2 : 1;
            }
            send();
            return;
        }
    }
}

function checkForWin(row, col) {
    return (
        checkDirection(row, col, 0, 1) || // Horizontal
        checkDirection(row, col, 1, 0) || // Vertical
        checkDirection(row, col, 1, 1) || // Diagonal \
        checkDirection(row, col, 1, -1)   // Diagonal /
    );
}

function checkDirection(row, col, dRow, dCol) {
    const player = board[row][col];
    let count = 1;
    count += countInDirection(row + dRow, col + dCol, player, dRow, dCol);
    count += countInDirection(row - dRow, col - dCol, player, -dRow, -dCol);
    return count >= 4;
}

function countInDirection(row, col, player, dRow, dCol) {
    if (row < 0 || row >= ROWS || col < 0 || col >= COLS || board[row][col] !== player) {
        return 0;
    }
    return 1 + countInDirection(row + dRow, col + dCol, player, dRow, dCol);
}


function resetGame() {
    board = [];
    for (let i = 0; i < ROWS; i++) {
        board.push(new Array(COLS).fill(0));
    }
    currentPlayer = 1;
    drawBoard();
}
