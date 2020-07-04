--[[
  Data Definition
--]]

print('Lua: Hello from data.lua')

data:extend(
  'example',
  {
    {
      name = 'test_1',
    },
    {
      name = 'test_2',
    },
  }
)

print('=== values start')
for mod_name, definitions in pairs(data:table()) do
  for def_name, val in pairs(definitions) do
    print(mod_name, def_name, val.name, val)
  end
end
print('=== values stop')

-- new_table = table.deepcopy(data:table()['core']['example']['test_1'])
-- new_table.name = 'test_3'
-- data:extend('example', { new_table })
