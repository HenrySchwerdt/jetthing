-- Loaded with require("util"); modules resolve inside the plugin directory.
local util = {}

function util.greeting(word, name)
  return string.format("%s, %s!", word or "Hello", name)
end

return util
