extends HBoxContainer

@export var username: String:
	set(value):
		username = value
		username_label.text = username
@export var message: String:
	set(value):
		message = value
		message_label.text = message
@export var id: String
@export var confirmed: bool:
	set(value):
		confirmed = value
		message_label.modulate.a = 1.0 if confirmed else 0.2

@export var username_label: Label
@export var message_label: Label


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	# NOTE: Do side effects
	confirmed = confirmed
	
	Socket.packed_recieved.connect(_on_packet_recieved)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func _on_packet_recieved(packet: ResponsePacket) -> void:
	if packet.type != "chat_message" or confirmed or packet.id != id:
		return
	
	username = Game.starting_player.username if packet.is_starting_player else Game.starting_player.get_opponent().username
	message = packet.data.text
	confirmed = true
