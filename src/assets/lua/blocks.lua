-- air = BlockState.new("air", 0);
-- grass = BlockState.new("grass", 1);
-- deepslate = BlockState.new("deepslate", 2);
-- dirt = BlockState.new("dirt", 3);
-- stone = BlockState.new("stone", 4);
-- ikrine_ore = BlockState.new("ikrine_ore", 5);
-- ikrine_block = BlockState.new("ikrine_block", 6);
-- platinum_ore = BlockState.new("platinum_ore", 7);

air = BlockState.new(0, false);
grass = BlockState.new(1, true);
deepslate = BlockState.new(2, true);
dirt = BlockState.new(1, true);
stone = BlockState.new(2, true);
ikrine_ore = BlockState.new(3, true);
ikrine_block = BlockState.new(3, true);
platinum_ore = BlockState.new(3, true);

return { minecraft = {air = air, grass = grass, deepslate = deepslate, dirt = dirt, stone = stone, ikrine_block = ikrine_block, ikrine_ore = ikrine_ore, platinum_ore = platinum_ore} }
