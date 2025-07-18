// Code generated by wit-bindgen-go. DO NOT EDIT.

package resolver

import (
	"go.bytecodealliance.org/cm"
)

// Exports represents the caller-defined exports from "grafbase:sdk/resolver".
var Exports struct {
	// Prepare represents the caller-defined, exported function "prepare".
	//
	//	prepare: func(context: shared-context, subgraph-name: string, directive: directive,
	//	root-field-id: field-id, fields: list<field>) -> result<list<u8>, error>
	Prepare func(context SharedContext, subgraphName string, directive Directive, rootFieldID FieldID, fields cm.List[Field]) (result cm.Result[ErrorShape, cm.List[uint8], Error])

	// Resolve represents the caller-defined, exported function "resolve".
	//
	//	resolve: func(context: shared-context, prepared: list<u8>, headers: headers, arguments:
	//	list<tuple<arguments-id, list<u8>>>) -> response
	Resolve func(context SharedContext, prepared cm.List[uint8], headers_ Headers, arguments cm.List[cm.Tuple[ArgumentsID, cm.List[uint8]]]) (result Response)

	// CreateSubscription represents the caller-defined, exported function "create-subscription".
	//
	//	create-subscription: func(context: shared-context, prepared: list<u8>, headers:
	//	headers, arguments: list<tuple<arguments-id, list<u8>>>) -> result<option<list<u8>>,
	//	error>
	CreateSubscription func(context SharedContext, prepared cm.List[uint8], headers_ Headers, arguments cm.List[cm.Tuple[ArgumentsID, cm.List[uint8]]]) (result cm.Result[ErrorShape, cm.Option[cm.List[uint8]], Error])

	// ResolveNextSubscriptionItem represents the caller-defined, exported function "resolve-next-subscription-item".
	//
	// resolves the next item in a subscription stream. Must be called after resolve-subscription
	// If data is null, it means the subscription is done and no more items will be requested.
	//
	//	resolve-next-subscription-item: func(context: shared-context) -> result<option<subscription-item>,
	//	error>
	ResolveNextSubscriptionItem func(context SharedContext) (result cm.Result[OptionSubscriptionItemShape, cm.Option[SubscriptionItem], Error])

	// DropSubscription represents the caller-defined, exported function "drop-subscription".
	//
	// Called if the key provided by resolve-subscription is enough and any stored state
	// can be dropped.
	// This implies resolve-next-subscription-item will never be called.
	//
	//	drop-subscription: func(context: shared-context)
	DropSubscription func(context SharedContext)
}
