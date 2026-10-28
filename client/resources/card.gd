class_name Card
extends Resource

@export var name: String:
	set(value):
		name = value
		emit_changed()
@export var cost: int:
	set(value):
		cost = value
		emit_changed()
@export var text: String:
	set(value):
		text = value
		emit_changed()
@export var location: String:
	set(value):
		location = value
		emit_changed()
@export var id: String:
	set(value):
		id = value
		emit_changed()
@export var hash: String:
	set(value):
		hash = value
		emit_changed()
