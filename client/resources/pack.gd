class_name Pack
extends Resource

@export var name: String:
	set(value):
		name = value
		emit_changed()
@export var author: String:
	set(value):
		author = value
		emit_changed()
@export var description: String:
	set(value):
		description = value
		emit_changed()
@export var license: String:
	set(value):
		license = value
		emit_changed()
@export var version: String:
	set(value):
		version = value
		emit_changed()
@export var cards: Array[Card]:
	set(value):
		cards = value
		emit_changed()


static func create_from_dict(dict: Dictionary) -> Pack:
	var pack := Pack.new()
	for key in dict.keys():
		var value = dict[key]
		
		if key == "cards":
			var c: Array[Card] = []
			for card_json in value:
				var card = Card.create_from_dict(card_json)
				c.append(card)
			pack.set(key, c)
			continue
		
		pack.set(key, value)
	return pack
