
--[[
    Example mod
    ===========

    The name of the top level `rengine` table is determined by
    the string passed to `AppBuilder.add_modding`.

    This table acts as the main library interface to the engine.
]]--

print('======== Core ========')

-- Engine version
print('Engine Version: ' .. rengine.version)

--[[
    Define an entity.
    
    The name of the definition must be prepended
    with the module's directory name.
]] 
rengine.register_entity('core:plant', {

})

function rengine.on_start()
    print('Start scene script')
    skelly.spawn_skelly(12, 12, 8)
    skelly.spawn_skelly(13, 12, 8)
    skelly.spawn_skelly(14, 12, 8)
end

function rengine.on_stop()
    print('Stop scene script')
end

