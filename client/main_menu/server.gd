extends Button

@export var server: Server
@export var name_label: Label
@export var address_label: Label
@export var connected_label: Label
@export var ping_label: Label


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	text = "."
	for i in ["font_color", "font_focus_color", "font_pressed_color", "font_hover_color", "font_hover_pressed_color", "font_disabled_color"]:
		add_theme_color_override(i, Color.hex(0xffffff00))
	
	update_ui()
	server.changed.connect(update_ui)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func update_ui() -> void:
	name_label.text = server.name
	address_label.text = server.address
	connected_label.text = "%s/%s" % [server.players_connected, server.max_players_connected]
	ping_label.text = "%sms" % server.ping
