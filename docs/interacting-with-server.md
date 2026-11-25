# Interacting with the server

Connect a WebSocket client to the url hosting the server (`ws://ip_or_domainname:port`). The port is by default `56040`, but can be changed.

Once the request goes through and you get the `open` WebSocket event, you're successfully connected.

Now you need to send a _Packet_ containing some information about you:

```json
{
  "type": "player_info",
  "data": {
    "deckcode": "Your deckcode", // Required
    "username": "Your Username", // Can be omitted
    "public_key": "-----BEGIN PUBLIC KEY-----" // Can be omitted
  }
}
```

_Packets_ are JSON strings, and they include a `type` field, and a `data` field. The fields you should include inside the `data` field changes based on the `type` of the packet.

In the above case, you need to supply a deckcode. You can optionally include a username. If you don't, the server will give you one that looks something like this: `Anon 01M4XTMW30JS2569SYXCWSKKSK`

Once you have sent this packet, the server will remember these details until you disconnect. If you want to update your username or deckcode, you can simply send this packet again, with the updated info. If you only specify the deckcode, it will only update the deckcode, leaving the username be, and vice-versa.

If the server accepts the packet successfully, it will echo this packet back to you. If there was an error, the packet will include an `error` field with an error message.

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
    "username": "Opponent's Username",
    "public_key": "-----BEGIN PUBLIC KEY-----" | "" // This can be any string, so be careful.
  }
}
```

This gives you some information about your opponent. Your opponent also recieves one of these packets about you.

```json
{
  "type": "begin_game",
  "is_starting_player": true | false,
  "data": {}
}
```

This packet signals that the game has begun. It tells you whether you are the starting player, or the player with the coin. You should store this value, since future packets may use this field to indicate who the packet applies to. Think of this boolean as your ID.

From this point onwards, you might recieve packets, or you might send packets. It's important that the information you send is correct, and that you handle the packets you recieve so that you maintain a correct gamestate.

Here is some basic JavaScript client code:

```js
console.log("Connecting...");

