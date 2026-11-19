card.name = "The Coin"
card.type = "spell"
card.text = "Gain 1 Mana Crystal this turn only."
card.cost = 0

function card.onplay(self, owner)
    owner.mana = owner.mana + 1
end
