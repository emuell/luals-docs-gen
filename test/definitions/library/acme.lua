---@meta

--------------------------------------------------------------------------------
-- ## acme

---Holds all acme related API test functions and classes.
---@class acme
acme = {}

---### constants

---This is a const
---@type number
acme.API_VERSION = 6.1

---### functions

---Global function docs
function acme.global_function() end

---More global function docs
---@return GlobalTestClass1|GlobalTestClass2
function acme.global_function2() end

--------------------------------------------------------------------------------

---@class GlobalTestClass1
---@field field1 number
---@field field2 string
local GlobalTestClass1 = {}


---@class GlobalTestClass2
---@field alias SomeAlias
---@field field GlobalTestClass1
local GlobalTestClass2 = {}

---This is an alias
---@alias SomeAlias string
