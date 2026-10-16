class_name Player
extends Resource

signal hand_changed
signal board_changed

@export var username: String:
	set(value):
		username = value
		emit_changed()
@export var is_starting_player: bool:
	set(value):
		is_starting_player = value
		emit_changed()

# NOTE: Elements are either Card or null.
var hand: Array[Variant]
var board: Array[Card]


func set_properties(packet: Packet):
	if packet.type != "player_info" and packet.type != "opponent_player_info":
		push_error("Invalid packet for populating player info. (%s)" % packet.type)
		return
	
	packet.set_properties(self)
