extends Control


@export var ghost_color: Color

@export var title_label: Label
@export var public_key_label: Label
@export var verified_label: Label
@export var usernames_label: Label
@export var usernames_container: HBoxContainer
@export var full_pk_label: Label
@export var trust_button: Button
@export var untrust_all_button: Button

@export var public_key_error_label: Label
@export var usernames_error_label: Label
@export var verified_error_label: Label

var max_checks := 3


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	update_ui()
	
	Net.opponent_identity_verified.connect(func(_id, usern):
		update_ui()
	)
	Net.opponent_identity_failed.connect(func(usern, key_tr, sign_ver):
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


func _on_close_button_pressed() -> void:
	queue_free()


func update_ui():
	var usernames = Net.opponent_usernames
	var key_trusted = Net.opponent_key_trusted
	var signature_verified = Net.opponent_signature_verified
	var ghost := Game.opponent.public_key.is_empty()
	var invalid_color := ghost_color if ghost else Color.RED
	
	var checks = 0
	if key_trusted:
		checks += 1
	if Net.opponent_username_trusted:
		checks += 1
	if signature_verified:
		checks += 1
	
	if ghost:
		title_label.text = "Verification Info"
	else:
		title_label.text = "Verification Info (%d/%d)" % [checks, max_checks]
	
	trust_button.text = "UNTRUST" if key_trusted and Net.opponent_username_trusted else "TRUST"
	trust_button.disabled = ghost
	untrust_all_button.disabled = ghost
	
	var pub_key := Game.opponent.public_key.split("-----").get(2).strip_escapes()
	if ghost:
		public_key_label.text = "Public Key: Ghost"
		public_key_error_label.text = "The opponent hasn't provided a public key."
	else:
		public_key_label.text = "Public Key: %s" % pub_key.substr(pub_key.length() - 16)
		public_key_error_label.text = "The opponent's public key is trusted." if key_trusted else "The opponent's public key is not trusted."
	public_key_label.modulate = Color.GREEN if key_trusted else invalid_color
	public_key_error_label.modulate = Color.GREEN if key_trusted else invalid_color
	
	# NOTE: Don't recompute the getter value each time.
	var username_trusted := Net.opponent_username_trusted
	usernames_label.text = "Username(s):"
	usernames_label.modulate = Color.GREEN if username_trusted else invalid_color
	if ghost:
		usernames_error_label.text = "Ghosts are untraceable."
	else:
		usernames_error_label.text = "Current username is trusted." if username_trusted else "Current username isn't trusted."
	usernames_error_label.modulate = Color.GREEN if username_trusted else invalid_color
	
	# Remove editor preview usernames.
	for child in usernames_container.get_children():
		if child == usernames_label:
			continue
		
		child.queue_free()
	
	if Game.opponent.username not in usernames:
		var label: Label = usernames_label.duplicate()
		label.text = Game.opponent.username + ("," if usernames.size() > 0 else "")
		label.modulate = invalid_color
		usernames_container.add_child(label)
	
	for username: String in usernames:
		var label: Label = usernames_label.duplicate()
		label.text = username + ("," if username != usernames.back() else "")
		label.modulate = Color.GREEN if username != Game.opponent.username or username_trusted else invalid_color
		usernames_container.add_child(label)
	
	verified_label.text = "Verified: %s" % ("Yes" if signature_verified else "No")
	verified_label.modulate = Color.GREEN if signature_verified else invalid_color
	if ghost:
		verified_error_label.text = "Ghosts are invisible."
	else:
		verified_error_label.text = "The opponent has proven their ownership of the key." if signature_verified else "The opponent has not proven their ownership of the key."
	verified_error_label.modulate = Color.GREEN if signature_verified else invalid_color
	
	if ghost:
		var text := ""
		for _i in range(1000):
			text += "O" if randf() < 0.5 else "o"
		full_pk_label.text = text
		full_pk_label.modulate = ghost_color.darkened(0.4)
		full_pk_label.clip_text = true
	else:
		full_pk_label.text = Game.opponent.public_key.rstrip("\n")
		full_pk_label.modulate = Color.WHITE
		full_pk_label.clip_text = false


func _on_trust_button_pressed() -> void:
	if Net.opponent_key_trusted and Net.opponent_username_trusted:
		# Untrust
		Net.rsa.remove_trusted_identity(Game.opponent.username, Game.opponent.public_key)
	else:
		# Trust
		var identity := RSAIdentity.create_from_pub(Game.opponent.username, Game.opponent.public_key)
		Net.rsa.add_trusted_identity(identity)


func _on_untrust_all_button_pressed() -> void:
	Net.rsa.remove_all_trusted_identities(Game.opponent.username, Game.opponent.public_key)
