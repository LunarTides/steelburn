extends Area3D

signal clicked

@export var model: Node3D


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	Game.turn_ended.connect(flip)
	Socket.packed_recieved.connect(_on_packet_recieved)
	
	if not Game.our_turn:
		flip(true)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func click():
	if not Game.our_turn:
		return
	
	Game.end_turn()
	clicked.emit()


func flip(instant = false):
	if instant:
		rotation_degrees.x = -90 if Game.our_turn else 90
		return
	
	_on_mouse_exited()
	var tween: Tween = create_tween().set_ease(Tween.EASE_OUT).set_trans(Tween.TRANS_CUBIC)
	tween.tween_property(self, "rotation_degrees:x", -90 if Game.our_turn else 90, 1)


func _on_input_event(camera: Node, event: InputEvent, event_position: Vector3, normal: Vector3, shape_idx: int) -> void:
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT and event.is_released():
			click()


func _on_mouse_entered() -> void:
	if Game.our_turn:
		scale *= 1.1
	
	var packet := Packet.from({
		"type": "set_intention",
		"data": {
			"type": "hover_end_turn",
		},
	})
	packet.send()


func _on_mouse_exited() -> void:
	scale = Vector3.ONE
	
	var packet := Packet.from({
		"type": "clear_intentions",
		"data": {},
	})
	packet.send()


func _on_packet_recieved(packet: ResponsePacket):
	var border: MeshInstance3D = model.get_node("Border")
	
	if packet.type == "set_intention":
		if packet.data.type != "hover_end_turn":
			return
		
		var material = StandardMaterial3D.new()
		material.albedo_color = Color.hex(0xffff00ff)
		border.set_surface_override_material(0, material)
	elif packet.type == "clear_intentions":
		border.set_surface_override_material(0, null)
