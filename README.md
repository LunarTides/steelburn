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

- [ ] begin work on proper client made in godot
- [x] add `test` deckcode (30 sheep)
- [x] add card logic using lua (see [oakdom](https://git.lunartides.dev/LunarTides/oakdom))
- [ ] add first-party pack support
- [ ] add server config
- [ ] packs should have client companions (`/client/`) that only install on the client
- [ ] pack negotiation screen in queue for when two peers with different packs installed. mark packs as must have, etc
