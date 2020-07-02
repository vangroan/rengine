--[[
  Data Definition
--]]

print('Lua: Hello from data.lua')

data:extend({
  {
    name = 'test 1',
  },
  {
    name = 'test 2',
  },
})

print('=== values start')
for mod_name, definitions in pairs(data:table()) do
  for def_name, val in pairs(definitions) do
    print(mod_name, def_name, val)
  end
end
print('=== values stop')

new_table = table.deepcopy(data:table()['core']['test 1'])
new_table.name = 'test 3'
data:extend({ new_table })
