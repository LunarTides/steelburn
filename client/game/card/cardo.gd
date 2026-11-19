class_name Cardo
extends Area3D

enum RenderMode {
	Default,
	Board,
}

signal hover_started
signal hover_ended
signal drag_started
signal drag_ended

@export var card: Card
@export var player: Player
@export var render_mode: RenderMode = RenderMode.Default:
	set(value):
		render_mode = value
		update_model()
		update_ui()

@export_category("Nodes")
@export var model: Node3D
@export var collision_shape: CollisionShape3D
@export var name_label: Label3D
@export var text_label: Label3D
@export var cost_label: Label3D
@export var attack_label: Label3D
@export var health_label: Label3D
@export var tribe_label: Label3D

var hovering: bool = false:
	set(value):
		if hovering == value:
			return
		
		hovering = value
		
		if render_mode == RenderMode.Default:
			var vector = Vector3(0, 1, -3)
			if hovering:
				position += vector
			else:
				position -= vector
		
		send_hover_intention()
var hovering_over_board: bool = false
var dragging: bool = false:
	set(value):
		if dragging == value:
			return
		
		dragging = value
		send_drag_intention()
var location: String


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	if not card:
		return
	
	location = card.location
	
	update_model()
	card.changed.connect(update_model)
	
	update_ui()
	card.changed.connect(update_ui)
	card.changed.connect(update_self)
	
	Socket.packed_recieved.connect(_on_packet_recieved)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func _unhandled_input(event: InputEvent) -> void:
	if not hovering or render_mode != RenderMode.Default:
		return
	
	if event is InputEventMouseMotion:
		if event.button_mask & MOUSE_BUTTON_LEFT == 1:
			# Move around.
			dragging = true
			drag_started.emit()
			global_position = Game.camera.project_position(event.global_position, 9.8)
	elif event is InputEventMouseButton:
		if event.is_released() and dragging:
			drag_ended.emit()
			dragging = false
			
			if event.button_index == MOUSE_BUTTON_LEFT:
				# Check if dragging over board.
				request_play()


func update_model():
	if not card:
		return
	
	var armor_mesh: MeshInstance3D = model.get_node("Armor")
	var attack_mesh: MeshInstance3D = model.get_node("Attack")
	var card_mesh: MeshInstance3D = model.get_node("Card")
	var crystal_mesh: MeshInstance3D = model.get_node("Crystal")
	var frame_mesh: MeshInstance3D = model.get_node("Frame")
	var health_mesh: MeshInstance3D = model.get_node("Health")
	var health_frame_mesh: MeshInstance3D = model.get_node("HealthFrame")
	var name_mesh: MeshInstance3D = model.get_node("Name")
	var rarity_mesh: MeshInstance3D = model.get_node("Rarity")
	var text_mesh: MeshInstance3D = model.get_node("Text")
	var tribe_mesh: MeshInstance3D = model.get_node("Tribe")
	
	name_mesh.visible = render_mode == RenderMode.Default
	text_mesh.visible = render_mode == RenderMode.Default
	# TODO: Rotate attack and health mesh depending on the side of the board.
	attack_mesh.visible = card.attack != 0
	health_mesh.visible = card.health != 0
	health_frame_mesh.visible = health_mesh.visible
	crystal_mesh.visible = render_mode == RenderMode.Default
	rarity_mesh.visible = render_mode == RenderMode.Default
	# TODO: Don't make any assumptions about card types. If the card has a tribe, show it.
	tribe_mesh.visible = render_mode == RenderMode.Default
	armor_mesh.visible = false
	
	card_mesh.visible = render_mode == RenderMode.Default
	frame_mesh.visible = true
	
	if render_mode == RenderMode.Board:
		#var attack_vector = Vector3(0.4, 0.05, -2.5)
		#var health_vector = Vector3(-0.5, -0.1, -2.5)
		var attack_vector = Vector3(0.3, 0.05, -1)
		var health_vector = Vector3(-0.4, 0, -1)
		attack_mesh.position += attack_vector
		attack_label.position += attack_vector
		health_mesh.position += health_vector
		health_frame_mesh.position += health_vector
		health_label.position += health_vector
		
		# TODO: Maybe don't do this?
		collision_shape.shape = collision_shape.shape.duplicate()
		collision_shape.shape.size.z = 2.881
		collision_shape.position.z = -1.06
	else:
		collision_shape.shape = collision_shape.shape.duplicate()
		collision_shape.shape.size.z = 6.685
		collision_shape.position.z = 0.84


func update_ui():
	if not card:
		return
	
	name_label.visible = render_mode == RenderMode.Default
	text_label.visible = render_mode == RenderMode.Default
	cost_label.visible = render_mode == RenderMode.Default
	attack_label.visible = card.attack != 0
	health_label.visible = card.health != 0
	tribe_label.visible = render_mode == RenderMode.Default
	
	name_label.text = card.name
	text_label.text = card.text
	cost_label.text = str(card.cost)
	attack_label.text = "%s" % card.attack
	health_label.text = "%s" % card.health
	#tribe_label.text = card.tribe


func update_self():
	if card.location != location:
		# The card's location has changed. A new Cardo has been made somewhere, so delete this one.
		queue_free()
		return
	
	location = card.location


func request_play():
	var packet = Packet.from({
		"type": "play_card",
		"data": {
			"hash": card.hash,
		},
	})
	packet.send()


func _on_mouse_entered() -> void:
	hovering = true
	hover_started.emit()


func _on_mouse_exited() -> void:
	hovering = false
	hover_ended.emit()


func _on_area_entered(area: Area3D) -> void:
	if area == Game.board:
		hovering_over_board = true


func _on_area_exited(area: Area3D) -> void:
	if area == Game.board:
		hovering_over_board = false


func _on_packet_recieved(packet: ResponsePacket) -> void:
	if packet.type == "set_intention":
		if packet.data.type != "hover_board":
			return
			
		var hash = packet.data.hash
		if hash != card.hash:
			return
		
		var material = StandardMaterial3D.new()
		material.albedo_color = Color.hex(0xffff00ff)
		model.get_node("Frame").set_surface_override_material(0, material)
	elif packet.type == "clear_intentions":
		model.get_node("Frame").set_surface_override_material(0, null)


func send_hover_intention():
	if not hovering:
		var packet := Packet.from({
			"type": "clear_intentions",
			"data": {}
		})
		packet.send()
		return
	
	if card.location == "Hand":
		var packet := Packet.from({
			"type": "set_intention",
			"data": {
				"type": "hover_hand",
				"hand_index": player.hand.find_custom(func(c: Card) -> bool: return c.hash == card.hash),
			},
		})
		packet.send()
	elif card.location == "Board":
		var packet := Packet.from({
			"type": "set_intention",
			"data": {
				"type": "hover_board",
				"hash": card.hash,
			},
		})
		packet.send()


func send_drag_intention():
	if not dragging:
		var packet := Packet.from({
			"type": "clear_intentions",
			"data": {},
		})
		packet.send()
		return
	
	var packet := Packet.from({
		"type": "set_intention",
		"data": {
			"type": "drag_hand",
			"hand_index": player.hand.find_custom(func(c: Card) -> bool: return c.hash == card.hash),
		},
	})
	packet.send()
