# Base modifier template with arithmetic
modifier-text = { $type ->
    [additive] { $value } སྤེལ་བའི་ { $stat }
    [multiplicative] { $value } མང་བའི་ { $stat }
    [flat-added] +{ $value } ལ་ { $stat }
    *[other] { $value } { $stat }
}

# Stat names
stat-basic-damage = གནོད་པ
stat-player-speed = རྩེད་མཁན་གྱི་མྱུར་ཚད
stat-projectile-speed = རྒྱུ་ཆའི་མྱུར་ཚད
stat-projectile-count = རྒྱུ་ཆའི་གྲངས
stat-projectile-duration = རྒྱུ་ཆའི་རིང་ཚད
stat-aoe-radius = ཁྱབ་ཁོངས་ཀྱི་ཞེང་ཚད
stat-max-health = བདེ་ཐང་མཐོ་ཤོས
stat-health-regen = བདེ་ཐང་སོར་ཆུད
stat-cooldown-recovery-rate = བསིལ་དུས་སོར་ཆུད་ཀྱི་ཚད
stat-homing-strength = ཁྱིམ་དུ་ལོག་པའི་ནུས

# Game Over Screen
game-over-title = རྩེད་རིགས་ཚར
game-over-final-score = མཐའ་མའི་ཐོབ་འབྲས། { $score }
game-over-enemies-defeated = དགྲ་བོ་ཕམ་པ། { $kills }
game-over-restart-instructions = R ནན་ནས་བསྐྱར་དུ་འགོ་ཚུགས། | ESC ནན་ནས་ཕྱིར་ཐོན།

# HUD Elements
hud-score = ཐོབ་འབྲས། { $score }
hud-combo = x{ $multiplier }
hud-wave = རླབས་ཚན { $number }

# Damage Numbers
damage-number = -{ $damage }

# Player Abilities
ability-dash = མྱུར་འགྲོ
ability-shotgun = མེ་མདའ་ཆེ

# Enemy Abilities
ability-slam = བརྡུང་བ
ability-missile = མདའ་འཕེན

# Watermark
watermark-dev-build = འཕྲུལ་མཁན་རིགས་ v{ $version }

# Stats Screen
stats-screen-title = རྩིས་ཐོ
stats-screen-instructions = TAB བཟུང་ནས་རྩིས་ཐོ་ལྟ