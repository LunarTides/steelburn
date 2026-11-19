card.name = "High Score"
card.type = "spell"
card.text = "Play a level of a 2d platformer game. If beaten, gain Mana Crystals equal to the lives remaining."
card.cost = 2

function card.onplay(self, owner)
    local data = {
        text = "example"
    }

    owner:send_packet("gamma_high_score_start", data)
    owner.opponent:send_packet("gamma_high_score_start", data)
end
