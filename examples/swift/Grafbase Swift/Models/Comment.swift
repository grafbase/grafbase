//
//  Comment.swift
//  Grafbase Swift
//

import Foundation

struct Comment: Decodable, Identifiable, Hashable {
    var id: String = UUID().uuidString
    let message: String
}
