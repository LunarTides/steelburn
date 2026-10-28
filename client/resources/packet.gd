class_name Packet
extends Resource

@export var type: String:
	set(value):
		type = value
		emit_changed()
@export var id: String:
	set(value):
		id = value
		emit_changed()
@export var data: Dictionary:
	set(value):
		data = value
		emit_changed()


static func get_random_id() -> String:
	var result: String = ""
	
	for i in 64:
		result += "abcdefghijklmnopqrstuvxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_".split("")[randf_range(0, 63)]
	
	return result


static func from(dict: Dictionary) -> Packet:
	var packet = Packet.new()
	packet.type = dict.get("type")
	packet.id = dict.get("id", Packet.get_random_id())
	packet.data = dict.get("data")
	return packet


func to_dict() -> Dictionary:
	var dict = {}
	dict.set("type", type)
	dict.set("id", id)
	dict.set("data", data)
	return dict 


func to_json() -> String:
	return JSON.stringify(to_dict())


func set_properties(object: Object, copy_from = data):
	for key in copy_from.keys():
		var value = copy_from[key]
		object.set(key, value)
