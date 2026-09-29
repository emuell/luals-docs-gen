# acme { #acme }
> Holds all acme related API test functions and classes.
* [Constants](#Constants)
	* [API_VERSION](#API_VERSION) : [`number`](../API/builtins/number.md)
* [Functions](#Functions)
	* [SomeClass](#SomeClass) () `->` [`SomeClassInstance`](#SomeClassInstance)
	* [global_function](#global_function) ()
	* [global_function2](#global_function2) () `->` [`GlobalTestClass1`](#GlobalTestClass1) | [`GlobalTestClass2`](#GlobalTestClass2)
* [Structs](#Structs)
	* [GlobalTestClass1](#GlobalTestClass1)
		* [Properties](#GlobalTestClass1.Properties)
			* [field1](#GlobalTestClass1.field1) : [`number`](../API/builtins/number.md)
			* [field2](#GlobalTestClass1.field2) : [`string`](../API/builtins/string.md)
	* [GlobalTestClass2](#GlobalTestClass2)
		* [Properties](#GlobalTestClass2.Properties)
			* [alias](#GlobalTestClass2.alias) : [`SomeAlias`](#SomeAlias)
			* [field](#GlobalTestClass2.field) : [`GlobalTestClass1`](#GlobalTestClass1)
		* [Aliases](#GlobalTestClass2.Aliases)
			* [SomeAlias](#GlobalTestClass2.SomeAlias)
	* [SomeClassInstance](#SomeClassInstance)
		* [Properties](#SomeClassInstance.Properties)
			* [some_property](#SomeClassInstance.some_property) : [`table`](../API/builtins/table.md)`<`[`string`](../API/builtins/string.md), [`integer`](../API/builtins/integer.md)`>`
		* [Functions](#SomeClassInstance.Functions)
			* [some_function](#SomeClassInstance.some_function) ([*self*](../API/builtins/self.md))
* [Aliases](#Aliases)
	* [SomeAlias](#SomeAlias)
---
## Constants { #Constants }
### API_VERSION : [`number`](../API/builtins/number.md) { #API_VERSION }
> This is a const

---
## Functions { #Functions }
### `SomeClass()` { #SomeClass }
`->`[`SomeClassInstance`](#SomeClassInstance)  

> SomeClassInstance Docs
### `global_function()` { #global_function }
> Global function docs
### `global_function2()` { #global_function2 }
`->`[`GlobalTestClass1`](#GlobalTestClass1) | [`GlobalTestClass2`](#GlobalTestClass2)  

> More global function docs
---
# Structs { #Structs }
# GlobalTestClass1 { #GlobalTestClass1 }
---
## Properties { #GlobalTestClass1.Properties }
### field1 : [`number`](../API/builtins/number.md) { #GlobalTestClass1.field1 }
### field2 : [`string`](../API/builtins/string.md) { #GlobalTestClass1.field2 }
# GlobalTestClass2 { #GlobalTestClass2 }
---
## Properties { #GlobalTestClass2.Properties }
### alias : [`SomeAlias`](#SomeAlias) { #GlobalTestClass2.alias }
> This is an alias

### field : [`GlobalTestClass1`](#GlobalTestClass1) { #GlobalTestClass2.field }
---
# Aliases { #GlobalTestClass2.Aliases }
---
### SomeAlias { #GlobalTestClass2.SomeAlias }
[`string`](../API/builtins/string.md)  
> This is an alias
---
# SomeClassInstance { #SomeClassInstance }
> SomeClass docs
---
## Properties { #SomeClassInstance.Properties }
### some_property : [`table`](../API/builtins/table.md)`<`[`string`](../API/builtins/string.md), [`integer`](../API/builtins/integer.md)`>` { #SomeClassInstance.some_property }
---
## Functions { #SomeClassInstance.Functions }
### some_function([*self*](../API/builtins/self.md)) { #SomeClassInstance.some_function }
> SomeFunction docs
---
# Aliases { #Aliases }
---
### SomeAlias { #SomeAlias }
[`string`](../API/builtins/string.md)  
> This is an alias
---