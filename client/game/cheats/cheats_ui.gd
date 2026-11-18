extends Control

const PACKS_CONTAINER = preload("uid://dyyu7ixuvj0b0")

@export var cards_container: VBoxContainer
@export var add_author: LineEdit
@export var add_pack: LineEdit
@export var add_card: LineEdit

@export var starting_player_check_box: CheckBox
@export var set_mana: LineEdit

var packs_container: Control


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	if not Game.server_settings.cheats:
		queue_free()
	
	get_packs()
	
	Socket.packed_recieved.connect(_on_packet_recieved)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func _on_card_name_text_submitted(new_text: String) -> void:
	add_card_to_hand()


func _on_add_to_hand_button_pressed() -> void:
	add_card_to_hand()


func add_card_to_hand() -> void:
	var packet := Packet.from({
		"type": "cheat_new_card",
		"data": {
			"author": add_author.text.trim_prefix("@"),
			"pack": add_pack.text,
			"id": add_card.text,
		}
	})
	packet.send()


func get_packs():
	var packet := Packet.from({
		"type": "cheat_get_packs",
		"data": {}
	})
	packet.send()


func _on_refresh_packs_button_pressed() -> void:
	get_packs()


func _on_set_mana_text_submitted(new_text: String) -> void:
	set_player_mana()


func _on_set_mana_button_pressed() -> void:
	set_player_mana()


func set_player_mana() -> void:
	var packet := Packet.from({
		"type": "cheat_set_mana",
		"data": {
			"is_starting_player": starting_player_check_box.button_pressed != Game.player.is_starting_player,
			"mana": set_mana.text.to_int(),
		}
	})
	packet.send()


func _on_end_turn_button_pressed() -> void:
	var packet := Packet.from({
		"type": "cheat_end_turn",
		"data": {}
	})
	packet.send()


func _on_packet_recieved(packet: ResponsePacket):
	if packet.type == "cheat_get_packs":
		var packs_json = packet.data.packs
		
		var packs: Array[Pack] = []
		for pack_json in packs_json:
			var pack = Pack.create_from_dict(pack_json)
			packs.append(pack)
		
		if packs_container:
			packs_container.queue_free()
		
		packs_container = PACKS_CONTAINER.instantiate()
		packs_container.packs = packs
		cards_container.add_child(packs_container)
