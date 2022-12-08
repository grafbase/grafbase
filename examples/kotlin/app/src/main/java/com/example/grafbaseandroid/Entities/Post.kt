package com.example.grafbaseandroid.Entities

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
@Parcelize
data class PostCollection (
    val postCollection: Edge<Post>,
): Parcelable

@Serializable
@Parcelize
data class Post(
    @SerialName("id")
    val id: String,
    @SerialName("title")
    val title: String,
    @SerialName("body")
    val body: String,
    @SerialName("comments")
    val comments: Edge<Comment>
): Parcelable {
    override fun toString(): String {
        return this.title;
    }
}