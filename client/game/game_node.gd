extends Node3D


@export var camera: Camera3D


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	Game.camera = camera


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
