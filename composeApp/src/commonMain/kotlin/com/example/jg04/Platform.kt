package com.example.jg04

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform