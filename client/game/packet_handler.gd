class_name GamePacketHandler
extends Object


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	Socket.packed_recieved.connect(_on_packet_recieved)


func _on_packet_recieved(packet: ResponsePacket):
	handle_changes(packet)
	
	var f = get(packet.type)
	if f:
		f.call(packet)


func handle_changes(packet: ResponsePacket):
	var player1 = packet.changes.get("Player 1")
	if player1:
		for k in player1.keys():
			var v = player1[k]
			Game.starting_player[k] = v
	
	var player2 = packet.changes.get("Player 2")
	var relevant_player = Game.starting_player.get_opponent()
	if player2:
		for k in player2.keys():
			var v = player2[k]
			relevant_player[k] = v


func player_info(packet: ResponsePacket):
	Game.player.set_properties(packet)


func opponent_player_info(packet: ResponsePacket):
	Game.opponent.set_properties(packet)


func begin_game(packet: ResponsePacket):
	Game.player.is_starting_player = packet.is_starting_player
	Game.opponent.is_starting_player = not packet.is_starting_player
	
	# TODO: Maybe don't assume this on client.
	Game.starting_player.mana = 1
	
	print("Game started.")
	Game.start()


func end_game(packet: ResponsePacket):
	var reason = packet.response_data.reason
	Game.end(reason)


func draw_cards(packet: ResponsePacket):
	var cards = packet.response_data.cards
	
	for card_json in cards:
		var card := Card.new()
		packet.set_properties(card, card_json)
		
		Game.player.hand.append(card)
		Game.player.hand_changed.emit()


func opponent_draw_cards(packet: ResponsePacket):
	var amount = packet.response_data.amount
	var null_array = []
	null_array.resize(amount)
	null_array.fill(null)
	Game.opponent.hand.append_array(null_array)
	Game.opponent.hand_changed.emit()


func end_turn(packet: ResponsePacket):
	Game.our_turn = not Game.our_turn
	Game.turn_ended.emit()


func play_card(packet: ResponsePacket):
	var card := Card.new()
	packet.set_properties(card, packet.response_data.card)
	
	var player = packet.get_relevant_player()
	if player.is_opponent:
		player.hand.pop_back()
		player.hand_changed.emit()
	else:
		player.remove_from_location(card)
	
	player.board.append(card)
	player.board_changed.emit()
