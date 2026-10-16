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
	return JSON.stringify(self.to_dict())


func set_properties(object: Object, copy_from = self.data):
	for key in copy_from.keys():
		var value = copy_from[key]
		object.set(key, value)
