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
var verification_message: PackedByteArray


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	rsa = RSA.load_identities()
	
	# TODO: Remove
	if rsa.user_identities.size() <= 0:
		var identity := RSAIdentity.create_own("en")
		rsa.add_identity(identity)
	rsa.trusted_identites = rsa.user_identities
	
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
	# Send an encrypted message to the opponent.
	# The message is encrypted using their supposed public key.
	# If they respond with the decrypted message, we know they have the corresponding private key.
	# The opponent is then verified.
	var crypto := Crypto.new()
	verification_message = crypto.generate_random_bytes(128)
	var key := RSAIdentity.create_from_pub(Game.opponent.username, Game.opponent.public_key)
	var encrypted := key.encrypt(verification_message)
	
	var packet = Packet.from({
		"type": "request_identity_verification",
		"data": {
			"encrypted_message": Marshalls.raw_to_base64(encrypted),
		},
	})
	packet.send()


func verify_opponent_identity():
	Socket.packed_recieved.connect(func(packet: ResponsePacket):
		if packet.type == "request_identity_verification":
			var encrypted_message := Marshalls.base64_to_raw(packet.data.encrypted_message)
			var decrypted_message := Net.rsa.current_identity.decrypt(encrypted_message)
			var new_packet = Packet.from({
				"type": "verify_identity",
				"data": {
					"decrypted_message": Marshalls.raw_to_base64(decrypted_message),
				},
			})
			new_packet.send()
		elif packet.type == "verify_identity":
			var trusted_usernames := Net.rsa.get_trusted_usernames_from_pub_key(Game.opponent.public_key)
			var key_trusted := Net.rsa.trusts_pub_key(Game.opponent.public_key)
			var decrypted_message = Marshalls.base64_to_raw(packet.data.decrypted_message)
			var verified = decrypted_message == verification_message
			
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
