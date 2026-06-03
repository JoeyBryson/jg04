import gobley.gradle.GobleyHost
import gobley.gradle.cargo.dsl.*
import gobley.gradle.rust.targets.*

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidLibrary)
    id("maven-publish")
    id("dev.gobley.cargo") version "0.3.7"
    id("dev.gobley.uniffi") version "0.3.7"
    kotlin("plugin.atomicfu") version libs.versions.kotlin
}

kotlin {
    androidTarget {
        publishLibraryVariants("release")
    }
    
    jvm() // Let Gobley handle the internal resources naturally
}

cargo {
    packageDirectory = layout.projectDirectory.dir("${rootDir}/rust-core")

    builds {
        android {}

        jvm {
            embedRustLibrary = rustTarget == RustPosixTarget.LinuxX64 
        }
    }
}

android {
    namespace = "com.example.corelib"
    compileSdk = libs.versions.android.compileSdk.get().toInt()

    defaultConfig {
        minSdk = libs.versions.android.minSdk.get().toInt()
        ndk {
            abiFilters += setOf("arm64-v8a", "x86_64")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
}

publishing {
    publications.withType<MavenPublication>().configureEach {
        groupId = "com.jg04"
        version = "1.0.0"
    }
}

