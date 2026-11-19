---@meta

---@class Card
---@field name string
---@field cost number
---@field text string
---@field type string
Card = {}

---Called when the card is played from the player's hand.
---@param self Card
---@param owner Player
function Card.onplay(self, owner) end

card = Card

---@class Player
---@field is_starting_player boolean
---@field username string
---@field mana number
Player = {}
