--     pub fn density_of(&self, x : f64, y : f64, z : f64, height : f64, weight_bias : f64, height_bias : f64) -> BlockType {
        local weight_bias = 0.4;
        local height_bias = 1.5;
        if y > height + 100.0 then
            return AIR
        end
        if y < height - 100.0 then
            return GRASS
        end
        local noise = 0.0;
        local offset = 0.02;
        local frequency = 0.03;
        local power = 4.0 / 7.0;
        for i = 0,2 do
            noise = noise + generator:get_perlin_3d((x + offset) * frequency, (y + offset) * frequency, (z + offset) * frequency) * power;
            frequency = frequency * 0.2;
            power = power / 2.0;
            offset = offset * 4.0;
        end

        noise = generator:interpolate_noise(noise, y, height, weight_bias, height_bias);

        if noise > 0.0 then
            if y < 0.0 then
                return PLATINUM_ORE
            elseif y < height then
                return AIR
            else
                return AIR
            end
        else
            if y >= height then
                return GRASS
            else
                return GRASS
            end
        end
