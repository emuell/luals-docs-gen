# acme { #acme }
> Holds all acme related API test functions and classes.
* [Constants](#constants)
	* [API_VERSION](#API_VERSION) : [`number`](../API/builtins/number.md)
* [Functions](#functions)
	* [SomeClass](#SomeClass) () `->` [`SomeClassInstance`](#someclassinstance)
	* global_function()
	* [global_function2](#global_function2) () `->` [`GlobalTestClass1`](#globaltestclass1) | [`GlobalTestClass2`](#globaltestclass2)
* [Structs](#structs)
	* [GlobalTestClass1](#GlobalTestClass1)
		* [Properties](#properties)
			* [field1](#field1) : [`number`](../API/builtins/number.md)
			* [field2](#field2) : [`string`](../API/builtins/string.md)
	* [GlobalTestClass2](#GlobalTestClass2)
		* [Properties](#properties)
			* [alias](#alias) : [`SomeAlias`](#SomeAlias)
			* [field](#field) : [`GlobalTestClass1`](#globaltestclass1)
		* [Aliases](#aliases)
			* [SomeAlias](#SomeAlias)
	* [SomeClassInstance](#SomeClassInstance)
		* [Properties](#properties)
			* [some_property](#some_property) : [`table`](../API/builtins/table.md)`<`[`string`](../API/builtins/string.md), [`integer`](../API/builtins/integer.md)`>`
		* [Functions](#functions)
			* [some_function](#some_function) ([*self*](../API/builtins/self.md))
* [Aliases](#aliases)
	* [SomeAlias](#SomeAlias)
---
## Constants
### API_VERSION : [`number`](../API/builtins/number.md) { #API_VERSION }
> This is a const

---
## Functions
### `SomeClass()` { #SomeClass }
`->`[`SomeClassInstance`](#someclassinstance)  

> SomeClassInstance Docs
### `global_function()` { #global_function }
> Global function docs
### `global_function2()` { #global_function2 }
`->`[`GlobalTestClass1`](#globaltestclass1) | [`GlobalTestClass2`](#globaltestclass2)  

> More global function docs
---
# Structs
---
# GlobalTestClass1 { #GlobalTestClass1 }
---
## Properties
### field1 : [`number`](../API/builtins/number.md) { #field1 }
### field2 : [`string`](../API/builtins/string.md) { #field2 }
---
# GlobalTestClass2 { #GlobalTestClass2 }
---
## Properties
### alias : [`SomeAlias`](#SomeAlias) { #alias }
> This is an alias

### field : [`GlobalTestClass1`](#globaltestclass1) { #field }
---
# Aliases
---
---
### SomeAlias { #SomeAlias }
[`string`](../API/builtins/string.md)  
> This is an alias
---
---
# SomeClassInstance { #SomeClassInstance }
> SomeClass docs
---
## Properties
### some_property : [`table`](../API/builtins/table.md)`<`[`string`](../API/builtins/string.md), [`integer`](../API/builtins/integer.md)`>` { #some_property }
---
## Functions
### some_function([*self*](../API/builtins/self.md)) { #some_function }
> SomeFunction docs
---
---
# Aliases
---
---
### SomeAlias { #SomeAlias }
[`string`](../API/builtins/string.md)  
> This is an alias
---