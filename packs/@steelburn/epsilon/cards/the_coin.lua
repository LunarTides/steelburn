local card = {}
card.name = "The Coin"
card.cost = 0

function card.onplay()
    owner.mana = owner.mana + 1
end

return card
