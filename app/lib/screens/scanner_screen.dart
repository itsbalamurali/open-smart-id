import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

class ScannerScreen extends StatefulWidget {
  const ScannerScreen({super.key});

  @override
  State<ScannerScreen> createState() => _ScannerScreenState();
}

class _ScannerScreenState extends State<ScannerScreen> {
  final _controller = MobileScannerController();
  bool _hasScanned = false;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Scan QR Code'),
        actions: [
          IconButton(
            icon: const Icon(Icons.flash_on),
            onPressed: () => _controller.toggleTorch(),
          ),
          IconButton(
            icon: const Icon(Icons.camera_front),
            onPressed: () => _controller.switchCamera(),
          ),
        ],
      ),
      body: Stack(
        children: [
          MobileScanner(controller: _controller, onDetect: _onDetect),
          // Overlay with scanning guide
          Center(
            child: Container(
              width: 250,
              height: 250,
              decoration: BoxDecoration(
                border: Border.all(color: Colors.white, width: 2),
                borderRadius: BorderRadius.circular(16),
              ),
            ),
          ),
          // Instructions
          Positioned(
            bottom: 80,
            left: 0,
            right: 0,
            child: Text(
              'Point your camera at the QR code\nshown on the service provider\'s screen',
              textAlign: TextAlign.center,
              style: TextStyle(
                color: Colors.white,
                fontSize: 16,
                shadows: [
                  Shadow(blurRadius: 8, color: Colors.black.withAlpha(180)),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _onDetect(BarcodeCapture capture) {
    if (_hasScanned) return;

    final barcodes = capture.barcodes;
    if (barcodes.isEmpty) return;

    final code = barcodes.first.rawValue;
    if (code == null) return;

    _hasScanned = true;
    _controller.stop();

    // Parse device-link URL to extract session token
    // Format: {deviceLinkBase}?sessionToken={token}
    final uri = Uri.tryParse(code);
    if (uri == null) {
      _showError('Invalid QR code');
      return;
    }

    final sessionToken = uri.queryParameters['sessionToken'];
    if (sessionToken == null) {
      _showError('Not a SmartID QR code');
      return;
    }

    // Navigate to session confirmation
    // In production: exchange the sessionToken for a sessionId via an API call
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text('Scanned session: $sessionToken')));
    Navigator.pop(context, sessionToken);
  }

  void _showError(String message) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
    Future.delayed(const Duration(seconds: 2), () {
      if (mounted) {
        _hasScanned = false;
        _controller.start();
      }
    });
  }
}
