package graph

import (
	"math/rand"
	"sync"
	"subgraph/graph/model"
)

// This file will not be regenerated automatically.
//
// It serves as dependency injection for your app, add any dependencies you require here.

type Resolver struct {
	// All messages since launching the GraphQL endpoint
	ChatMessages  []*model.Message
	// All active subscriptions
	ChatObservers map[string]*Observer
	mu            sync.Mutex
}

type Observer struct {
    user string
    channel chan *model.Message
}

var letterRunes = []rune("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")

func randString(n int) string {
	b := make([]rune, n)
	for i := range b {
		b[i] = letterRunes[rand.Intn(len(letterRunes))]
	}
	return string(b)
}
