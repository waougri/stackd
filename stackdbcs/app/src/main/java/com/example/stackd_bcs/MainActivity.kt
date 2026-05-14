package com.example.stackd_bcs

import android.Manifest
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.camera.core.ImageAnalysis
import androidx.camera.mlkit.vision.MlKitAnalyzer
import androidx.camera.view.LifecycleCameraController
import androidx.camera.view.PreviewView
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.FlashOff
import androidx.compose.material.icons.filled.FlashOn
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.QrCodeScanner
import androidx.compose.material.icons.filled.SwapHoriz
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.adaptive.navigationsuite.NavigationSuiteScaffold
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.example.stackd_bcs.ui.theme.StackdbcsTheme
import com.google.mlkit.vision.barcode.BarcodeScannerOptions
import com.google.mlkit.vision.barcode.BarcodeScanning
import com.google.mlkit.vision.barcode.common.Barcode
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable

@Serializable
sealed interface Routes {

    @Serializable
    object HomeRoute

    @Serializable
    object HistoryRoute

    @Serializable
    object ScannerRoute

}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            StackdbcsTheme {
                MainApp()
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ScannerScreen(onNavigateBack: () -> Unit) {
    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current

    var isTorchOn by remember { mutableStateOf(false) }
    var scannedText by remember { mutableStateOf("Scan a code") }
    var hasPermission by remember { mutableStateOf(false) }
    var barcodeBounds by remember { mutableStateOf<android.graphics.Rect?>(null) }

    val launcher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.RequestPermission(),
        onResult = { granted -> hasPermission = granted })

    LaunchedEffect(Unit) { launcher.launch(Manifest.permission.CAMERA) }

    // 1. Create the controller
    val cameraController = remember { LifecycleCameraController(context) }

    // 2. Configure the Analyzer
    LaunchedEffect(cameraController) {
        val options =
            BarcodeScannerOptions.Builder().setBarcodeFormats(Barcode.FORMAT_QR_CODE, Barcode.FORMAT_UPC_A, Barcode.FORMAT_UPC_E).build()
        val barcodeScanner = BarcodeScanning.getClient(options)

        cameraController.setImageAnalysisAnalyzer(
            ContextCompat.getMainExecutor(context), MlKitAnalyzer(
                listOf(barcodeScanner),
                ImageAnalysis.COORDINATE_SYSTEM_VIEW_REFERENCED,
                ContextCompat.getMainExecutor(context)
            ) { result ->
                val barcodes = result?.getValue(barcodeScanner)
                if (!barcodes.isNullOrEmpty()) {
                    val barcode = barcodes.first()
                    scannedText = barcodes.first().rawValue ?: "unknown data"
                    barcodeBounds = barcode.boundingBox
                } else {
                    barcodeBounds = null
                }
            })
        // 3. Bind to lifecycle here
        cameraController.bindToLifecycle(lifecycleOwner)
    }

    Box(modifier = Modifier.fillMaxSize()) {
        if (hasPermission) {
            // 4. Use PreviewView with the controller directly
            AndroidView(
                modifier = Modifier.fillMaxSize(), factory = { ctx ->
                    PreviewView(ctx).apply {
                        controller = cameraController
                    }
                })

            IconButton(
                onClick = {
                    isTorchOn = !isTorchOn
                    cameraController.enableTorch(isTorchOn) // Use the controller for torch
                }, modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(16.dp)
            ) {
                Icon(
                    imageVector = if (isTorchOn) Icons.Default.FlashOn else Icons.Default.FlashOff,
                    contentDescription = "Toggle Torch"
                )
            }

            Canvas(modifier = Modifier.fillMaxSize()) {
                barcodeBounds?.let { rect ->
                    drawRect(
                        color = Color.Yellow,
                        topLeft = Offset(rect.left.toFloat(), rect.top.toFloat()),
                        size = Size(rect.width().toFloat(), rect.height().toFloat()),
                        style = Stroke(width = 8f)
                    )

                }
            }

            Text(
                text = scannedText,
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(32.dp),
                color = MaterialTheme.colorScheme.onPrimary
            )
        } else {
            Box(contentAlignment = Alignment.Center, modifier = Modifier.fillMaxSize()) {
                Text("Camera permission is required to scan.")
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainApp() {
    val navController = rememberNavController()
    var currentRoute by remember { mutableStateOf<Any>(Routes.HomeRoute) }

    NavigationSuiteScaffold(
        navigationSuiteItems = {
            item(
                selected = currentRoute is Routes.HomeRoute,
                onClick = {
                    currentRoute = Routes.HomeRoute
                    navController.navigate(Routes.HomeRoute) {
                        popUpTo(0)
                    }
                },
                icon = { Icon(Icons.Default.Home, contentDescription = null) },
                label = { Text("Home") })
            item(
                selected = currentRoute is Routes.HistoryRoute,
                onClick = {
                    currentRoute = Routes.HistoryRoute
                    navController.navigate(Routes.HistoryRoute) {
                        popUpTo(0)
                    }
                },
                icon = { Icon(Icons.Default.History, contentDescription = null) },
                label = { Text("History") })
        }) {
        NavHost(
            navController = navController,
            startDestination = Routes.HomeRoute,
            modifier = Modifier.fillMaxSize()
        ) {
            composable<Routes.HomeRoute> {
                HomeScreen(onScanRequested = { navController.navigate(Routes.ScannerRoute) })

            }
            composable<Routes.HistoryRoute> {
                HistoryScreen()
            }
            composable<Routes.ScannerRoute> {
                ScannerScreen({ navController.popBackStack() })
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    onScanRequested: () -> Unit
) {
    val scrollBehavior = TopAppBarDefaults.pinnedScrollBehavior()
    var showSheet by remember { mutableStateOf(false) }
    val sheetState = rememberModalBottomSheetState()
    val scope = rememberCoroutineScope()

    Scaffold(modifier = Modifier.nestedScroll(scrollBehavior.nestedScrollConnection), topBar = {
        CenterAlignedTopAppBar(
            title = { Text("StackD BCS") }, scrollBehavior = scrollBehavior
        )
    }, floatingActionButton = {
        ExtendedFloatingActionButton(
            onClick = { showSheet = true },
            icon = { Icon(Icons.Default.Add, contentDescription = null) },
            text = { Text("Actions") })
    }) { innerPadding ->
        Column(
            modifier = Modifier
                .padding(innerPadding)
                .fillMaxSize()
                .padding(16.dp)
        ) {
            Text(
                text = "Welcome to the latest StackD BCS UI!",
                style = MaterialTheme.typography.headlineSmall
            )
            Text(
                text = "This app uses Material 3 Adaptive, Type-safe Navigation, and Edge-to-Edge.",
                style = MaterialTheme.typography.bodyMedium,
                modifier = Modifier.padding(top = 8.dp)
            )
        }

        if (showSheet) {
            ModalBottomSheet(
                onDismissRequest = { showSheet = false }, sheetState = sheetState
            ) {
                // Sheet content
                Column(modifier = Modifier.padding(bottom = 32.dp)) {
                    ListItem(
                        headlineContent = { Text("Log Event") },
                        leadingContent = { Icon(Icons.Default.Add, contentDescription = null) },
                        modifier = Modifier.clickable {
                            scope.launch { sheetState.hide() }.invokeOnCompletion {
                                if (!sheetState.isVisible) showSheet = false
                            }
                        })
                    ListItem(headlineContent = { Text("Move Stock") }, leadingContent = {
                        Icon(
                            Icons.Default.SwapHoriz, contentDescription = null
                        )
                    }, modifier = Modifier.clickable {
                        scope.launch { sheetState.hide() }.invokeOnCompletion {
                            if (!sheetState.isVisible) showSheet = false
                        }
                    })
                    ListItem(headlineContent = { Text("Scan QR") }, leadingContent = {
                        Icon(
                            Icons.Default.QrCodeScanner, contentDescription = null
                        )
                    }, modifier = Modifier.clickable {
                        scope.launch { sheetState.hide() }.invokeOnCompletion {
                            if (!sheetState.isVisible) showSheet = false
                            onScanRequested()
                        }
                    })
                }
            }
        }
    }
}

@Composable
fun HistoryScreen() {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Text("History Screen - Coming Soon")
    }
}

// Extension to avoid repetitive code for clickable in ListItem if needed,
// but here I'll just use the standard Modifier.clickable.
// Wait, I need to import clickable.

@Preview(showBackground = true)
@Composable
fun DefaultPreview() {
    StackdbcsTheme {
        MainApp()
    }
}
