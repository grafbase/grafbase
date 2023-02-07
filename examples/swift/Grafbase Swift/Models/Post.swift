//
//  Post.swift
//  Grafbase Swift
//

import Foundation

struct PostCollection: Decodable {
    let postCollection: Edge<Post>
}

struct Post: Decodable, Identifiable, Hashable {
    var id: String = UUID().uuidString
    let title: String
    var body: String = ""
    var comments: Edge<Comment>?
    
    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }
    
    static func ==(lhs: Post, rhs: Post) -> Bool {
        return lhs.id == rhs.id
    }
}
