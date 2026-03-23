import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/app_provider.dart';

class OnboardingScreen extends StatefulWidget {
  const OnboardingScreen({super.key});

  @override
  State<OnboardingScreen> createState() => _OnboardingScreenState();
}

class _OnboardingScreenState extends State<OnboardingScreen> {
  final _pageController = PageController();
  final _semanticIdController = TextEditingController();
  final _pinController = TextEditingController();
  final _pinConfirmController = TextEditingController();

  @override
  void dispose() {
    _pageController.dispose();
    _semanticIdController.dispose();
    _pinController.dispose();
    _pinConfirmController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: PageView(
          controller: _pageController,
          physics: const NeverScrollableScrollPhysics(),
          onPageChanged: (page) => setState(() {}),
          children: [
            _buildWelcomePage(),
            _buildIdentityPage(),
            _buildPinPage(),
          ],
        ),
      ),
    );
  }

  Widget _buildWelcomePage() {
    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.verified_user, size: 96, color: Colors.blue),
          const SizedBox(height: 32),
          Text(
            'SmartID',
            style: Theme.of(
              context,
            ).textTheme.headlineLarge?.copyWith(fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 16),
          Text(
            'Your digital identity in your pocket. Authenticate and sign documents securely from your phone.',
            textAlign: TextAlign.center,
            style: Theme.of(
              context,
            ).textTheme.bodyLarge?.copyWith(color: Colors.grey[600]),
          ),
          const SizedBox(height: 48),
          FilledButton(
            onPressed: () => _pageController.nextPage(
              duration: const Duration(milliseconds: 300),
              curve: Curves.easeInOut,
            ),
            child: const Padding(
              padding: EdgeInsets.symmetric(horizontal: 32, vertical: 12),
              child: Text('Get Started'),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildIdentityPage() {
    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            'Enter your identity',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          const SizedBox(height: 8),
          Text(
            'Provide your personal number in ETSI format.',
            style: TextStyle(color: Colors.grey[600]),
          ),
          const SizedBox(height: 32),
          TextField(
            controller: _semanticIdController,
            decoration: const InputDecoration(
              labelText: 'Personal Number',
              hintText: 'PNOEE-48010010101',
              border: OutlineInputBorder(),
              prefixIcon: Icon(Icons.person),
            ),
            textCapitalization: TextCapitalization.characters,
          ),
          const SizedBox(height: 24),
          Row(
            children: [
              TextButton(
                onPressed: () => _pageController.previousPage(
                  duration: const Duration(milliseconds: 300),
                  curve: Curves.easeInOut,
                ),
                child: const Text('Back'),
              ),
              const Spacer(),
              FilledButton(
                onPressed: _semanticIdController.text.length >= 6
                    ? () => _pageController.nextPage(
                        duration: const Duration(milliseconds: 300),
                        curve: Curves.easeInOut,
                      )
                    : null,
                child: const Text('Next'),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildPinPage() {
    final provider = context.watch<AppProvider>();

    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            'Create your PIN',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          const SizedBox(height: 8),
          Text(
            'This PIN will be used to confirm authentication and signing requests.',
            style: TextStyle(color: Colors.grey[600]),
          ),
          const SizedBox(height: 32),
          TextField(
            controller: _pinController,
            decoration: const InputDecoration(
              labelText: 'PIN',
              border: OutlineInputBorder(),
              prefixIcon: Icon(Icons.lock),
            ),
            obscureText: true,
            keyboardType: TextInputType.number,
            maxLength: 6,
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _pinConfirmController,
            decoration: const InputDecoration(
              labelText: 'Confirm PIN',
              border: OutlineInputBorder(),
              prefixIcon: Icon(Icons.lock_outline),
            ),
            obscureText: true,
            keyboardType: TextInputType.number,
            maxLength: 6,
          ),
          if (provider.error != null) ...[
            const SizedBox(height: 16),
            Text(provider.error!, style: const TextStyle(color: Colors.red)),
          ],
          const SizedBox(height: 24),
          Row(
            children: [
              TextButton(
                onPressed: () => _pageController.previousPage(
                  duration: const Duration(milliseconds: 300),
                  curve: Curves.easeInOut,
                ),
                child: const Text('Back'),
              ),
              const Spacer(),
              FilledButton(
                onPressed: provider.isLoading ? null : _onRegister,
                child: provider.isLoading
                    ? const SizedBox(
                        width: 20,
                        height: 20,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Text('Register'),
              ),
            ],
          ),
        ],
      ),
    );
  }

  void _onRegister() {
    if (_pinController.text.length < 4) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('PIN must be at least 4 digits')),
      );
      return;
    }
    if (_pinController.text != _pinConfirmController.text) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('PINs do not match')));
      return;
    }

    context.read<AppProvider>().register(
      semanticId: _semanticIdController.text.trim(),
      pin: _pinController.text,
    );
  }
}
