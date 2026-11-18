extends Control

const ERROR = preload("uid://8d161fair6hn")

@export var opponent_name: Label
@export var player_name: Label
@export var opponent_mana: Label
@export var player_mana: Label
@export var errors_container: VBoxContainer


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	player_name.text = Game.player.username
	opponent_name.text = Game.opponent.username
	
	for child in errors_container.get_children():
		child.queue_free()
	
	_on_player_changed(Game.player)
	_on_player_changed(Game.opponent)
	Game.player.changed.connect(_on_player_changed.bind(Game.player))
	Game.opponent.changed.connect(_on_player_changed.bind(Game.opponent))
	
	Game.packet_handler.error_recieved.connect(_on_packet_error)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func _on_player_changed(player: Player):
	if player.is_opponent:
		opponent_mana.text = "%s/%s" % [player.mana, player.empty_mana]
	else:
		player_mana.text = "%s/%s" % [player.mana, player.empty_mana]


func _on_packet_error(error: String):
	# Create a new error node
	var error_node: Control = ERROR.instantiate()
	var error_label = error_node.get_node("Label")
	error_label.text = "ERROR: %s" % error
	error_label.modulate = Color.RED
	
	error_node.get_node("DismissButton").pressed.connect(func():
		error_node.queue_free()
	)
	
	errors_container.add_child(error_node)
