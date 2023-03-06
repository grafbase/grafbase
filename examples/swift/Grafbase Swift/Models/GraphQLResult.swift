//
//  GraphQLResult.swift
//  Grafbase Swift
//

import Foundation

struct Edge<T: Decodable>: Decodable {
    let edges: [Node<T>];
}

struct Node<T: Decodable>: Decodable{
    let node: T;
}

struct GraphQLResult<T: Decodable>: Decodable {
    var object: T?
    var errorMessages: [String] = []
    
    enum CodingKeys: String, CodingKey {
        case data
        case errors
    }
    
    struct Error: Decodable {
        let message: String
    }
    
    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        do {
            self.object = try container.decodeIfPresent(T.self, forKey: .data)
        } catch {
            print(error)
        }
        
        var errorMessages: [String] = []

        let errors = try container.decodeIfPresent([Error].self, forKey: .errors)
        if let errors = errors {
            errorMessages.append(contentsOf: errors.map { $0.message })
        }

        self.errorMessages = errorMessages
    }
}
