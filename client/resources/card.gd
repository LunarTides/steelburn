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
@export var is_starting_player: bool:
	set(value):
		is_starting_player = value
		emit_changed()
@export var hash: String:
	set(value):
		hash = value
		emit_changed()


var owner: Player:
	get:
		return Game.starting_player if is_starting_player else Game.starting_player.get_opponent()


static func create_from_dict(dict: Dictionary) -> Card:
	var card := Card.new()
	for key in dict.keys():
		var value = dict[key]
		card.set(key, value)
	card.add_to_location()
	return card


static func find_from_hash(hash: String) -> Card:
	for player: Player in [Game.player, Game.opponent]:
		# NOTE: For some reaon, `find` doesn't work here.
		var hand_index = player.hand.filter(func(c: Card): return c and c.hash == hash)
		if hand_index.size() > 0:
			return hand_index[0]
		
		var board_index = player.board.filter(func(c: Card): return c and c.hash == hash)
		if board_index.size() > 0:
			return board_index[0]
	
	return null


func remove_from_location():
	owner.remove_from_location(self)


func add_to_location():
	remove_from_location()
	
	# TODO: Free on Oblivion.
	if location == "Hand":
		owner.hand.append(self)
		owner.hand_changed.emit()
	elif location == "Board":
		owner.board.append(self)
		owner.board_changed.emit()
