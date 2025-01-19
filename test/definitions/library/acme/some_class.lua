---@meta

--------------------------------------------------------------------------------
---## acme.SomeClass

---@class acme.SomeClass
acme.SomeClass = {}

---### constants

---SOME_CONSTANT docs
acme.SomeClass.SOME_CONSTANT = 4

--------------------------------------------------------------------------------

---SomeClass docs
---@class SomeClassInstance
---@field some_property table<string, integer>
local SomeClassInstance = {}

---SomeClassInstance Docs
---@return SomeClassInstance
function acme.SomeClass() end

---SomeFunction docs
function SomeClassInstance:some_function() end
