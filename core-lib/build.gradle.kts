import gobley.gradle.GobleyHost
import gobley.gradle.cargo.dsl.*
import gobley.gradle.rust.targets.*
import gobley.gradle.Variant
import gobley.gradle.rust.targets.RustAndroidTarget
import gobley.gradle.rust.targets.RustPosixTarget

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidLibrary)
    id("maven-publish")
    id("dev.gobley.cargo") version "0.3.7"
    id("dev.gobley.uniffi") version "0.3.7"
    kotlin("plugin.atomicfu") version libs.versions.kotlin
}

uniffi {
    generateFromLibrary {
        namespace = "..."
        build = RustAndroidTarget.Arm64
        variant = Variant.Debug
    }
//
//    generateFromLibrary {
//        namespace = "..."
//        build = RustPosixTarget.LinuxX64
//        variant = Variant.Release
//    }
}

kotlin {
    jvmToolchain(21)
    androidTarget {
        publishLibraryVariants("debug")
    }
    
    jvm() // Let Gobley handle the internal resources naturally
}

cargo {
    jvmVariant = Variant.Debug
    jvmPublishingVariant = Variant.Debug

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
            abiFilters += setOf("arm64-v8a")
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

