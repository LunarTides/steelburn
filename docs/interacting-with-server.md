# Interacting with the server
Connect a WebSocket client to the url hosting the server (`ws://ip_or_domainname:port`). The port is by default `56040`, but can be changed.

Once the request goes through and you get the `open` WebSocket event, you're successfully connected.

Now you need to send a _Packet_ containing some information about you:
```json
{
  "type": "player_info",
  "data": {
    "username": "Your Username", // Can be omitted
    "deckcode": "Your deckcode" // Required
  }
}
```

_Packets_ are JSON strings, and they include a `type` field, and a `data` field. The fields you should include inside the `data` field changes based on the `type` of the packet.

In the above case, you need to supply a deckcode. You can optionally include a username. If you don't, the server will give you one that looks something like this: `Anon 01M4XTMW30JS2569SYXCWSKKSK`

Once you have sent this packet, the server will remember these details until you disconnect. If you want to update your username or deckcode, you can simply send this packet again, with the updated info. If you only specify the deckcode, it will only update the deckcode, leaving the username be, and vice-versa.

If the server accepts the packet successfully, it will echo this packet back to you.

Once you have sent this packet, it's time for the next step. Once you are ready to play, simply send a `enter_queue` packet:
```json
{
  "type": "enter_queue",
  "data": {}
}
```

If the server accepts the packet successfully, it will echo this packet back to you.

Now you have entered the queue, and the server is ready to pair you up with another player when there are enough players in the queue.

Once you are paired with another player, the server will send you a few packets, in this order:
```json
{
  "type": "opponent_player_info",
  "data": {
    "username": "Opponent's Username"
  }
}
```
This gives you some information about your opponent. Your opponent also recieves one of these packets about you.

```json
{
  "type": "begin_game",
  "data": {
    "is_starting_player": false | true
  }
}
```
This packet signals that the game has begun. It tells you whether you are the starting player, or the player with the coin. You should store this value, since future packets may use this field to indicate who the packet applies to. Think of this boolean as your ID.

From this point onwards, you might recieve packets, or you might send packets. It's important that the information you send is correct, and that you handle the packets you recieve so that you maintain a correct gamestate.

Here is some basic JavaScript client code:
```js
console.log("Connecting...")

const host = "steelburn.lunartides.dev";
const port = 56040;
const socket = new WebSocket(`ws://${host}:${port}`);

let isInGame = false;
let isStartingPlayer = false;

function sendPacket(packet) {
  socket.send(JSON.stringify(packet));
}

socket.addEventListener("open", () => {
  console.log("Connected to steelburn server.");

  // Send player info.
  sendPacket({
    type: "player_info",
    data: {
      // username: "Your Username", // optional
      deckcode: "Your Deckcode",
    },
  });
});

socket.addEventListener("close", (event) => {
  console.log(`Disconnected. Reason: ${event.reason}`);
  process.exit(0);
});

socket.addEventListener("message", (event) => {
  const packet = JSON.parse(event.data);

  if (packet.type === "player_info") {
    // Your information has been accepted. Enter queue.
    sendPacket({
      type: "enter_queue",
      data: {},
    });
    console.log("Entered queue.");
    return;
  }

  if (packet.type === "begin_game") {
    isInGame = true;
    isStartingPlayer = packet.data.is_starting_player;
    return;
  }

  if (packet.type === "end_game") {
    isInGame = false;
    
    // Re-enter queue.
    sendPacket({
      type: "enter_queue",
      data: {},
    });
    console.log("Entered queue.");
    return
  }

  if (packet.type === "draw_cards") {
    // ...
  }
});
```

## Packets you might send

### Player Info
```json
{
  "type": "player_info",
  "data": {
    "username": "Your Username", // Can be omitted
    "deckcode": "Your deckcode" // Required
  }
}
```
Gives the server information about you, some of which is neccessary to enter the queue. Some of this information may be shared with your opponent through the `opponent_player_info` packet.

### Enter Queue
```json
{
  "type": "enter_queue",
  "data": {}
}
```
Signals the server to place you in the queue. This will allow the server to pair you up with other players in the queue.

### End Turn
```json
{
  "type": "end_turn",
  "data": {}
}
```
Signals the game to end your turn. The server will respond with a similar packet:
```json
{
  "type": "end_turn",
  "data": {
    "is_starting_player": false | true
  },
}
```
This is called a `response packet`. It has the same type as the original packet, but it has the `is_starting_player` field, indicating whose turn ended, and might include other data too. This is a confirmation that your packet was recieved, but this packet is also sent to your opponent too.

### Play Card
```json
{
  "type": "play_card",
  "data": {
    "hash": "01M4XVPMMJG44HZXAQ6DWB89DK",
  }
}
```
Indicates that you want to play a card. Cards (which the server will send to you in full JSON objects) include hashes, which is a unique identifier for that card. Use this hash to refer to that corresponding card.

The response packet:
```json
{
  "type": "play_card",
  "data": {
    "is_starting_player": false | true,
    "card": {
      "name": "Sheep",
      "cost": 1,
      "hash": "01M4XVPMMJG44HZXAQ6DWB89DK",
      ...
    }
  }
}
```
It includes the player who played the card, as in the `is_starting_player` state of the player. It also includes the entire card object, rather than just its hash. This is because this packet is also sent to your opponent, and this will tell them about the card that was played.

## Packets you might recieve
Other than response packets, which are listed in the "Packets you might send" section, there are some packets you'll recieve without an explicit incident packet.

### Opponent Player Info
```json
{
  "type": "opponent_player_info",
  "data": {
    "username": "Opponent's Username"
  }
}
```
This has already been talked about further up.

### Begin Game
```json
{
  "type": "begin_game",
  "data": {
    "is_starting_player": false | true
  }
}
```
This has already been talked about further up.

### End Game
```json
{
  "type": "end_game",
  "data": {
    "reason": "opponent_disconnected"
  }
}
```
You'll recieve this when the game ends. The `reason` field shows why the game ended.

### Draw Cards
```json
{
  "type": "draw_cards",
  "data": {
    "cards": [
      {
        "name": "Sheep",
        "cost": 1,
        "hash": "01M4XVPMMJG44HZXAQ6DWB89DK",
        ...
      },
      {
        "name": "Sheep",
        "cost": 1,
        "hash": "01M4XWFTN0CMW5Y5XHER9Y7H54",
        ...
      }
    ]
  }
}
```
You'll recieve this when you draw cards. Notice how there's no `is_starting_player` field. That is because when the opponent draws cards, there's a different packet sent.

### Opponent Draw Cards
```json
{
  "type": "opponent_draw_cards",
  "data": {
    "amount": 2,
  }
}
```
You'll recieve this when the opponent draws cards. You won't be able to see information about the cards yet, only how many were drawn.
