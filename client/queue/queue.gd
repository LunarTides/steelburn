extends Control


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	var enter_queue_packet = Packet.from({
		"type": "enter_queue",
		"data": {},
	})
	Socket.send_packet(enter_queue_packet)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func _on_exit_button_pressed() -> void:
	Socket.close()
	get_tree().change_scene_to_file("uid://r6pj7x28jibn")
