class_name RSA
extends Resource


signal trusted_identity_added(identity: RSAIdentity)
signal trusted_identity_removed(identity: RSAIdentity)

@export var current_identity: RSAIdentity
@export var user_identities: Array[RSAIdentity]:
	set(value):
		user_identities = value
		emit_changed()
@export var trusted_identites: Array[RSAIdentity]:
	set(value):
		trusted_identites = value
		emit_changed()


static func verify_from_pub_key(public_key: String, message: PackedByteArray, signature: PackedByteArray) -> bool:
	var crypto := Crypto.new()
	var key := CryptoKey.new()
	key.load_from_string(public_key, true)
	
	var ctx := HashingContext.new()
	ctx.start(HashingContext.HASH_SHA256)
	ctx.update(message)
	var hash := ctx.finish()
	
	return crypto.verify(HashingContext.HASH_SHA256, hash, signature, key)


func add_identity(identity: RSAIdentity):
	user_identities.append(identity)
	current_identity = identity


func add_trusted_identity(identity: RSAIdentity):
	trusted_identites.append(identity)
	
	Net.opponent_usernames.push_front(Game.opponent.username)
	Net.opponent_key_trusted = identity.public_key == Game.opponent.public_key
	
	trusted_identity_added.emit(identity)


func remove_trusted_identity(username: String, public_key: String):
	var index = trusted_identites.find_custom(func(identity: RSAIdentity) -> bool: return identity.username == username && identity.public_key == public_key)
	if index == -1:
		return
	
	var identity = trusted_identites[index]
	trusted_identites.remove_at(index)
	
	Net.opponent_usernames.erase(username)
	Net.opponent_key_trusted = trusts_pub_key(public_key)
	
	trusted_identity_removed.emit(identity)


func remove_all_trusted_identities(username: String, public_key: String):
	var identities = trusted_identites.filter(func(identity: RSAIdentity) -> bool: return identity.username == username || identity.public_key == public_key)
	Net.opponent_key_trusted = false
	Net.opponent_usernames.clear()
	
	for identity: RSAIdentity in identities:
		trusted_identites.erase(identity)
		trusted_identity_removed.emit(identity)


func get_trusted_usernames_from_pub_key(public_key: String) -> Array:
	var usernames := trusted_identites.filter(func(identity: RSAIdentity) -> bool: return identity.public_key == public_key).map(func(identity: RSAIdentity) -> String: return identity.username)
	usernames.reverse()
	return usernames


func trusts_pub_key(public_key: String) -> bool:
	return trusted_identites.any(func(identity: RSAIdentity) -> bool: return identity.public_key == public_key)


static func load_identities() -> RSA:
	if not FileAccess.file_exists("user://rsa.tres"):
		return
	
	return load("user://rsa.tres")


func save_identities():
	ResourceSaver.save(self, "user://rsa.tres")
