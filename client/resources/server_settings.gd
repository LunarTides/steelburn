class_name ServerSettings
extends Resource

@export var cheats: bool:
	set(value):
		cheats = value
		emit_changed()
