package com.example.argus_frontend

import android.bluetooth.*
import android.bluetooth.le.*
import android.content.Context
import android.os.ParcelUuid
import androidx.annotation.NonNull
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import java.util.UUID
import android.util.Log

class MainActivity: FlutterActivity() {
    private val CHANNEL = "argus_ble_channel"
    private var gattServer: BluetoothGattServer? = null
    private var bluetoothManager: BluetoothManager? = null
    private var methodChannel: MethodChannel? = null
    private var advertiser: BluetoothLeAdvertiser? = null
    private var isAdvertising = false

    // UUIDs must match Rust constants
    private val SERVICE_UUID = UUID.fromString("12345678-90ab-cdef-1234-567890abcdef")
    private val CHAR_UUID = UUID.fromString("abcdef12-3456-7890-abcd-ef1234567890")

    private val advertiseCallback = object : AdvertiseCallback() {
        override fun onStartSuccess(settingsInEffect: AdvertiseSettings?) {
            super.onStartSuccess(settingsInEffect)
            isAdvertising = true
            Log.d("ArgusGatt", "Legacy advertising started successfully")
            runOnUiThread {
                methodChannel?.invokeMethod("onAdvertisingStarted", true)
            }
        }

        override fun onStartFailure(errorCode: Int) {
            super.onStartFailure(errorCode)
            isAdvertising = false
            Log.e("ArgusGatt", "Legacy advertising failed with error: $errorCode")
            runOnUiThread {
                methodChannel?.invokeMethod("onAdvertisingStarted", false)
            }
        }
    }

    private val gattServerCallback = object : BluetoothGattServerCallback() {
        override fun onConnectionStateChange(device: BluetoothDevice, status: Int, newState: Int) {
            super.onConnectionStateChange(device, status, newState)
            val stateStr = if (newState == BluetoothProfile.STATE_CONNECTED) "CONNECTED" else "DISCONNECTED"
            Log.d("ArgusGatt", "Peer $stateStr: ${device.address} (status=$status)")
        }

        override fun onCharacteristicWriteRequest(
            device: BluetoothDevice,
            requestId: Int,
            characteristic: BluetoothGattCharacteristic,
            preparedWrite: Boolean,
            responseNeeded: Boolean,
            offset: Int,
            value: ByteArray
        ) {
            super.onCharacteristicWriteRequest(device, requestId, characteristic, preparedWrite, responseNeeded, offset, value)
            Log.d("ArgusGatt", "Write request from ${device.address} on ${characteristic.uuid}, size: ${value.size}")

            if (characteristic.uuid == CHAR_UUID) {
                // Send data to Flutter
                runOnUiThread {
                    methodChannel?.invokeMethod("onDataReceived", value)
                }
                
                if (responseNeeded) {
                    gattServer?.sendResponse(device, requestId, BluetoothGatt.GATT_SUCCESS, offset, value)
                }
            } else {
                 if (responseNeeded) {
                    gattServer?.sendResponse(device, requestId, BluetoothGatt.GATT_FAILURE, offset, null)
                }
            }
        }
    }

    override fun configureFlutterEngine(@NonNull flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        
        bluetoothManager = getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
        methodChannel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL)

        methodChannel!!.setMethodCallHandler { call, result ->
            when (call.method) {
                "startGattServer" -> {
                    val success = startGattServer()
                    result.success(success)
                }
                "stopGattServer" -> {
                    stopGattServer()
                    result.success(true)
                }
                "startAdvertising" -> {
                    val success = startAdvertising()
                    result.success(success)
                }
                "stopAdvertising" -> {
                    stopAdvertising()
                    result.success(true)
                }
                else -> result.notImplemented()
            }
        }
    }

    private fun startGattServer(): Boolean {
        if (gattServer != null) return true // Already started

        Log.d("ArgusGatt", "Starting GATT Server...")
        try {
            gattServer = bluetoothManager?.openGattServer(this, gattServerCallback)
            if (gattServer == null) {
                Log.e("ArgusGatt", "Unable to open GATT Server")
                return false
            }

            val service = BluetoothGattService(SERVICE_UUID, BluetoothGattService.SERVICE_TYPE_PRIMARY)
            
            // Add Write Characteristic
            val characteristic = BluetoothGattCharacteristic(
                CHAR_UUID,
                BluetoothGattCharacteristic.PROPERTY_WRITE or
                    BluetoothGattCharacteristic.PROPERTY_WRITE_NO_RESPONSE or
                    BluetoothGattCharacteristic.PROPERTY_NOTIFY,
                BluetoothGattCharacteristic.PERMISSION_WRITE
            )
            
            service.addCharacteristic(characteristic)
            
            val added = gattServer?.addService(service)
            Log.d("ArgusGatt", "Service added: $added")
            
            return added ?: false
        } catch (e: Exception) {
            Log.e("ArgusGatt", "Error starting GATT Server: $e")
            return false
        }
    }

    /**
     * Start BLE advertising using the LEGACY API (startAdvertising).
     * The legacy API shares the same BLE address as the GATT server,
     * ensuring that scanners can connect to the advertised address.
     */
    private fun startAdvertising(): Boolean {
        if (isAdvertising) return true

        val adapter = bluetoothManager?.adapter
        if (adapter == null || !adapter.isEnabled) {
            Log.e("ArgusGatt", "Bluetooth adapter not available or disabled")
            return false
        }

        advertiser = adapter.bluetoothLeAdvertiser
        if (advertiser == null) {
            Log.e("ArgusGatt", "BLE Advertiser not available")
            return false
        }

        val settings = AdvertiseSettings.Builder()
            .setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY)
            .setTxPowerLevel(AdvertiseSettings.ADVERTISE_TX_POWER_MEDIUM)
            .setConnectable(true)
            .setTimeout(0) // Advertise indefinitely
            .build()

        val data = AdvertiseData.Builder()
            .setIncludeDeviceName(false)
            .addServiceUuid(ParcelUuid(SERVICE_UUID))
            .build()

        val scanResponse = AdvertiseData.Builder()
            .setIncludeDeviceName(true)
            .build()

        Log.d("ArgusGatt", "Starting legacy BLE advertising...")
        try {
            advertiser?.startAdvertising(settings, data, scanResponse, advertiseCallback)
            return true
        } catch (e: Exception) {
            Log.e("ArgusGatt", "Error starting advertising: $e")
            return false
        }
    }

    private fun stopAdvertising() {
        if (isAdvertising) {
            advertiser?.stopAdvertising(advertiseCallback)
            isAdvertising = false
            Log.d("ArgusGatt", "Advertising stopped")
        }
    }

    private fun stopGattServer() {
        stopAdvertising()
        gattServer?.close()
        gattServer = null
        Log.d("ArgusGatt", "GATT Server stopped")
    }
}
