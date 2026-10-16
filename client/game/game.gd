extends Node

const GAME = preload("uid://buos5pg3t5ke2")
const MAIN_MENU = preload("uid://r6pj7x28jibn")

signal started
signal ended
signal turn_ending
signal turn_ended

@export var player: Player = Player.new()
@export var opponent: Player = Player.new()

var packet_handler: GamePacketHandler = GamePacketHandler.new()
var camera: Camera3D
var playing: bool = false
var our_turn: bool = false


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	packet_handler._ready()


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func start():
	playing = true
	our_turn = player.is_starting_player
	get_tree().change_scene_to_packed(GAME)
	started.emit()


func end(reason: String):
	clean_slate()
	get_tree().change_scene_to_packed(MAIN_MENU)
	ended.emit()


func end_turn():
	if not our_turn:
		push_error("It is not our turn to end.")
		return
	
	var packet = Packet.from({
		"type": "end_turn",
		"data": {},
	})
	Socket.send_packet(packet)
	turn_ending.emit()


func clean_slate():
	playing = false
	player = Player.new()
	opponent = Player.new()
	_ready()
