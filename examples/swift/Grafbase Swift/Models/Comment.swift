//
//  Comment.swift
//  Grafbase Swift
//
//  Created by Craig Tweedy on 05/12/2022.
//

import Foundation

struct Comment: Decodable, Identifiable, Hashable {
    var id: String = UUID().uuidString
    let message: String
}
