extends Node

signal connected(server: Server)
# TODO: Handle diconnects during game.
signal disconnected(server: Server, code: int, reason: String, clean: bool)
signal packed_sent(packet: Packet)
signal packed_recieved(packet: ResponsePacket)

var socket = WebSocketPeer.new()
var server: Server
var is_connecting_to_server = false
var is_connected_to_server = false


func _process(delta):
	if not is_connecting_to_server and not is_connected_to_server:
		return
	
	socket.poll()
	var state = socket.get_ready_state()
	if state == WebSocketPeer.STATE_OPEN:
		if not is_connected_to_server:
			connected.emit(server)
		
		is_connecting_to_server = false
		is_connected_to_server = true
		
		while socket.get_available_packet_count():
			var json = socket.get_packet().get_string_from_utf8()
			print()
			print(json)
			var packet = ResponsePacket.from(JSON.parse_string(json))
			packed_recieved.emit(packet)
	elif state == WebSocketPeer.STATE_CLOSING:
		# Keep polling to achieve proper close.
		pass
	elif state == WebSocketPeer.STATE_CLOSED:
		var code = socket.get_close_code()
		var reason = socket.get_close_reason()
		print("WebSocket closed with code: %d, reason %s. Clean: %s" % [code, reason, code != -1])
		socket = WebSocketPeer.new()
		is_connecting_to_server = false
		is_connected_to_server = false
		disconnected.emit(server, code, reason, code != -1)


func connect_to(srv: Server):
	close()
	is_connecting_to_server = true
	server = srv
	# TODO: Add timeout.
	socket.connect_to_url("%s://%s" % ["wss" if server.secure else "ws", server.address])


func close():
	var ready_state = socket.get_ready_state()
	if is_connected_to_server or (ready_state != WebSocketPeer.STATE_CLOSING and ready_state != WebSocketPeer.STATE_CLOSED):
		socket.close()
	is_connected_to_server = false
	is_connecting_to_server = false


func send_packet(packet: Packet, randomize_packet_id: bool = true):
	if randomize_packet_id:
		packet.id = Packet.get_random_id()
	
	socket.send_text(packet.to_json())
	packed_sent.emit(packet)
