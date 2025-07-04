// Code generated by wit-bindgen-go. DO NOT EDIT.

// Package headers represents the imported interface "grafbase:sdk/headers".
package headers

import (
	"go.bytecodealliance.org/cm"
)

// Headers represents the imported resource "grafbase:sdk/headers#headers".
//
// A resource for accessing HTTP headers.
//
//	resource headers
type Headers cm.Resource

// ResourceDrop represents the imported resource-drop for resource "headers".
//
// Drops a resource handle.
//
//go:nosplit
func (self Headers) ResourceDrop() {
	self0 := cm.Reinterpret[uint32](self)
	wasmimport_HeadersResourceDrop((uint32)(self0))
	return
}

// HeadersNew represents the imported static function "new".
//
// Create new headers
//
//	new: static func() -> headers
//
//go:nosplit
func HeadersNew() (result Headers) {
	result0 := wasmimport_HeadersNew()
	result = cm.Reinterpret[Headers]((uint32)(result0))
	return
}

// Append represents the imported method "append".
//
// Append a value for a name. Does not change or delete any existing
// values for that name.
//
// Fails with `header-error.immutable` if the `fields` are immutable.
//
//	append: func(name: string, value: list<u8>) -> result<_, header-error>
//
//go:nosplit
func (self Headers) Append(name string, value cm.List[uint8]) (result cm.Result[HeaderError, struct{}, HeaderError]) {
	self0 := cm.Reinterpret[uint32](self)
	name0, name1 := cm.LowerString(name)
	value0, value1 := cm.LowerList(value)
	wasmimport_HeadersAppend((uint32)(self0), (*uint8)(name0), (uint32)(name1), (*uint8)(value0), (uint32)(value1), &result)
	return
}

// Delete represents the imported method "delete".
//
// Delete all values for a name. Does nothing if no values for the name
// exist.
//
// Fails with `header-error.immutable` if the `fields` are immutable.
//
//	delete: func(name: string) -> result<_, header-error>
//
//go:nosplit
func (self Headers) Delete(name string) (result cm.Result[HeaderError, struct{}, HeaderError]) {
	self0 := cm.Reinterpret[uint32](self)
	name0, name1 := cm.LowerString(name)
	wasmimport_HeadersDelete((uint32)(self0), (*uint8)(name0), (uint32)(name1), &result)
	return
}

// Entries represents the imported method "entries".
//
// Retrieve the full set of names and values in the Fields. Like the
// constructor, the list represents each name-value pair.
//
// The outer list represents each name-value pair in the Fields. Names
// which have multiple values are represented by multiple entries in this
// list with the same name.
//
// The names and values are always returned in the original casing and in
// the order in which they will be serialized for transport.
//
//	entries: func() -> list<tuple<string, list<u8>>>
//
//go:nosplit
func (self Headers) Entries() (result cm.List[cm.Tuple[string, cm.List[uint8]]]) {
	self0 := cm.Reinterpret[uint32](self)
	wasmimport_HeadersEntries((uint32)(self0), &result)
	return
}

// Get represents the imported method "get".
//
// Get all of the values corresponding to a name. If the name is not present
// in this `fields`, an empty list is returned. However, if the name is
// present but empty, this is represented by a list with one or more
// empty values present.
//
//	get: func(name: string) -> list<list<u8>>
//
//go:nosplit
func (self Headers) Get(name string) (result cm.List[cm.List[uint8]]) {
	self0 := cm.Reinterpret[uint32](self)
	name0, name1 := cm.LowerString(name)
	wasmimport_HeadersGet((uint32)(self0), (*uint8)(name0), (uint32)(name1), &result)
	return
}

// GetAndDelete represents the imported method "get-and-delete".
//
// Delete all values for a name. Does nothing if no values for the name
// exist.
//
// Returns all values previously corresponding to the name, if any.
//
// Fails with `header-error.immutable` if the `fields` are immutable.
//
//	get-and-delete: func(name: string) -> result<list<list<u8>>, header-error>
//
//go:nosplit
func (self Headers) GetAndDelete(name string) (result cm.Result[cm.List[cm.List[uint8]], cm.List[cm.List[uint8]], HeaderError]) {
	self0 := cm.Reinterpret[uint32](self)
	name0, name1 := cm.LowerString(name)
	wasmimport_HeadersGetAndDelete((uint32)(self0), (*uint8)(name0), (uint32)(name1), &result)
	return
}

// Has represents the imported method "has".
//
// Returns `true` when the name is present in this `fields`. If the name is
// syntactically invalid, `false` is returned.
//
//	has: func(name: string) -> bool
//
//go:nosplit
func (self Headers) Has(name string) (result bool) {
	self0 := cm.Reinterpret[uint32](self)
	name0, name1 := cm.LowerString(name)
	result0 := wasmimport_HeadersHas((uint32)(self0), (*uint8)(name0), (uint32)(name1))
	result = (bool)(cm.U32ToBool((uint32)(result0)))
	return
}

// Set represents the imported method "set".
//
// Set all of the values for a name. Clears any existing values for that
// name, if they have been set.
//
// Fails with `header-error.immutable` if the `fields` are immutable.
//
//	set: func(name: string, value: list<list<u8>>) -> result<_, header-error>
//
//go:nosplit
func (self Headers) Set(name string, value cm.List[cm.List[uint8]]) (result cm.Result[HeaderError, struct{}, HeaderError]) {
	self0 := cm.Reinterpret[uint32](self)
	name0, name1 := cm.LowerString(name)
	value0, value1 := cm.LowerList(value)
	wasmimport_HeadersSet((uint32)(self0), (*uint8)(name0), (uint32)(name1), (*cm.List[uint8])(value0), (uint32)(value1), &result)
	return
}

// HeaderError represents the variant "grafbase:sdk/headers#header-error".
//
// setting or appending to a `fields` resource.
//
//	variant header-error {
//		invalid-syntax,
//		forbidden,
//		immutable,
//	}
type HeaderError uint8

const (
	// This error indicates that a `field-name` or `field-value` was
	// syntactically invalid when used with an operation that sets headers in a
	// `fields`.
	HeaderErrorInvalidSyntax HeaderError = iota

	// This error indicates that a forbidden `field-name` was used when trying
	// to set a header in a `fields`.
	HeaderErrorForbidden

	// This error indicates that the operation on the `fields` was not
	// permitted because the fields are immutable.
	HeaderErrorImmutable
)

var _HeaderErrorStrings = [3]string{
	"invalid-syntax",
	"forbidden",
	"immutable",
}

// String implements [fmt.Stringer], returning the enum case name of e.
func (e HeaderError) String() string {
	return _HeaderErrorStrings[e]
}

// MarshalText implements [encoding.TextMarshaler].
func (e HeaderError) MarshalText() ([]byte, error) {
	return []byte(e.String()), nil
}

// UnmarshalText implements [encoding.TextUnmarshaler], unmarshaling into an enum
// case. Returns an error if the supplied text is not one of the enum cases.
func (e *HeaderError) UnmarshalText(text []byte) error {
	return _HeaderErrorUnmarshalCase(e, text)
}

var _HeaderErrorUnmarshalCase = cm.CaseUnmarshaler[HeaderError](_HeaderErrorStrings[:])
