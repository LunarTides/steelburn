extends ScrollContainer

const PACK_CONTAINER = preload("uid://cxjrumqj0se6b")

@export var packs: Array[Pack]

@export var container: VBoxContainer


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	for child in container.get_children():
		child.queue_free()
	
	for pack in packs:
		var pack_ui = PACK_CONTAINER.instantiate()
		pack_ui.pack = pack
		container.add_child(pack_ui)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
