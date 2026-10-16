extends Control

@export var opponent_name: Label
@export var player_name: Label


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	player_name.text = Game.player.username
	opponent_name.text = Game.opponent.username


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
