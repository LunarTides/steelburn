extends TextureButton


const VERIFICATION_INFO_UI = preload("uid://44y1vtwtxmhy")
const LOCK_KEYHOLE = preload("uid://fbcdwumscjvj")
const LOCK_KEYHOLE_OFF = preload("uid://bd5bljqxel0ju")


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	update_ui()
	
	Net.opponent_identity_verified.connect(func(identity: RSAIdentity, usernames: Array):
		update_ui()
	)
	Net.rsa.trusted_identity_added.connect(func(identity: RSAIdentity):
		update_ui()
	)
	Net.rsa.trusted_identity_removed.connect(func(identity: RSAIdentity):
		update_ui()
	)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func update_ui():
	if Net.opponent_verified:
		texture_normal = LOCK_KEYHOLE
		modulate = Color.GREEN
	else:
		texture_normal = LOCK_KEYHOLE_OFF
		modulate = Color.hex(0xa0a0a0ff) if Game.opponent.public_key.is_empty() else Color.RED


func _on_pressed() -> void:
	for child in get_children():
		child.queue_free()
	
	var ui: Control = VERIFICATION_INFO_UI.instantiate()
	add_child(ui)
