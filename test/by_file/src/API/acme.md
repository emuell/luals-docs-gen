# acme  
* [acme](#acme)  
	* [Constants](#acme.Constants)  
		* [API_VERSION](#acme.API_VERSION) : [`number`](../API/builtins/number.md)  
	* [Functions](#acme.Functions)  
		* [SomeClass](#acme.SomeClass) () `->` [`SomeClassInstance`](../API/some_class.md#SomeClassInstance)  
		* [global_function](#acme.global_function) ()  
		* [global_function2](#acme.global_function2) () `->` [`GlobalTestClass1`](../API/acme.md#GlobalTestClass1) | [`GlobalTestClass2`](../API/acme.md#GlobalTestClass2)  
* [GlobalTestClass1](#GlobalTestClass1)  
	* [Properties](#GlobalTestClass1.Properties)  
		* [field1](#GlobalTestClass1.field1) : [`number`](../API/builtins/number.md)  
		* [field2](#GlobalTestClass1.field2) : [`string`](../API/builtins/string.md)  
* [GlobalTestClass2](#GlobalTestClass2)  
	* [Properties](#GlobalTestClass2.Properties)  
		* [alias](#GlobalTestClass2.alias) : [`SomeAlias`](#SomeAlias)  
		* [field](#GlobalTestClass2.field) : [`GlobalTestClass1`](../API/acme.md#GlobalTestClass1)  
	* [Aliases](#GlobalTestClass2.Aliases)  
		* [SomeAlias](#GlobalTestClass2.SomeAlias)  
# acme { #acme }
> Holds all acme related API test functions and classes.
---
## Constants { #acme.Constants }
### API_VERSION : [`number`](../API/builtins/number.md) { #acme.API_VERSION }
> This is a const

---
## Functions { #acme.Functions }
### `SomeClass()` { #acme.SomeClass }
`->`[`SomeClassInstance`](../API/some_class.md#SomeClassInstance)  

> SomeClassInstance Docs
### `global_function()` { #acme.global_function }
> Global function docs
### `global_function2()` { #acme.global_function2 }
`->`[`GlobalTestClass1`](../API/acme.md#GlobalTestClass1) | [`GlobalTestClass2`](../API/acme.md#GlobalTestClass2)  

> More global function docs  
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

### field : [`GlobalTestClass1`](../API/acme.md#GlobalTestClass1) { #GlobalTestClass2.field }
---
# Aliases { #GlobalTestClass2.Aliases }
---
### SomeAlias { #GlobalTestClass2.SomeAlias }
[`string`](../API/builtins/string.md)  
> This is an alias
---