// Code generated by wit-bindgen-go. DO NOT EDIT.

package cache

import (
	"go.bytecodealliance.org/cm"
)

func lower_OptionU64(v cm.Option[uint64]) (f0 uint32, f1 uint64) {
	some := v.Some()
	if some != nil {
		f0 = 1
		v1 := (uint64)(*some)
		f1 = (uint64)(v1)
	}
	return
}
