class_name Server
extends Resource

@export var name: String:
	set(value):
		name = value
		emit_changed()
@export var address: String:
	set(value):
		address = value
		emit_changed()
@export var secure: bool:
	set(value):
		secure = value
		emit_changed()
@export var players_connected: int:
	set(value):
		players_connected = value
		emit_changed()
@export var max_players_connected: int:
	set(value):
		max_players_connected = value
		emit_changed()
@export var ping: int:
	set(value):
		ping = value
		emit_changed()
