func _ready():
	Socket.packed_recieved.connect(_on_packet_recieved)


func _on_packet_recieved(packet: ResponsePacket):
	if packet.type == "gamma_high_score_start":
		var text = packet.data.text
		print(text)
