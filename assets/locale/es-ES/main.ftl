# Base modifier template with arithmetic
modifier-text = { $type ->
    [additive] { $value } aumentado { $stat }
    [multiplicative] { $value } más { $stat }
    [flat-added] +{ $value } a { $stat }
    *[other] { $value } { $stat }
}

# Stat names
stat-basic-damage = daño
stat-player-speed = velocidad del jugador
stat-projectile-speed = velocidad del proyectil
stat-projectile-count = cantidad de proyectiles
stat-projectile-duration = duración del proyectil
stat-aoe-radius = radio de efecto de área
stat-max-health = salud máxima
stat-health-regen = regeneración de salud
stat-cooldown-recovery-rate = tasa de recuperación de enfriamiento
stat-homing-strength = fuerza de seguimiento

# Game Over Screen
game-over-title = JUEGO TERMINADO
game-over-final-score = Puntuación Final: { $score }
game-over-enemies-defeated = Enemigos Derrotados: { $kills }
game-over-restart-instructions = Presiona R para Reiniciar | Presiona ESC para Salir

# HUD Elements
hud-score = Puntuación: { $score }
hud-combo = x{ $multiplier }
hud-wave = Oleada { $number }

# Damage Numbers
damage-number = -{ $damage }

# Player Abilities
ability-dash = Dash
ability-shotgun = Escopeta

# Enemy Abilities
ability-slam = Golpe
ability-missile = Misil

# Watermark
watermark-dev-build = Versión de Desarrollo v{ $version }

# Stats Screen
stats-screen-title = ESTADÍSTICAS
stats-screen-instructions = Mantén TAB para ver estadísticas