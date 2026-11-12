class_name RSAIdentity
extends Resource

@export var username: String:
	set(value):
		username = value
		emit_changed()
@export var public_key: String:
	set(value):
		public_key = value
		emit_changed()
@export var private_key: String:
	set(value):
		private_key = value
		emit_changed()
@export var has_private_key: bool:
	set(value):
		has_private_key = value
		emit_changed()


static func create_own(username: String) -> RSAIdentity:
	var identity := RSAIdentity.new()
	identity.username = username
	
	var crypto := Crypto.new()
	var rsa := crypto.generate_rsa(4096)
	
	identity.private_key = rsa.save_to_string()
	identity.public_key = rsa.save_to_string(true)
	identity.has_private_key = true
	return identity


static func create_from_pub(username: String, public_key: String) -> RSAIdentity:
	var identity := RSAIdentity.new()
	identity.username = username
	identity.public_key = public_key
	identity.has_private_key = false
	return identity


func sign(message: PackedByteArray) -> PackedByteArray:
	if not has_private_key:
		push_error("This key is not a signing key. We do not know its private key.")
		return PackedByteArray()
	
	var crypto := Crypto.new()
	var key := CryptoKey.new()
	key.load_from_string(private_key)
	
	var ctx := HashingContext.new()
	ctx.start(HashingContext.HASH_SHA256)
	ctx.update(message)
	var hash := ctx.finish()
	
	return crypto.sign(HashingContext.HASH_SHA256, hash, key)


func sign_string(message: String) -> PackedByteArray:
	return sign(var_to_bytes(message))


func verify(message: PackedByteArray, signature: PackedByteArray) -> bool:
	return RSA.verify_from_pub_key(public_key, message, signature)


func verify_string(message: String, signature: PackedByteArray) -> bool:
	return verify(var_to_bytes(message), signature)
