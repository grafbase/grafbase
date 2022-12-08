package com.example.grafbaseandroid.API

import kotlinx.serialization.Serializable;
import kotlinx.serialization.SerialName

@Serializable
data class GraphQLOperation (
    @SerialName("query")
    val operationString: String
)