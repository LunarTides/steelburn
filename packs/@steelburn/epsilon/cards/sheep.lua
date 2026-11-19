card.name = "Sheep"
card.type = "minion"
card.cost = 1

local mana = 1

function card.onplay(self, owner)
    print("[Lua/Sheep/onplay] Play")
    print("[Lua/Sheep/onplay] Is owner starting player: " .. tostring(owner.is_starting_player))
    self.text = "i was played :O"
    print("[Lua/Sheep/onplay] " .. tostring(self.text))
    print("[Lua/Sheep/onplay] " .. tostring(mana))

    mana = 2
    print("[Lua/Sheep/onplay] " .. tostring(mana))
end
