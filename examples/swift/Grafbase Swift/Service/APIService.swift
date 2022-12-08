//
//  APIService.swift
//  Grafbase Swift
//
//  Created by Craig Tweedy on 05/12/2022.
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
