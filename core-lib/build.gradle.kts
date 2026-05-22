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

    sourceSets {
        val commonMain by getting {
            dependencies {
                // Keep your normal dependencies here...
            }
        }
        // Cleaned up the manual jvmMain block that was conflicting
    }
}

cargo {
    packageDirectory = layout.projectDirectory.dir("${rootDir}/rust-core")

    builds {
        android {}

        jvm {
            // Gobley handles embedding automatically!
            embedRustLibrary = rustTarget == RustPosixTarget.LinuxX64 
        }

        linux {
            embedRustLibrary = rustTarget == RustPosixTarget.LinuxX64 
        }

        mingw {
            embedRustLibrary = false
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

// THIS IS THE KEY RULE FOR GOBLEY + JNA
// Gobley embeds the file, and this rule maps it to the folder JNA expects
tasks.withType<ProcessResources>().configureEach {
    // 1. Ensure the Cargo build task runs first
    val cargoTask = tasks.matching { it.name == "cargoBuildLinuxX64Release" }
    dependsOn(cargoTask)

    // 2. Explicitly pull the file from your cargo output and place it correctly
    from("path/to/your/rust/target/x86_64-unknown-linux-gnu/release") {
        include("librust_api.so")
        into("linux-x86-64") // This cleanly places it into resources/linux-x86-64/
    }
}