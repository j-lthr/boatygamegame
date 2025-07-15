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