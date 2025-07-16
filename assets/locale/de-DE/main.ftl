# Base modifier template with arithmetic
modifier-text = { $type ->
    [additive] { $stat } erhöht um { $value }
    [multiplicative] { $value } mehr { $stat }
    *[other] { $value } { $stat }
}

# Stat names
stat-basic-damage = Schaden
stat-player-speed = Spielergeschwindigkeit
stat-projectile-speed = Projektilgeschwindigkeit
stat-projectile-count = Projektilanzahl
stat-projectile-duration = Projektildauer
stat-aoe-radius = Flächenschadenradius

# Game Over Screen
game-over-title = SPIEL VORBEI
game-over-final-score = Endpunktzahl: { $score }
game-over-enemies-defeated = Besiegte Feinde: { $kills }
game-over-restart-instructions = R drücken zum Neustarten | ESC drücken zum Beenden

# HUD Elements
hud-score = Punkte: { $score }
hud-combo = x{ $multiplier }
hud-wave = Welle { $number }

# Damage Numbers
damage-number = -{ $damage }

# Player Abilities
ability-dash = Sprint
ability-shotgun = Schrotflinte

# Enemy Abilities
ability-slam = Schlag
ability-missile = Rakete

# Watermark
watermark-dev-build = Entwicklerversion v{ $version }