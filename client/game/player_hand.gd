extends Node3D

const CARDO = preload("uid://cu4ssj05rminf")
const CARDO_PLACEHOLDER = preload("uid://hi33bri48kwu")

@export var is_opponent := false

var player: Player


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	if is_opponent:
		player = Game.opponent
	else:
		player = Game.player
	
	# Get rid of editor cards.
	for child in get_children():
		child.queue_free()
	
	Socket.packed_recieved.connect(_packet_recieved)
	
	player.changed.connect(update)
	player.hand_changed.connect(update)
	update()


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func update():
	var children = get_children().filter(func(child: Node3D): return not child.is_queued_for_deletion())
	
	if is_opponent:
		# TODO: Support revealed cards in the opponent's hand.
		var amount = player.hand_size - children.size()
		if amount < 0:
			# Get rid of excess cards.
			for i in range(-amount):
				remove_child(get_child(0))
			layout()
			return
		
		for i in range(amount):
			# Create CardoPlaceholder.
			var cardo_placeholder = CARDO_PLACEHOLDER.instantiate()
			add_child(cardo_placeholder)
		layout()
		return
	
	for card in player.hand:
		var exists = false
		for child in children:
			if child.card == card:
				exists = true
				break
		
		if exists:
			# Already exists, don't worry about it.
			continue
		
		# New card. Create new cardo.
		var cardo: Cardo = CARDO.instantiate()
		cardo.card = card
		cardo.player = player
		cardo.drag_ended.connect(func():
			cardo.hovering = false
			layout()
		)
		add_child(cardo)
		
	layout()


func layout():
	var children = get_children()
	var i = 0
	
	var spacing = 3.25
	
	for child: Node3D in children:
		child.position = Vector3((i - children.size() / 2) * spacing, 0, 0)
		child.rotation = Vector3.ZERO
		i += 1


func _packet_recieved(packet: ResponsePacket):
	if not is_opponent:
		return
	
	# Show card outline when the opponent hovers over it.
	if packet.type == "set_intention":
		var intention_type = packet.data.type
		if intention_type == "hover_hand":
			var index = packet.data.hand_index
			handle_hover_hand_intention(index)
		elif intention_type == "drag_hand":
			var index = packet.data.hand_index
			handle_drag_hand_intention(index)
	elif packet.type == "clear_intentions":
		for cardo_placeholder: Area3D in get_children():
			cardo_placeholder.get_node("Outline").hide()
		layout()


func handle_hover_hand_intention(index: int):
	var child_count := get_child_count()
	
	if child_count <= index:
		return
	
	# NOTE: The cards are in reversed order.
	var cardo_placeholder: Area3D = get_child(index)
	cardo_placeholder.get_node("Outline").show()


func handle_drag_hand_intention(index: int):
	var child_count := get_child_count()
	
	if child_count <= index:
		return
	
	# NOTE: The cards are in reversed order.
	var cardo_placeholder: Area3D = get_child(index)
	var window_size := get_window().size / 2
	print(window_size)
	print(cardo_placeholder.global_position)
	
	cardo_placeholder.global_position += Vector3(0, 0.25, 5.5)
	cardo_placeholder.global_position.x = 0
	cardo_placeholder.rotate_y(deg_to_rad(30))
