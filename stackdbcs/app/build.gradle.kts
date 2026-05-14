import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import com.android.build.api.dsl.ApplicationExtension

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.ksp)
    alias(libs.plugins.hilt)
}

extensions.configure<ApplicationExtension> {
    namespace = "com.example.stackd_bcs"

    compileSdk = 37

    defaultConfig {
        applicationId = "com.example.stackd_bcs"

        minSdk = 26
        targetSdk = 34

        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner =
            "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            isMinifyEnabled = false

            proguardFiles(
                getDefaultProguardFile(
                    "proguard-android-optimize.txt"
                ),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    buildFeatures {
        compose = true
    }
}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}

ksp {
    arg("room.schemaLocation", "$projectDir/schemas")
}

dependencies {

    // Compose
    implementation(platform(libs.androidx.compose.bom))

    implementation(libs.androidx.activity.compose)

    implementation(libs.androidx.compose.material3)

    implementation(
        libs.androidx.compose.material3.adaptive.navigation.suite
    )

    implementation(
        libs.androidx.compose.material.icons.extended
    )

    implementation(libs.androidx.compose.ui)

    implementation(libs.androidx.compose.ui.graphics)

    implementation(
        libs.androidx.compose.ui.tooling.preview
    )

    // Core Android
    implementation(libs.androidx.core.ktx)

    implementation(
        libs.androidx.lifecycle.runtime.ktx
    )

    // Testing
    testImplementation(libs.junit)

    androidTestImplementation(
        platform(libs.androidx.compose.bom)
    )

    androidTestImplementation(
        libs.androidx.compose.ui.test.junit4
    )

    androidTestImplementation(
        libs.androidx.espresso.core
    )

    androidTestImplementation(
        libs.androidx.junit
    )

    debugImplementation(
        libs.androidx.compose.ui.tooling
    )

    debugImplementation(
        libs.androidx.compose.ui.test.manifest
    )

    // Hilt
    implementation(libs.hilt.android)

    ksp(libs.hilt.compiler)

    implementation(libs.hilt.navigation.compose)

    // CameraX
    implementation(libs.camerax.core)

    implementation(libs.camerax.camera2)

    implementation(libs.camerax.lifecycle)

    implementation(libs.camerax.view)

    // ML Kit
    implementation(libs.mlkit.barcode)

    implementation(libs.androidx.camera.mlkit.vision)

    // Room
    implementation(libs.room.runtime)

    implementation(libs.room.ktx)

    ksp(libs.room.compiler)

    // WorkManager
    implementation(libs.workmanager.ktx)

    // Ktor
    implementation(libs.ktor.client.android)

    implementation(
        libs.ktor.client.content.negotiation
    )

    implementation(
        libs.ktor.serialization.json
    )

    implementation(
        libs.kotlinx.serialization.json
    )

    // Security
    implementation(libs.security.crypto)

    // Navigation
    implementation(libs.navigation.compose)
}