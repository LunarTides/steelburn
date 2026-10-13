local card = {}
card.name = "Sheep"
card.cost = 1

function card.onplay()
    print("[Sheep/play] is player1 starting player: " .. player1.is_starting_player())
end

return card
