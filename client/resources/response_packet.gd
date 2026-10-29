class_name ResponsePacket
extends Resource

@export var type: String:
	set(value):
		type = value
		emit_changed()
@export var id: String:
	set(value):
		id = value
		emit_changed()
@export var is_starting_player: bool:
	set(value):
		is_starting_player = value
		emit_changed()
@export var data: Dictionary:
	set(value):
		data = value
		emit_changed()
@export var error: String:
	set(value):
		error = value
		emit_changed()
@export var changes: Dictionary:
	set(value):
		changes = value
		emit_changed()


static func from(dict: Dictionary) -> ResponsePacket:
	var packet = ResponsePacket.new()
	packet.type = dict.get("type")
	var new_id = dict.get("id")
	if new_id:
		packet.id = new_id
	packet.is_starting_player = dict.get("is_starting_player")
	packet.data = dict.get("data")
	var err = dict.get("error")
	if err:
		packet.error = err
	packet.changes = dict.get("changes")
	return packet


func to_dict() -> Dictionary:
	var dict = {}
	dict.set("type", type)
	dict.set("id", id)
	dict.set("is_starting_player", is_starting_player)
	dict.set("data", data)
	dict.set("error", error)
	dict.set("changes", changes)
	return dict 


func to_json() -> String:
	return JSON.stringify(to_dict())


func set_properties(object: Object, copy_from = data):
	for key in copy_from.keys():
		var value = copy_from[key]
		object.set(key, value)


func get_relevant_player() -> Player:
	return Game.player if Game.player.is_starting_player == is_starting_player else Game.opponent


func get_irrelevant_player() -> Player:
	return Game.opponent if Game.player.is_starting_player == is_starting_player else Game.player
