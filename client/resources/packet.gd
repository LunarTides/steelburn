class_name Packet
extends Resource

@export var type: String:
	set(value):
		type = value
		emit_changed()
@export var data: Dictionary:
	set(value):
		data = value
		emit_changed()


static func from(dict: Dictionary) -> Packet:
	var packet = Packet.new()
	packet.type = dict.get("type")
	packet.data = dict.get("data")
	return packet


func to_dict() -> Dictionary:
	var dict = {}
	dict.set("type", type)
	dict.set("data", data)
	return dict 


func to_json() -> String:
	return JSON.stringify(to_dict())


func set_properties(object: Object, copy_from = data):
	for key in copy_from.keys():
		var value = copy_from[key]
		object.set(key, value)


func is_response() -> bool:
	return data.has("is_starting_player")


func get_relevant_player() -> Player:
	if not is_response():
		return
	
	var is_starting_player = data.is_starting_player
	return Game.player if Game.player.is_starting_player == is_starting_player else Game.opponent


func get_irrelevant_player() -> Player:
	if not is_response():
		return
	
	var is_starting_player = data.is_starting_player
	return Game.opponent if Game.player.is_starting_player == is_starting_player else Game.player
