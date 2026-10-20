local card = {}
card.name = "Sheep"
card.cost = 1

function card.onplay()
    print("[Lua/Sheep/onplay] Play")
    print("[Lua/Sheep/onplay] Is owner starting player: " .. tostring(owner.is_starting_player))
    owner.username = owner.username .. " (Lua)"
    print("[Lua/Sheep/onplay] " .. tostring(owner.username))
end

return card
