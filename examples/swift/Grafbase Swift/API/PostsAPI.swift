//
//  PostsAPI.swift
//  Grafbase Swift
//

import Foundation

extension GraphQLOperation {
    static var LIST_POSTS: Self {
        GraphQLOperation(
                    """
                        {
                            postCollection(first:10) {
                                edges {
                                  node {
                                    id
                                    title
                                    body
                                    comments(first: 10) {
                                      edges {
                                        node {
                                          id
                                          message
                                        }
                                      }
                                    }
                                  }
                                }
                              }
                        }
                    """
        )
    }
}
