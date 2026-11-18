extends PanelContainer


@export var pack: Pack
@export var card: Card:
	set(value):
		card = value
		card.changed.connect(update_ui)
		update_ui()

@export var name_label: Label
@export var text_label: Label


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func update_ui() -> void:
	name_label.text = "%s (%s) - %s" % [card.name, card.cost, card.type]
	text_label.text = "(No text)" if card.text.is_empty() else card.text


func _on_add_to_hand_button_pressed() -> void:
	var packet := Packet.from({
		"type": "cheat_new_card",
		"data": {
			"author": pack.author,
			"pack": pack.name,
			"id": card.id,
		}
	})
	packet.send()
