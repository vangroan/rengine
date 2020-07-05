
print('======== Skelly ========')
print('Init Skelly Mod')

--[[
  Test for spawning entities
--]]

function on_init()
  GAME:spawn_soldier('skelly:soldier:skelly_soldier', { position = { 0.5, 8.0 + 0.5, 15.5 } });
end
