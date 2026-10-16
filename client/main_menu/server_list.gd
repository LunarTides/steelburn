extends VBoxContainer

@export var connect_button: Button

var selected: Server
var selected_node: Button

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	for server in get_children():
		server.pressed.connect(_on_server_clicked.bind(server))


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func _on_server_clicked(server: Button):
	if server == selected_node:
		# Pressed the selected server. Deselect.
		selected_node = null
		connect_button.disabled = true
		return
	
	if selected_node:
		selected_node.button_pressed = false
	
	selected = server.server
	selected_node = server
	
	connect_button.disabled = false


func _on_connect_button_pressed() -> void:
	print("Connecting to %s..." % selected.address)
	Socket.connect_to(selected)
	Socket.connected.connect(func(_server: Server):
		# TODO: Allow player to customize this.
		# TODO: Don't send if it hasn't been changed since last time.
		var packet = Packet.from({
			"type": "player_info",
			"data": {
				"deckcode": "debug",
				#"username": "en",
			},
		})
		Socket.send_packet(packet)
	, CONNECT_ONE_SHOT)
