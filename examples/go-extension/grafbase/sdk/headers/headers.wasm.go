// Code generated by wit-bindgen-go. DO NOT EDIT.

package headers

import (
	"go.bytecodealliance.org/cm"
)

// This file contains wasmimport and wasmexport declarations for "grafbase:sdk".

//go:wasmimport grafbase:sdk/headers [resource-drop]headers
//go:noescape
func wasmimport_HeadersResourceDrop(self0 uint32)

//go:wasmimport grafbase:sdk/headers [static]headers.new
//go:noescape
func wasmimport_HeadersNew() (result0 uint32)

//go:wasmimport grafbase:sdk/headers [method]headers.append
//go:noescape
func wasmimport_HeadersAppend(self0 uint32, name0 *uint8, name1 uint32, value0 *uint8, value1 uint32, result *cm.Result[HeaderError, struct{}, HeaderError])

//go:wasmimport grafbase:sdk/headers [method]headers.delete
//go:noescape
func wasmimport_HeadersDelete(self0 uint32, name0 *uint8, name1 uint32, result *cm.Result[HeaderError, struct{}, HeaderError])

//go:wasmimport grafbase:sdk/headers [method]headers.entries
//go:noescape
func wasmimport_HeadersEntries(self0 uint32, result *cm.List[cm.Tuple[string, cm.List[uint8]]])

//go:wasmimport grafbase:sdk/headers [method]headers.get
//go:noescape
func wasmimport_HeadersGet(self0 uint32, name0 *uint8, name1 uint32, result *cm.List[cm.List[uint8]])

//go:wasmimport grafbase:sdk/headers [method]headers.get-and-delete
//go:noescape
func wasmimport_HeadersGetAndDelete(self0 uint32, name0 *uint8, name1 uint32, result *cm.Result[cm.List[cm.List[uint8]], cm.List[cm.List[uint8]], HeaderError])

//go:wasmimport grafbase:sdk/headers [method]headers.has
//go:noescape
func wasmimport_HeadersHas(self0 uint32, name0 *uint8, name1 uint32) (result0 uint32)

//go:wasmimport grafbase:sdk/headers [method]headers.set
//go:noescape
func wasmimport_HeadersSet(self0 uint32, name0 *uint8, name1 uint32, value0 *cm.List[uint8], value1 uint32, result *cm.Result[HeaderError, struct{}, HeaderError])
