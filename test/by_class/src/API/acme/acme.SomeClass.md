# acme.SomeClass <span style="visibility: hidden">SomeClass</span> { #acme.SomeClass }
* [Constants](#Constants)
	* [SOME_CONSTANT](#SOME_CONSTANT) : [`integer`](../../API/builtins/integer.md)
	* [StatusCode](#StatusCode)
* [Properties](#Properties)
	* [some_field](#some_field) : [`boolean`](../../API/builtins/boolean.md)
	* [__index](#__index) : [`function`](../../API/builtins/function.md) | [`acme.SomeClass`](../../API/acme/acme.SomeClass.md)
* [Functions](#Functions)
	* [some_function](#some_function) ([*self*](../../API/builtins/self.md))
	* [function_with_enum_return](#function_with_enum_return) ([*self*](../../API/builtins/self.md)) `->` [`acme.SomeClass.StatusCode`](acme.SomeClass.md#StatusCode)
---
## Constants { #Constants }
### StatusCode { #StatusCode }
> ```lua
> {
>     OK: integer = 0,
>     ERROR: integer = 1,
> }
> ```
### SOME_CONSTANT : [`integer`](../../API/builtins/integer.md) { #SOME_CONSTANT }
> SOME_CONSTANT docs

---
## Properties { #Properties }
### some_field : [`boolean`](../../API/builtins/boolean.md) { #some_field }
### __index : [`function`](../../API/builtins/function.md) | [`acme.SomeClass`](../../API/acme/acme.SomeClass.md) { #__index }
---
## Functions { #Functions }
### some_function([*self*](../../API/builtins/self.md)) { #some_function }
> This function does something.
### function_with_enum_return([*self*](../../API/builtins/self.md)) { #function_with_enum_return }
`->`[`acme.SomeClass.StatusCode`](acme.SomeClass.md#StatusCode)  

> This function also does something and returns a status code enum.