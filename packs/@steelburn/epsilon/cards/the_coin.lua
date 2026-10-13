local card = {}
card.name = "The Coin"
card.cost = 0

function card.onplay()
    print("[Lua/The Coin/onplay] Play")
end

return card
