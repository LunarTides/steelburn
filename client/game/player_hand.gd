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
	
	player.hand_changed.connect(update)
	update()


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func update():
	var children = get_children()
	
	for card in player.hand:
		var exists = false
		for child in children:
			if child == card:
				exists = true
				break
		
		if exists:
			# Already exists, don't worry about it.
			continue
		
		# New card. Create new cardo.
		if is_opponent:
			# Create CardoPlaceholder.
			var cardo_placeholder = CARDO_PLACEHOLDER.instantiate()
			cardo_placeholder.rotation_degrees.y = 180
			add_child(cardo_placeholder)
			continue
		
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
		i += 1
