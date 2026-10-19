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
var hovering_over_board: bool = false
var dragging: bool = false


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	if not card:
		return
	
	update_model()
	card.changed.connect(update_model)
	
	update_ui()
	card.changed.connect(update_ui)
	
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
	attack_mesh.visible = true
	health_mesh.visible = true
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
		# TODO: Handling hovering over a card on the board.
		collision_shape.disabled = true


func update_ui():
	name_label.visible = render_mode == RenderMode.Default
	text_label.visible = render_mode == RenderMode.Default
	cost_label.visible = render_mode == RenderMode.Default
	attack_label.visible = true
	health_label.visible = true
	tribe_label.visible = render_mode == RenderMode.Default
	
	if not card:
		return
	
	name_label.text = card.name
	text_label.text = card.text
	cost_label.text = str(card.cost)
	#attack_label.text = card.attack
	#attack_label.text = card.health
	#attack_label.text = card.tribe


func request_play():
	var packet = Packet.from({
		"type": "play_card",
		"data": {
			"hash": card.hash,
		},
	})
	Socket.send_packet(packet)


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


func _on_packet_recieved(packet: Packet) -> void:
	if packet.type == "play_card":
		if packet.data.card.hash != card.hash:
			return
		
		queue_free()
