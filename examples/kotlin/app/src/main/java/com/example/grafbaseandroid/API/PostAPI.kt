package com.example.grafbaseandroid.API

import com.example.grafbaseandroid.Entities.GraphQLResult
import com.example.grafbaseandroid.Entities.Post
import com.example.grafbaseandroid.Entities.PostCollection
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.request.*

class PostAPIImpl(
    private val client: HttpClient
) : PostAPI {
    override suspend fun getPosts(): GraphQLResult<PostCollection> {
        return client.post {
            setBody(GraphQLOperation("""
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
            """.trimIndent()))
        }.body()
    }
}

interface PostAPI {
    suspend fun getPosts(): GraphQLResult<PostCollection>

    companion object {
        fun create(): PostAPI {
            return PostAPIImpl(
                ktorHttpClient
            )
        }
    }
}