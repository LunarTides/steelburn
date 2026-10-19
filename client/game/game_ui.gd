extends Control

@export var opponent_name: Label
@export var player_name: Label
@export var opponent_mana: Label
@export var player_mana: Label


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	player_name.text = Game.player.username
	opponent_name.text = Game.opponent.username
	
	_on_player_changed(Game.player)
	_on_player_changed(Game.opponent)
	Game.player.changed.connect(_on_player_changed.bind(Game.player))
	Game.opponent.changed.connect(_on_player_changed.bind(Game.opponent))


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func _on_player_changed(player: Player):
	if player.is_opponent:
		opponent_mana.text = "%s/%s" % [player.mana, player.empty_mana]
	else:
		player_mana.text = "%s/%s" % [player.mana, player.empty_mana]
