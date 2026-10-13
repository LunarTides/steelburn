console.log("Connecting...")

const username = process.argv[2];

const port = 56040;
const socket = new WebSocket(`ws://localhost:${port}`);

let isInGame = false;
let currentHand = [];
let opponentHandSize = 0;

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
  }
});

setInterval(() => {
  if (!isInGame) {
    return;
  }

  console.log("Hand:", currentHand, "/", opponentHandSize);
}, 1000);
