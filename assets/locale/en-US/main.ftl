# Base modifier template with arithmetic
modifier-text = { $type ->
    [additive] { $value } increased { $stat }
    [multiplicative] { $value } more { $stat }
    *[other] { $value } { $stat }
}

# Stat names
stat-basic-damage = damage
stat-player-speed = player speed
stat-projectile-speed = projectile speed
stat-projectile-count = projectile count
stat-projectile-duration = projectile duration
stat-aoe-radius = AoE radius

# Game Over Screen
game-over-title = GAME OVER
game-over-final-score = Final Score: { $score }
game-over-enemies-defeated = Enemies Defeated: { $kills }
game-over-restart-instructions = Press R to Restart | Press ESC to Quit

# HUD Elements
hud-score = Score: { $score }
hud-combo = x{ $multiplier }
hud-wave = Wave { $number }

# Damage Numbers
damage-number = -{ $damage }

# Player Abilities
ability-dash = Dash
ability-shotgun = Shotgun

# Enemy Abilities
ability-slam = Slam
ability-missile = Missile

# Watermark
watermark-dev-build = Developer Build v{ $version }