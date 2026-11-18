extends PanelContainer

const PACK_CARD = preload("uid://dj34m4pq3cgi1")

@export var pack: Pack

@export var pack_label: Label
@export var card_container: VBoxContainer


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	if not pack:
		return
	
	pack_label.text = "@%s/%s v%s" % [pack.author, pack.name, pack.version]
	
	for child in card_container.get_children():
		child.queue_free()
	
	for card in pack.cards:
		var pack_card = PACK_CARD.instantiate()
		pack_card.card = card
		pack_card.pack = pack
		card_container.add_child(pack_card)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