const host = "steelburn.lunartides.dev"; // This URL is an example. It does not exist yet.
const port = 56040;
const socket = new WebSocket(`wss://${host}:${port}`);

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
      // public_key: "", // optional
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
  
  if (packet.error) {
    // There was an error :(
    console.error(packet.error);
    return;
  }

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
    isStartingPlayer = packet.is_starting_player;
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
    return;
  }

  if (packet.type === "draw_cards") {
    // ...
  }
});
```

## Response Packets
When the server repeats a packet back to you, it usually does so in the form of a "response packet". These packets look like this:
```json
{
  "type": "packet_type",
  "is_starting_player": true | false, // May be omitted.
  "id": "packet_id" | null,           // May be omitted.
  "data": {},
  "error": "Error message" | null,    // May be omitted.
  "changes": {},                      // May be omitted.
}
```

This is called a `response packet`. It has the same type as the original packet, but it has the `is_starting_player` field, indicating whose turn ended, and might include other data too. This is a confirmation that your packet was recieved, but this packet is also often sent to your opponent too.

Any packet sent to the server may include an "id" field. This field will be included in any response packet that comes as a result of the packet you sent. Response packets also include a `is_starting_player` field. This shows the most relevant player to the packet. Simply think of this as the player who triggered the packet.

The changes field is very important. It usually looks something like this:
```json
{
  "changes": {
    "Player 1": {
      "mana": 4,
      "empty_mana": 4,
    },
    "01M876VWR1CDDVTJZ5QXKMEG2Y": {
      "health": 1,
    }
  }
}
```
This means that Player 1's (the starting player) mana and empty mana has changed to 4, and that the card with the hash `01M876VWR1CDDVTJZ5QXKMEG2Y` has had its health changed to 1. This shows the changes that have happened since the last packet, and may be included with any response packet.

## Packets

Here is a list of packets and their explanations.

### Server Info

```json
{
  "type": "server_info",
  "data": {
    "cheats": true | false,
    "queue_size": 0,
  }
}
```

Gives information about the server. This is usually sent right after connecting to the server. It includes whether cheats are enabled (cheat packets), and how many players are waiting in the queue.

### Player Info

```json
{
  "type": "player_info",
  "data": {
    "deckcode": "Your deckcode", // Required
    "username": "Your Username", // Can be omitted
    "public_key": "-----BEGIN PUBLIC KEY-----" // Can be omitted
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

### Request Identity Verification

```json
{
  "type": "request_identity_verification",
  "data": {
    "encrypted_message": "A message that has been encrypted using the opponent's public key.",
  }
}
```

Ask the opponent to verify their ownership of the public key recieved from `opponent_player_info`. Include a randomly generated message encrypted using their public key. The opponent should send back a `verify_identity` packet with the decrypted message. Then compare the original unencrypted message with the decrypted message. If they match, the opponent has the corresponding private key.

### Verify Identity

```json
{
  "type": "verify_identity",
  "data": {
    "decrypted_message": "Decrypted message from `request_identity_verification`",
  }
}
```

Verify that you own the private key corresponding to the public key you sent in the `player_info` packet by decrypting a message.

### End Turn

```json
{
  "type": "end_turn",
  "data": {}
}
```

If sent to the server, it signals the game to end your turn. If received, you should end your turn on the client.

### Play Card

```json
{
  "type": "play_card",
  "data": {
    "hash": "01M4XVPMMJG44HZXAQ6DWB89DK"
  }
}
```

Indicates that you want to play a card. Cards (which the server will often send to you in full JSON objects) include hashes, which is a unique identifier for that card. Use this hash to refer to that corresponding card.

The response packet:

```json
{
  "type": "play_card",
  "is_starting_player": false | true,
  "data": {
    "card": {
      "name": "Sheep",
      "cost": 1,
      "hash": "01M4XVPMMJG44HZXAQ6DWB89DK",
      ...
    } | "01M4XVPMMJG44HZXAQ6DWB89DK"
  }
}
```

The `card` field may be a full JSON object, or just the hash of the card. If the server has ever sent the full JSON object of this card before, it will refer to it through its hash from now on, to avoid repetition. Any changes to the card caused by, for example, Battlecry, is contained in the `changes` field of the response packet.

The `is_starting_player` field of the packet is who played the card.

### Opponent Player Info

```json
{
  "type": "opponent_player_info",
  "data": {
    "username": "Opponent's Username"
    "public_key": "-----BEGIN PUBLIC KEY-----" | "" // This can be any string. Be careful.
  }
}
```

This has already been talked about further up.

### Begin Game

```json
{
  "type": "begin_game",
  "is_starting_player": false | true
  "data": {}
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

### New Cards

```json
{
  "type": "new_cards",
  "data": {
    "cards": [
      {
        "name": "Sheep",
        "cost": 1,
        "hash": "01M4XVPMMJG44HZXAQ6DWB89DK",
        "location": "Hand" | "Board" | "Deck" | "Oblivion"
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

You'll recieve this when new cards are added to the game state. The location tells you where to place the card. Oblivion means the card isn't stored anywhere, and should be removed from play.

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

You'll recieve this when you draw cards.

### Opponent Draw Cards

```json
{
  "type": "opponent_draw_cards",
  "data": {
    "amount": 2
  }
}
```

You'll recieve this when the opponent draws cards. You won't be able to see information about the cards yet, only how many were drawn.

### Attack

```json
{
  "type": "attack",
  "data": {
    "attacker": "Player 1 | Player 2 | Card hash"
    "target": "Player 1 | Player 2 | Card hash"
  }
}
```

Attack a target using the attacker.

### Set Intention

```json
{
  "type": "set_intention",
  "data": {
    "type": "drag_hand" | "hover_hand" | "hover_board" | "hover_end_turn"
    "hand_index": 0, // Used in: drag_hand | hover_hand
    "hash": "01M4XVPMMJG44HZXAQ6DWB89DK" // Used in: hover_board
  }
}
```

Shows that the opponent is thinking about doing something. This shows as a yellow outline around something.

If the type is `drag_hand`, it means the opponent is dragging a card out of their hand. This card may be played. The `hand_index` is the index of the card that is being dragged.

If the type is `hover_hand`, it means the opponent is hovering over a card in their hand. The `hand_index` is the index of the card that is being hovered over.

If the type is `hover_board`, it means the opponent is hovering over a card on the board. The `hash` is the hash of the card that is being hovered over.

If the type is `hover_end_turn`, it means the opponent is hovering over the end turn button.

If you are implmenting your own client, remember to send these intentions yourself when relevant. This makes the game feel a lot more alive.

### Clear Intentions

```json
{
  "type": "clear_intentions",
  "data": {}
}
```

Remove all currently active intentions.

### Chat Message

```json
{
  "type": "chat_message",
  "data": {
    "message": "Message"
  }
}
```

Send a chat message to the opponent.

## Cheat Packets
These packets only work when cheats are enabled on the server. The server will tell you if cheats are enabled using the `cheats` property of the `server_info` packet.

### Get Packs

```json
{
  "type": "cheat_get_packs",
  "data": {}
}
```

Returns a list of all packs:
```json
{
  "type": "cheat_get_packs",
  "data": {
    "packs": [
      {
        "name": "epsilon",
        "description": "First official pack.",
        "author": "steelburn",
        "version": "0.1.0",
        "api_version": "0.1.0",
        "license": "GPL-3.0",
        "cards": [
          {
            "name": "Sheep",
            "cost": 1,
            ...
          }
        ],
      }
    ]
  }
}
```

### New Card

```json
{
  "type": "cheat_new_card",
  "data": {
    "author": "steelburn", // The author of the pack.
    "pack": "epsilon", // The name of the pack.
    "id": "The Coin" // The id of the card.
  }
}
```

Adds a card to your hand. A card's id is almost always equal to its name.

### Set Mana

```json
{
  "type": "cheat_set_mana",
  "data": {
    "is_starting_player": true | false,
    "mana": 0,
  }
}
```

Set the player associated with `is_starting_player`'s mana to `mana`.

### End Turn

```json
{
  "type": "cheat_end_turn",
  "data": {}
}
```

This ends the current turn without regard for whose turn it is. There is no validation, you can end your opponent's turn.
