card.name = "The Coin"
card.type = "spell"
card.text = "<b>Battlecry:</b> Gain 1 Mana Crystal this turn only."
card.cost = 0

function card.onplay()
    owner.mana = owner.mana + 1
end
