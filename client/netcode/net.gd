extends Node

signal opponent_identity_verified(identity: RSAIdentity, usernames: Array)
signal opponent_identity_failed(trusted_usernames: Array[String], key_trusted: bool, signature_verified: bool)

var rsa: RSA
var opponent_key_trusted: bool = false
var opponent_signature_verified: bool = false
var opponent_usernames: Array = []
var opponent_username_trusted: bool:
	get:
		return opponent_usernames.any(func(username: String) -> bool: return username == Game.opponent.username)
var opponent_verified: bool:
	get:
		return opponent_signature_verified and opponent_key_trusted and opponent_username_trusted


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	rsa = RSA.load_identities()
	
	# TODO: Remove
	rsa.trusted_identites = rsa.user_identities
	#if rsa.user_identities.size() <= 0:
		#var identity := RSAIdentity.create_own("en")
		#rsa.add_identity(identity)
	
	Game.started.connect(func():
		send_identity_verification()
		verify_opponent_identity()
	)
	Game.ended.connect(rsa.save_identities)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		rsa.save_identities()


func send_identity_verification():
	var crypto := Crypto.new()
	var message := crypto.generate_random_bytes(512)
	var signature := Net.rsa.current_identity.sign(message)
	
	var packet = Packet.from({
		"type": "verify_identity",
		"data": {
			"message": Marshalls.raw_to_base64(message),
			"signature": Marshalls.raw_to_base64(signature),
		},
	})
	Socket.send_packet(packet)


func verify_opponent_identity():
	Socket.packed_recieved.connect(func(packet: ResponsePacket):
		if packet.type != "verify_identity":
			return
		
		var trusted_usernames := Net.rsa.get_trusted_usernames_from_pub_key(Game.opponent.public_key)
		var key_trusted := Net.rsa.trusts_pub_key(Game.opponent.public_key)
		
		var message := Marshalls.base64_to_raw(packet.data.message)
		var signature := Marshalls.base64_to_raw(packet.data.signature)
		
		var verified := RSA.verify_from_pub_key(Game.opponent.public_key, message, signature)
		
		#print("Player %s - " % Game.player.is_starting_player, verified, key_trusted, trusted_usernames)
		
		opponent_key_trusted = key_trusted
		opponent_usernames = trusted_usernames
		opponent_signature_verified = verified
		
		if opponent_verified:
			var identity := RSAIdentity.create_from_pub(Game.opponent.username, Game.opponent.public_key)
			opponent_identity_verified.emit(identity, trusted_usernames)
		else:
			opponent_identity_failed.emit(trusted_usernames, key_trusted, verified)
	)
