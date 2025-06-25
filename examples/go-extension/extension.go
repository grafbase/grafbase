package main

import (
	"fmt"
	"strings"
	"unsafe"

	"example.com/grafbase-go-extension/grafbase/sdk/authentication"
	"example.com/grafbase-go-extension/grafbase/sdk/authorization"
	"example.com/grafbase-go-extension/grafbase/sdk/hooks"
	"example.com/grafbase-go-extension/grafbase/sdk/resolver"
	"example.com/grafbase-go-extension/grafbase/sdk/sdk"
	"example.com/grafbase-go-extension/grafbase/sdk/token"
	"example.com/grafbase-go-extension/internal/stubs"
	"go.bytecodealliance.org/cm"

	"github.com/fxamacker/cbor/v2"
)

// Config defines the expected structure of the extension's configuration
type Config struct {
	Secret string `cbor:"secret"`
}

// Global for the extension state
var extension struct {
	config Config
}

func main() {
	// main is never called
}

func init() {
	// Register our exports with the runtime
	sdk.Exports.RegisterExtension = registerExtension
	sdk.Exports.Init = InitExtension
	authentication.Exports.Authenticate = authenticate
	authentication.Exports.PublicMetadata = stubs.PublicMetadata
	authorization.Exports.AuthorizeQuery = stubs.AuthorizeQuery
	authorization.Exports.AuthorizeResponse = stubs.AuthorizeResponse
	hooks.Exports.OnRequest = stubs.OnRequestHook
	hooks.Exports.OnResponse = stubs.OnResponseHook
	resolver.Exports.Prepare = stubs.PrepareResolver
	resolver.Exports.CreateSubscription = stubs.CreateSubscription
	resolver.Exports.DropSubscription = stubs.DropSubscription
	resolver.Exports.Resolve = stubs.Resolve
	resolver.Exports.ResolveNextSubscriptionItem = stubs.ResolveNextSubscriptionItem
}

// registerExtension is called to register the extension with Grafbase
func registerExtension() {
	fmt.Println("Registering Grafbase go extension example")
}

// InitExtension is called to initialize the extension with schema directives and configuration
// This function receives:
// - schemas: A list of (name, schema) tuples where each schema is associated with a name
// - configuration: Configuration data for the extension (as bytes that can be parsed)
// - canSkipSendingEvents: A flag indicating if events can be skipped
func InitExtension(
	schemas cm.List[cm.Tuple[string, sdk.Schema]],
	configuration cm.List[uint8],
	canSkipSendingEvents bool,
) (result cm.Result[string, struct{}, string]) {
	fmt.Println("Initializing GOLANG Grafbase extension")

	// Parse the configuration if provided
	if int(configuration.Len()) > 0 {
		configBytes := unsafe.Slice(configuration.Data(), configuration.Len())

		// Deserialize from CBOR into the typed Config struct
		var config Config
		if err := cbor.Unmarshal(configBytes, &config); err != nil {
			fmt.Printf("Configuration provided but not valid CBOR: %s\n", err)
			fmt.Printf("Raw configuration bytes: %v\n", configBytes)
			panic("Invalid configuration")
		} else {
			fmt.Printf("Deserialized CBOR configuration\n")
			extension.config = config
		}
	} else {
		fmt.Println("No configuration provided")
	}

	return cm.OK[cm.Result[string, struct{}, string]](struct{}{})
}

// The runtime logic for our extension.
func authenticate(context authentication.SharedContext, headers authentication.Headers) (result cm.Result[authentication.ErrorResponseShape, authentication.Token, authentication.ErrorResponse]) {
	fmt.Println("Authenticating from the go extension")

	// Extract the token from the Authorization header
	authorizationHeaderValues := headers.Get("Authorization").Slice()

	if len(authorizationHeaderValues) == 0 {
		return cm.OK[cm.Result[authentication.ErrorResponseShape, authentication.Token, authentication.ErrorResponse]](token.TokenAnonymous())
	}

	authorizationHeader := authorizationHeaderValues[len(authorizationHeaderValues)-1]

	authorizationHeaderString := unsafe.String(authorizationHeader.Data(), authorizationHeader.Len())

	tokenFromHeader, _ := strings.CutPrefix(authorizationHeaderString, "Bearer ")

	// Validate whether the token is the same as the configured secret
	if extension.config.Secret == tokenFromHeader {
		fmt.Println("Successful authentication")

		contents := []byte(tokenFromHeader)

		return cm.OK[cm.Result[authentication.ErrorResponseShape, authentication.Token, authentication.ErrorResponse]](token.TokenBytes(cm.NewList[uint8](&contents[0], len(tokenFromHeader))))
	} else {
		fmt.Println("Wrong authentication secret. Refusing access.")

		// Wrong secret. Authenticate as anonymous.
		return cm.Err[cm.Result[authentication.ErrorResponseShape, authentication.Token, authentication.ErrorResponse]](authentication.ErrorResponse{StatusCode: 401})
	}
}
