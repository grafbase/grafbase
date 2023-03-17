package com.example.grafbaseandroid.Repositories

import com.example.grafbaseandroid.API.PostAPI
import com.example.grafbaseandroid.Entities.PostCollection
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class PostRepository {
    suspend fun fetchPosts(): Result<PostCollection> {
        return withContext(Dispatchers.IO) {
            try {
                val result = PostAPI.create().getPosts().data
                Result.success(result)
            } catch (exception: Exception) {
                Result.failure(exception)
            } as Result<PostCollection>
        }
    }
}