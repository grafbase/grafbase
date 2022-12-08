package com.example.grafbaseandroid.Entities

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
@Parcelize
data class Edge<T: Parcelable> (
    @SerialName("edges")
    val edges: Array<Node<T>>
): Parcelable {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as Edge<*>

        if (!edges.contentEquals(other.edges)) return false

        return true
    }

    override fun hashCode(): Int {
        return edges.contentHashCode()
    }
}

@Serializable
@Parcelize
data class Node<T: Parcelable> (
    @SerialName("node")
    val node: T,
): Parcelable

@Serializable()
@Parcelize
data class GraphQLResult<T: Parcelable>(
    val data: T?,
) : Parcelable