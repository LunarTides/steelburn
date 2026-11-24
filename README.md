# steelburn

yet another hearthstone recreation. this time, the idea is a bit more refined. a combination of efforts and what i've learned from [hearthstone.js](https://git.lunartides.dev/LunarTides/Hearthstone.js), [hearthstone.gd](https://git.lunartides.dev/LunarTides/Hearthstone.gd), [hearthstone.py](https://git.lunartides.dev/LunarTides/Hearthstone.py), and [oakdom](https://git.lunartides.dev/LunarTides/oakdom)

## rough idea

all calculations handled in rust server. server sends json card objects to client. client is just ui interface / packet sender. client does no calculations.

server handles multiple games at the same time.

cards made in Lua. only handled server-side.

on client attack:
send packet to server -> server accepts -> client plays animation. -> cards updated (through ids)

### packs

first party support for packs

packs have a server and a client component (client companion located in `/client` of pack)

server component can send packets to activate its client companion. this only works if the pack is installed on both the server and client, since the client needs to have the client companion installed. should minimize security risks.

## rough todo list

- [x] begin work on proper client made in godot
- [x] add `test` deckcode (30 sheep)
- [x] add card logic using lua (see [oakdom](https://git.lunartides.dev/LunarTides/oakdom))
- [x] add first-party pack support
- [x] All packets should include a `changes` field with player / card changes.
	- [x] Player
	- [x] Card
- [x] Cards should have locations. (Enum)
    - [x] Oblivion location for gone cards. (0)
    - [x] Don’t assume location changes on client.
- [x] Keep track of if cards have been sent to a peer already. If it has, only include the hash. Add a packet method for this.
    - [x] Add `has_sent_to_player1` field to cards
- [x] Fix ending turn
- [x] Store oblivion cards until next packet
- [x] Add `hand_size` to players
- [x] Add `test` deckcode (30 Sheep)
- [x] Add intentions
- [x] Add cheats
	- [x] `--cheats` argument
	- [x] Add `server_info` packet containing `cheats: bool` and `queue_size: int`
	- [x] Add `cheat_give` packet that allows giving a card
	- [ ] Make the UI collapsible
- [x] Add ssh auth for authentication?
    - [x] Lock next to opponent username. Click to trust ssh key
    - [ ] Add warning when trusting a connection outside the official server
    - [ ] Show other unknown usernames matching public key when username is different
        - [ ] Add identity manager with different keypairs to prevent this
	- [x] Make sure it supports empty public keys. (Ghost players)
	- [x] YOU CAN COPY THE MESSAGE AND SIGNATURE. SIGN A MESSAGE SENT BY THE OPPONENT INSTEAD
- [x] Add first-party pack support
    - [ ] ~~Should be global and only checked when the server starts. `packs: Vec<Arc<Pack>>`~~
- [x] Packets should have a hash
    - [ ] Only listen to response packets with the same id as the request packet
- [x] Send response packets to clients
	- [x] Put responsible player in response packets
	- [x] Make `play_card` response packet include actual card
	- [ ] Include queue info in the response packet for `enter_queue`
- [x] Choose which server to connect to in client
    - [ ] Allow servers to advertise using API endpoint
        - [ ] Periodically check these servers and get info about them
- [x] Add chat
	- [ ] Allow hiding the chat for the game (dropdown)
	- [ ] Turn off chat in settings
	- [ ] Gray out the chat message until the server confirms it
- [ ] Add server config
    - [x] Add debug mode to servers to allow debug packets. (Reimport packs, etc.)
- [ ] Implement anticheat
- [ ] Add pack info to packets
- [ ] Confirm packets on server (make sure packets sent to server are recieved)
- [ ] Confirm packets on client (make sure packets sent to client are recieved)
- [ ] Add animation library that client companions can add to
	- [ ] List of animations that shows them (in their different stages)
- [ ] Add tool for managing packs (create/delete) as command in steelburn server
- [x] Add 2d plattformer example pack
    - [ ] `Play a 2d platformer game. If beaten, gain Mana equal to the lives remaining.`
    - [ ] Show on opponents screen too. (Packets)
- [x] Give each card / resource it’s own lua env
    - [x] Don’t define / return card in lua file
- [ ] Merge `opponent_draw_cards` with `draw_cards`
	- [ ] Add `censored` field to packets, if some of the response is censored for the peer.
- [ ] Replace `draw_cards` with `new_card` with reason `draw`
- [ ] Make `Player.info` not an `Arc<Mutex>`
- [ ] Add `leave_queue` packet
- [ ] Manually serialize response packets
- [ ] Add a way for packs to interact / interfere with each other. (For modding)
- [ ] Add docs to rust code
	- [x] Update the markdown file
	- [ ] Add proper maintainable list of packets 
- [ ] Add docs to godot code
- [ ] Add docs to lua code
- [ ] Add reporting
- [ ] Add placing minions on different places on the board
- [ ] Add moderation tools
	- [ ] Record packets sent from clients and server to a file.
	- [ ] Step through packets on server. Save screenshots at each step so we can step backwards.
	- [ ] Ban / kick users
	- [ ] Roles in config file
	- [ ] Permissions for roles
	- [ ] Add `request_permissions` packet using public key
- [ ] Add targeting
	- [ ] `Card.target_on_play: "none" | "all" | "players" | "minions" | "enemy_minions" | "friendly_minions"`
	- [ ] `Card.target: Card | Player`
	- [ ] `Card.is_card` `Card.is_player` `Player.is_card` `Player.is_player`
- [ ] Add enchantments
- [ ] Add events
	- [ ] From lua side: `function event.playcard(card) print(card) end`
- [ ] Add pack conflict warning when moving a third-party source above the official one.
- [ ] Packs should have client companions (`/client/`) that only install on the client
- [ ] Pack negotiation screen in queue for when two peers with different packs pair. Mark packs as must have, etc.
    - [ ] Or maybe just look at the deckcode? But then we need to handle pack side effects…
- [ ] ~~Add testing server for testing cards~~
    - [ ] ~~Force players to upload packs (configurable from server settings)~~
    - [ ] ~~Play against AI~~
        - [ ] ~~Maybe you can choose what the AI does?~~
        - [ ] ~~Maybe the client can switch sides?~~
