extends Control

const QUEUE = preload("uid://c7nuuhce2ximt")

@export var server_list: VBoxContainer
@export var localhost_server_button: Button


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	Socket.connected.connect(func(server: Server):
		get_tree().change_scene_to_packed(QUEUE)
	)
	
	if OS.has_feature("connect_localhost"):
		server_list.selected = localhost_server_button.server
		server_list._on_connect_button_pressed()


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
