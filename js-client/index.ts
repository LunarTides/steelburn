console.log("Connecting...")

const username = process.argv[2];

const port = 56040;
const socket = new WebSocket(`ws://localhost:${port}`);

let isInGame = false;
let isStartingPlayer = false;
let currentHand = [];
let opponentHandSize = 0;
let currentBoard = [];
let opponentBoard = [];

function enterQueue() {
  socket.send(JSON.stringify({
    type: "enter_queue",
    data: {},
  }));

  console.log("Entered queue.");
}

socket.addEventListener("open", () => {
  console.log("Connected.");

  socket.send(JSON.stringify({
    type: "player_info",
    data: {
      username: username ?? Bun.randomUUIDv7(),
      deckcode: "debug",
    },
  }));

  enterQueue();
});
socket.addEventListener("close", (event) => {
  console.log(`Disconnected. Reason: ${event.reason} (${event.code})`);
  process.exit(0);
});
socket.addEventListener("message", (event) => {
  const packet = JSON.parse(event.data);
  console.log("\n[Server/Packet]");
  console.log(`${JSON.stringify(packet, null, 2)}`);

  if (packet.type === "begin_game") {
    isInGame = true;
    isStartingPlayer = packet.data.is_starting_player;

    // Testing packet.
    socket.send(JSON.stringify({
      type: "end_turn",
      data: {},
    }));
  } else if (packet.type === "end_game") {
    isInGame = false;
    currentHand = [];
    opponentHandSize = 0;

    // socket.close();
    // Enter queue again.
    enterQueue();
  } else if (packet.type === "draw_cards") {
    const cards = packet.data.cards;
    currentHand.push(...cards);

    socket.send(JSON.stringify({
      type: "play_card",
      data: {
        hash: cards[0].hash,
      },
    }));
  } else if (packet.type === "opponent_draw_cards") {
    const amount = packet.data.amount;
    opponentHandSize += amount;
  } else if (packet.type === "play_card") {
    const isFriendly = packet.data.is_starting_player === isStartingPlayer;
    const card = packet.data.card;

    if (isFriendly) {
      currentBoard.push(card);
      currentHand.splice(currentHand.indexOf(card), 1);
    } else {
      opponentBoard.push(card);
      opponentHandSize -= 1;
    }
  }
});

setInterval(() => {
  if (!isInGame) {
    return;
  }

  console.log("Hand:", currentHand, "/", opponentHandSize);
  console.log("Board:", currentBoard, "/", opponentBoard);
}, 1000);
