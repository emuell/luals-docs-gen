---@meta


---### constants

---@enum acme.SomeClass.StatusCode
acme.Status = {
    OK = 0,
    ERROR = 1,
}

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

---This function also does something and returns a status code enum.
---@return acme.SomeClass.StatusCode
function acme.SomeClass:function_with_enum_return() end

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
