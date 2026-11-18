class_name GamePacketHandler
extends Object


signal error_recieved(error: String)


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	Socket.packed_recieved.connect(_on_packet_recieved)


func _on_packet_recieved(packet: ResponsePacket):
	handle_changes(packet)
	
	if packet.error:
		error_recieved.emit(packet.error)
		return
	
	var f = get(packet.type)
	if f:
		f.call(packet)


func handle_changes(packet: ResponsePacket):
	for key in packet.changes.keys():
		var val = packet.changes[key]
		
		if key == "Player 1":
			for k in val.keys():
				var v = val[k]
				Game.starting_player[k] = v
			continue
		
		if key == "Player 2":
			var relevant_player = Game.starting_player.get_opponent()
			for k in val.keys():
				var v = val[k]
				relevant_player[k] = v
			continue
		
		# Cards
		var card = Card.find_from_hash(key)
		if not card:
			continue
		
		for k in val.keys():
			var v = val[k]
			card[k] = v
			
			if k == "location":
				card.add_to_location()


func player_info(packet: ResponsePacket):
	Game.player.set_properties(packet)


func opponent_player_info(packet: ResponsePacket):
	Game.opponent.set_properties(packet)


func begin_game(packet: ResponsePacket):
	Game.player.is_starting_player = packet.is_starting_player
	Game.opponent.is_starting_player = not packet.is_starting_player
	
	print("Game started.")
	Game.start()


func end_game(packet: ResponsePacket):
	var reason = packet.data.reason
	Game.end(reason)


func draw_cards(packet: ResponsePacket):
	var cards = packet.data.cards
	
	# TODO: Ew. The cards should probably always be an array.
	if typeof(cards) == TYPE_DICTIONARY:
		Card.create_from_dict(cards)
		return
	
	for card_json in cards:
		Card.create_from_dict(card_json)


func opponent_draw_cards(packet: ResponsePacket):
	pass


func new_cards(packet: ResponsePacket):
	var cards = packet.data.get("cards")
	if cards:
		# TODO: Ew. The cards should probably always be an array.
		if typeof(cards) == TYPE_DICTIONARY:
			Card.create_from_dict(cards)
			return
		
		for card_json in cards:
			Card.create_from_dict(card_json)


func end_turn(packet: ResponsePacket):
	Game.our_turn = not Game.our_turn
	Game.turn_ended.emit()


func play_card(packet: ResponsePacket):
	var player = packet.get_relevant_player()
	if not player.is_opponent:
		return
	
	Card.create_from_dict(packet.data.card)
