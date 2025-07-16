# Base modifier template with arithmetic
modifier-text = { $type ->
    [additive] { $stat } erhöht um { $value }
    [multiplicative] { $value } meh { $stat }
    *[other] { $value } { $stat }
}

# Stat names
stat-basic-damage = Schade
stat-player-speed = Spielergschwindigkeit
stat-projectile-speed = Projektilgschwindigkeit
stat-projectile-count = Projektilazahl
stat-projectile-duration = Projektilduur
stat-aoe-radius = Flächeschaderadius

# Game Over Screen
game-over-title = SPIEL FERTIG
game-over-final-score = Schlussresultat: { $score }
game-over-enemies-defeated = Gschlageni Finde: { $kills }
game-over-restart-instructions = R drücke zum Neustarte | ESC drücke zum Ufhöre

# HUD Elements
hud-score = Pünkt: { $score }
hud-combo = x{ $multiplier }
hud-wave = Welle { $number }

# Damage Numbers
damage-number = -{ $damage }

# Player Abilities
ability-dash = Sprinte
ability-shotgun = Schrotflinete

# Enemy Abilities
ability-slam = Schlag
ability-missile = Rakete

# Watermark
watermark-dev-build = Entwicklerversion v{ $version }