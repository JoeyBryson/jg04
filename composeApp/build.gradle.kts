import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidApplication)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
    alias(libs.plugins.composeHotReload)
    alias(libs.plugins.kotlinSerialization)
    // kotlin("plugin.atomicfu") version libs.versions.kotlin
}

kotlin {
    androidTarget {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_11)
        }
    }

    // Temporarily disabled iOS targets
    /*
    listOf(
        iosArm64(),
        iosSimulatorArm64()
    ).forEach { iosTarget ->
        iosTarget.binaries.framework {
            baseName = "ComposeApp"
            isStatic = true
        }
    }
    */

    jvm()

    sourceSets {
        androidMain.dependencies {
            implementation(libs.androidx.activity.compose)
        }
        commonMain.dependencies {
            implementation(libs.compose.runtime)
            implementation(libs.compose.foundation)
            implementation(libs.compose.material3)
            implementation(libs.compose.ui)

            implementation("com.jg04:core-lib:1.0.0")

            implementation("org.jetbrains.androidx.lifecycle:lifecycle-runtime-compose:2.10.0")
            implementation("org.jetbrains.androidx.navigation3:navigation3-ui:1.0.0-alpha05")
            implementation("org.jetbrains.androidx.lifecycle:lifecycle-viewmodel-navigation3:2.10.0")
            implementation("org.jetbrains.kotlinx:kotlinx-serialization-core")

            implementation("androidx.navigation3:navigation3-runtime:1.1.3")

            //implementation("androidx.savedstate:savedstate:1.5.0")
            api(libs.androidx.lifecycle.viewmodel)
        }
        commonTest.dependencies {
            implementation(libs.kotlin.test)
        }
        jvmMain.dependencies {
            implementation(compose.desktop.currentOs)
            implementation(libs.kotlinx.coroutinesSwing)
            runtimeOnly("com.jg04:core-lib-jvm:1.0.0:linux-x86-64-debug")
            implementation("org.jetbrains.kotlinx:kotlinx-coroutines-swing:1.10.2")
        }
    }
}


android {
    namespace = "com.example.jg04"
    compileSdk = libs.versions.android.compileSdk.get().toInt()

    defaultConfig {
        applicationId = "com.example.jg04"
        minSdk = 26
        targetSdk = libs.versions.android.targetSdk.get().toInt()
        versionCode = 1
        versionName = "1.0"
    }
    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
}

dependencies {
    debugImplementation(libs.compose.uiTooling)
}

compose.desktop {
    application {
        mainClass = "com.example.jg04.MainKt"

        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
            packageName = "com.example.jg04"
            packageVersion = "1.0.0"
        }

    }
}

compose.resources {
    packageOfResClass = "com.example.jg04.resources"
}

//configurations.all {
//    resolutionStrategy.eachDependency {
//        // Redirect any Google AndroidX Compose core dependencies to JetBrains versions
//        if (requested.group.startsWith("androidx.compose")) {
//            // Exclude compiler or runtime modules if necessary, but target the main UI/Foundation layers
//            val targetGroup = requested.group.replace("androidx.compose", "org.jetbrains.compose")
//            useTarget("$targetGroup:${requested.name}:${requested.version}")
//        }
//    }
//}