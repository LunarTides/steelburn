extends PanelContainer

const CHAT_MESSAGE = preload("uid://baydnsnagxdrc")

@export var message: LineEdit
@export var chat_messages_container: VBoxContainer
@onready var scroll_container: ScrollContainer = $VBoxContainer/ScrollContainer


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	for child in chat_messages_container.get_children():
		child.queue_free()
	
	Socket.packed_recieved.connect(_on_packet_recieved)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func _on_send_button_pressed() -> void:
	var packet := Packet.from({
		"type": "chat_message",
		"data": {
			"text": message.text,
		},
	})
	Socket.send_packet(packet)
	
	message.clear()
	message.edit()


func _on_message_text_submitted(new_text: String) -> void:
	_on_send_button_pressed()


func _on_packet_recieved(packet: ResponsePacket) -> void:
	if packet.type != "chat_message":
		return
	
	var text = packet.data.text
	
	var chat_message := CHAT_MESSAGE.instantiate()
	chat_message.get_node(^"Username").text = Game.starting_player.username if packet.is_starting_player else Game.starting_player.get_opponent().username
	chat_message.get_node(^"Message").text = text
	chat_messages_container.add_child(chat_message)
	
	await get_tree().process_frame
	scroll_container.scroll_vertical = INT32_MAX
