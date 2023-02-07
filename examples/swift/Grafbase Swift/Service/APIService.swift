//
//  APIService.swift
//  Grafbase Swift
//

import Foundation

class APIService {
    let api: GraphQLAPI = GraphQLAPI()
    
    func listPosts() async -> PostCollection? {
        return (
            try? await self.api.performOperation(GraphQLOperation.LIST_POSTS)
        )
    }
}
