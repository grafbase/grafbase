package stubs

import (
	"example.com/grafbase-go-extension/grafbase/sdk/authentication"
	"example.com/grafbase-go-extension/grafbase/sdk/authorization"
	"example.com/grafbase-go-extension/grafbase/sdk/hooks"
	"example.com/grafbase-go-extension/grafbase/sdk/resolver"
	"go.bytecodealliance.org/cm"
)

func PrepareResolver(context resolver.SharedContext, subgraphName string, directive resolver.Directive, rootFieldID resolver.FieldID, fields cm.List[resolver.Field]) (result cm.Result[resolver.ErrorShape, cm.List[uint8], resolver.Error]) {
	panic("prepare")
}

func Resolve(context resolver.SharedContext, prepared cm.List[uint8], headers_ resolver.Headers, arguments cm.List[cm.Tuple[resolver.ArgumentsID, cm.List[uint8]]]) (result resolver.Response) {
	panic("resolve func")
}

func CreateSubscription(context resolver.SharedContext, prepared cm.List[uint8], headers_ resolver.Headers, arguments cm.List[cm.Tuple[resolver.ArgumentsID, cm.List[uint8]]]) (result cm.Result[resolver.ErrorShape, cm.Option[cm.List[uint8]], resolver.Error]) {
	panic("createSubscription func")
}

func ResolveNextSubscriptionItem(context resolver.SharedContext) (result cm.Result[resolver.OptionSubscriptionItemShape, cm.Option[resolver.SubscriptionItem], resolver.Error]) {
	panic("resolveNextSubscriptionItem func")
}

func DropSubscription(context resolver.SharedContext) {
	panic("dropSubscription func")
}

func OnRequestHook(context hooks.SharedContext, url string, method hooks.HTTPMethod, headers_ hooks.Headers) (result cm.Result[hooks.ErrorResponse, struct{}, hooks.ErrorResponse]) {
	panic("on request hook")
}

func OnResponseHook(context hooks.SharedContext, status uint16, headers_ hooks.Headers, eventQueue hooks.EventQueue) (result cm.Result[string, struct{}, string]) {
	panic("on response hook")
}

func AuthorizeQuery(context authorization.SharedContext, headers_ authorization.Headers, token_ authorization.Token, elements authorization.QueryElements) (result cm.Result[authorization.TupleAuthorizationDecisionsListU8Shape, cm.Tuple[authorization.AuthorizationDecisions, cm.List[uint8]], authorization.ErrorResponse]) {
	panic("authorize query")
}

func AuthorizeResponse(context authorization.SharedContext, state cm.List[uint8], elements authorization.ResponseElements) (result cm.Result[authorization.AuthorizationDecisionsShape, authorization.AuthorizationDecisions, authorization.Error]) {
	panic("authorize response")
}

func PublicMetadata() (result cm.Result[authentication.ErrorShape, cm.List[authentication.PublicMetadataEndpoint], authentication.Error]) {
	panic("public metadata")
}
