// Code generated by wit-bindgen-go. DO NOT EDIT.

package authentication

import (
	"go.bytecodealliance.org/cm"
	"unsafe"
)

// ErrorResponseShape is used for storage in variant or result types.
type ErrorResponseShape struct {
	_     cm.HostLayout
	shape [unsafe.Sizeof(ErrorResponse{})]byte
}

// ErrorShape is used for storage in variant or result types.
type ErrorShape struct {
	_     cm.HostLayout
	shape [unsafe.Sizeof(Error{})]byte
}
