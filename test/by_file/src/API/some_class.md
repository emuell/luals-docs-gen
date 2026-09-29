# some_class  
* [acme.SomeClass](#acme.SomeClass)  
	* [Constants](#acme.SomeClass.Constants)  
		* [SOME_CONSTANT](#acme.SomeClass.SOME_CONSTANT) : [`integer`](../API/builtins/integer.md)  
		* [StatusCode](#acme.SomeClass.StatusCode)  
	* [Properties](#acme.SomeClass.Properties)  
		* [some_field](#acme.SomeClass.some_field) : [`boolean`](../API/builtins/boolean.md)  
		* [__index](#acme.SomeClass.__index) : [`function`](../API/builtins/function.md) | [`acme.SomeClass`](../API/some_class.md#acme.SomeClass)  
	* [Functions](#acme.SomeClass.Functions)  
		* [some_function](#acme.SomeClass.some_function) ([*self*](../API/builtins/self.md))  
		* [function_with_enum_return](#acme.SomeClass.function_with_enum_return) ([*self*](../API/builtins/self.md)) `->` [`acme.SomeClass.StatusCode`](../API/some_class.md#acme.SomeClass.StatusCode)  
* [SomeClassInstance](#SomeClassInstance)  
	* [Properties](#SomeClassInstance.Properties)  
		* [some_property](#SomeClassInstance.some_property) : [`table`](../API/builtins/table.md)`<`[`string`](../API/builtins/string.md), [`integer`](../API/builtins/integer.md)`>`  
	* [Functions](#SomeClassInstance.Functions)  
		* [some_function](#SomeClassInstance.some_function) ([*self*](../API/builtins/self.md))  
# acme.SomeClass <span style="visibility: hidden">SomeClass</span> { #acme.SomeClass }
---
## Constants { #acme.SomeClass.Constants }
### StatusCode { #acme.SomeClass.StatusCode }
> ```lua
> {
>     OK: integer = 0,
>     ERROR: integer = 1,
> }
> ```
### SOME_CONSTANT : [`integer`](../API/builtins/integer.md) { #acme.SomeClass.SOME_CONSTANT }
> SOME_CONSTANT docs

---
## Properties { #acme.SomeClass.Properties }
### some_field : [`boolean`](../API/builtins/boolean.md) { #acme.SomeClass.some_field }
### __index : [`function`](../API/builtins/function.md) | [`acme.SomeClass`](../API/some_class.md#acme.SomeClass) { #acme.SomeClass.__index }
---
## Functions { #acme.SomeClass.Functions }
### some_function([*self*](../API/builtins/self.md)) { #acme.SomeClass.some_function }
> This function does something.
### function_with_enum_return([*self*](../API/builtins/self.md)) { #acme.SomeClass.function_with_enum_return }
`->`[`acme.SomeClass.StatusCode`](../API/some_class.md#acme.SomeClass.StatusCode)  

> This function also does something and returns a status code enum.  
# SomeClassInstance { #SomeClassInstance }
> SomeClass docs
---
## Properties { #SomeClassInstance.Properties }
### some_property : [`table`](../API/builtins/table.md)`<`[`string`](../API/builtins/string.md), [`integer`](../API/builtins/integer.md)`>` { #SomeClassInstance.some_property }
---
## Functions { #SomeClassInstance.Functions }
### some_function([*self*](../API/builtins/self.md)) { #SomeClassInstance.some_function }
> SomeFunction docs