// Code generated by wit-bindgen-go. DO NOT EDIT.

// Package resolvertypes represents the imported interface "grafbase:sdk/resolver-types".
//
// Types used by the resolver functions
package resolvertypes

import (
	sdkerror "example.com/grafbase-go-extension/grafbase/sdk/error"
	"example.com/grafbase-go-extension/grafbase/sdk/schema"
	"go.bytecodealliance.org/cm"
)

// DefinitionID represents the type alias "grafbase:sdk/resolver-types#definition-id".
//
// See [schema.DefinitionID] for more information.
type DefinitionID = schema.DefinitionID

// Error represents the type alias "grafbase:sdk/resolver-types#error".
//
// See [sdkerror.Error] for more information.
type Error = sdkerror.Error

// Data represents the variant "grafbase:sdk/resolver-types#data".
//
// Any raw data that the engine can read.
//
//	variant data {
//		json(list<u8>),
//		cbor(list<u8>),
//	}
type Data cm.Variant[uint8, cm.List[uint8], cm.List[uint8]]

// DataJSON returns a [Data] of case "json".
func DataJSON(data cm.List[uint8]) Data {
	return cm.New[Data](0, data)
}

// JSON returns a non-nil *[cm.List[uint8]] if [Data] represents the variant case "json".
func (self *Data) JSON() *cm.List[uint8] {
	return cm.Case[cm.List[uint8]](self, 0)
}

// DataCbor returns a [Data] of case "cbor".
func DataCbor(data cm.List[uint8]) Data {
	return cm.New[Data](1, data)
}

// Cbor returns a non-nil *[cm.List[uint8]] if [Data] represents the variant case "cbor".
func (self *Data) Cbor() *cm.List[uint8] {
	return cm.Case[cm.List[uint8]](self, 1)
}

var _DataStrings = [2]string{
	"json",
	"cbor",
}

// String implements [fmt.Stringer], returning the variant case name of v.
func (v Data) String() string {
	return _DataStrings[v.Tag()]
}

// FieldID represents the u16 "grafbase:sdk/resolver-types#field-id".
//
// index within the list of fields provided to the prepare() function
//
//	type field-id = u16
type FieldID uint16

// FieldIDRange represents the tuple "grafbase:sdk/resolver-types#field-id-range".
//
// range within the list of fields provided to the prepare() function
//
//	type field-id-range = tuple<field-id, field-id>
type FieldIDRange [2]FieldID

// ArgumentsID represents the u16 "grafbase:sdk/resolver-types#arguments-id".
//
// In the prepare() function we don't have yet access to the arguments as they depend
// on the variables. So instead we provide an arguments id. The gateway will be provide
// the
// serialized arguments for every arguments-id.
//
//	type arguments-id = u16
type ArgumentsID uint16

// SelectionSet represents the record "grafbase:sdk/resolver-types#selection-set".
//
// Query selection set
//
//	record selection-set {
//		requires-typename: bool,
//		fields-ordered-by-parent-entity: field-id-range,
//	}
type SelectionSet struct {
	_                           cm.HostLayout `json:"-"`
	RequiresTypename            bool          `json:"requires-typename"`
	FieldsOrderedByParentEntity FieldIDRange  `json:"fields-ordered-by-parent-entity"`
}

// Field represents the record "grafbase:sdk/resolver-types#field".
//
// Query field
//
//	record field {
//		alias: option<string>,
//		definition-id: definition-id,
//		arguments: option<arguments-id>,
//		selection-set: option<selection-set>,
//	}
type Field struct {
	_     cm.HostLayout     `json:"-"`
	Alias cm.Option[string] `json:"alias"`

	// Definition id which can be used to retrieve additional data from the subgraph schema
	// provided to the init() function.
	DefinitionID DefinitionID            `json:"definition-id"`
	Arguments    cm.Option[ArgumentsID]  `json:"arguments"`
	SelectionSet cm.Option[SelectionSet] `json:"selection-set"`
}

// Response represents the record "grafbase:sdk/resolver-types#response".
//
// Resolver response
//
//	record response {
//		data: option<data>,
//		errors: list<error>,
//	}
type Response struct {
	_      cm.HostLayout   `json:"-"`
	Data   cm.Option[Data] `json:"data"`
	Errors cm.List[Error]  `json:"errors"`
}

// SubscriptionItem represents the variant "grafbase:sdk/resolver-types#subscription-item".
//
// Subscription item. In case of multiple responses, they're treated as if we received
// multiple items in the subscription.
//
//	variant subscription-item {
//		single(response),
//		multiple(list<response>),
//	}
type SubscriptionItem cm.Variant[uint8, ResponseShape, Response]

// SubscriptionItemSingle returns a [SubscriptionItem] of case "single".
func SubscriptionItemSingle(data Response) SubscriptionItem {
	return cm.New[SubscriptionItem](0, data)
}

// Single returns a non-nil *[Response] if [SubscriptionItem] represents the variant case "single".
func (self *SubscriptionItem) Single() *Response {
	return cm.Case[Response](self, 0)
}

// SubscriptionItemMultiple returns a [SubscriptionItem] of case "multiple".
func SubscriptionItemMultiple(data cm.List[Response]) SubscriptionItem {
	return cm.New[SubscriptionItem](1, data)
}

// Multiple returns a non-nil *[cm.List[Response]] if [SubscriptionItem] represents the variant case "multiple".
func (self *SubscriptionItem) Multiple() *cm.List[Response] {
	return cm.Case[cm.List[Response]](self, 1)
}

var _SubscriptionItemStrings = [2]string{
	"single",
	"multiple",
}

// String implements [fmt.Stringer], returning the variant case name of v.
func (v SubscriptionItem) String() string {
	return _SubscriptionItemStrings[v.Tag()]
}
