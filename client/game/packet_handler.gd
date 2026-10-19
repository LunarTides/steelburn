class_name GamePacketHandler
extends Object


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	Socket.packed_recieved.connect(_on_packet_recieved)


func _on_packet_recieved(packet: Packet):
	var f = get(packet.type)
	if f:
		f.call(packet)


func player_info(packet: Packet):
	Game.player.set_properties(packet)


func opponent_player_info(packet: Packet):
	Game.opponent.set_properties(packet)


func begin_game(packet: Packet):
	var is_starting_player = packet.data.is_starting_player
	Game.player.is_starting_player = is_starting_player
	Game.opponent.is_starting_player = not is_starting_player
	
	# TODO: Maybe don't assume this on client.
	Game.starting_player.mana = 1
	
	print("Game started.")
	Game.start()


func end_game(packet: Packet):
	var reason = packet.data.reason
	Game.end(reason)


func draw_cards(packet: Packet):
	var cards = packet.data.cards
	
	for card_json in cards:
		var card := Card.new()
		packet.set_properties(card, card_json)
		
		Game.player.hand.append(card)
		Game.player.hand_changed.emit()


func opponent_draw_cards(packet: Packet):
	var amount = packet.data.amount
	var null_array = []
	null_array.resize(amount)
	null_array.fill(null)
	Game.opponent.hand.append_array(null_array)
	Game.opponent.hand_changed.emit()


func end_turn(packet: Packet):
	Game.our_turn = not Game.our_turn
	
	var player = packet.get_irrelevant_player()
	packet.set_properties(player, packet.data.opponent)
	
	Game.turn_ended.emit()


func play_card(packet: Packet):
	var card := Card.new()
	packet.set_properties(card, packet.data.card)
	
	var player = packet.get_relevant_player()
	packet.set_properties(player, packet.data.player)
	
	if player.is_opponent:
		player.hand.pop_back()
		player.hand_changed.emit()
	else:
		player.remove_from_location(card)
	
	player.board.append(card)
	player.board_changed.emit()
