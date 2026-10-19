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
var is_opponent: bool:
	get:
		return is_starting_player == Game.opponent.is_starting_player


func set_properties(packet: Packet):
	if packet.type != "player_info" and packet.type != "opponent_player_info":
		push_error("Invalid packet for populating player info. (%s)" % packet.type)
		return
	
	packet.set_properties(self)


func remove_from_location(card: Card):
	# NOTE: For some reaon, `find` doesn't work here.
	var hand_index = hand.filter(func(c: Card): return c.hash == card.hash)
	if hand_index.size() > 0:
		hand.erase(hand_index[0])
		hand_changed.emit()
		return
	
	var board_index = board.filter(func(c: Card): return c.hash == card.hash)
	if board_index.size() > 0:
		board.erase(board_index[0])
		board_changed.emit()
		return
