package com.example.grafbaseandroid.Entities

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
@Parcelize
data class Comment(
    @SerialName("id")
    val id: String,
    @SerialName("message")
    val message: String,
): Parcelable {
    override fun toString(): String {
        return this.message;
    }
}