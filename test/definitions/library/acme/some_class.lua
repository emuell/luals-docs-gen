---@meta

--------------------------------------------------------------------------------
---## acme.SomeClass

---@class acme.SomeClass
---@field some_field boolean
acme.SomeClass = {}
acme.SomeClass.__index = acme.SomeClass

---### constants

---SOME_CONSTANT docs
acme.SomeClass.SOME_CONSTANT = 4

---This function does something.
function acme.SomeClass:some_function() end

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
