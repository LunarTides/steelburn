extends Area3D

signal clicked


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	Game.turn_ended.connect(flip)
	
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
		rotation_degrees.x += 180
		return
	
	_on_mouse_exited()
	var tween: Tween = create_tween().set_ease(Tween.EASE_OUT).set_trans(Tween.TRANS_CUBIC)
	tween.tween_property(self, "rotation_degrees:x", rotation_degrees.x + 180, 1)


func _on_input_event(camera: Node, event: InputEvent, event_position: Vector3, normal: Vector3, shape_idx: int) -> void:
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT and event.is_released():
			click()


func _on_mouse_entered() -> void:
	if Game.our_turn:
		scale *= 1.1


func _on_mouse_exited() -> void:
	scale = Vector3.ONE
