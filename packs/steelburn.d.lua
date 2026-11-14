---@meta

---@class Card
---@field name string
---@field cost number
---@field text string
---@field type string
Card = {}

---Called when the card is played from the player's hand.
function Card.onplay() end

card = Card
self = Card

---@class Player
---@field is_starting_player boolean
---@field username string
---@field mana number
Player = {}

owner = Player
