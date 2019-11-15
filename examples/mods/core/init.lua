
--[[
    Example mod
    ===========

    The name of the top level `rengine` table is determined by
    the string passed to `AppBuilder.add_modding`.

    This table acts as the main library interface to the engine.
]]--

-- Engine version
print(rengine.version)

--[[
    Define an entity.
    
    The name of the definition must be prepended
    with the module's directory name.
]] 
rengine.register_entity('core:plant', {

})
