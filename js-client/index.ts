console.log("Connecting...")

const username = process.argv[2];

const port = 56040;
const socket = new WebSocket(`ws://localhost:${port}`);

socket.addEventListener("open", () => {
  console.log("Connected.");

  socket.send(JSON.stringify({
    type: "player_info",
    data: {
      username: username ?? Bun.randomUUIDv7(),
    },
  }));

  socket.send(JSON.stringify({
    type: "end_turn",
    data: {},
  }));
});
socket.addEventListener("close", (event) => {
  console.log(`Disconnected. Reason: ${event.reason} (${event.code})`);
  process.exit(0);
});
socket.addEventListener("message", (event) => {
  console.log(`New message: ${event.data}`);
});
