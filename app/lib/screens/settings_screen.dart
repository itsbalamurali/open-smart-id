import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/app_provider.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<AppProvider>();

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        children: [
          // Account info
          const _SectionHeader(title: 'Account'),
          ListTile(
            leading: const Icon(Icons.person),
            title: const Text('Identity'),
            subtitle: Text(provider.semanticId ?? 'Not registered'),
          ),
          ListTile(
            leading: const Icon(Icons.badge),
            title: const Text('Document Number'),
            subtitle: Text(provider.documentNumber ?? 'N/A'),
          ),
          ListTile(
            leading: const Icon(Icons.phone_android),
            title: const Text('Device ID'),
            subtitle: Text(
              provider.deviceId ?? 'N/A',
              style: const TextStyle(fontSize: 12),
            ),
          ),

          const Divider(),
          const _SectionHeader(title: 'Security'),
          ListTile(
            leading: const Icon(Icons.lock),
            title: const Text('Change PIN'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () {
              // TODO: implement PIN change
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('PIN change not implemented yet')),
              );
            },
          ),
          ListTile(
            leading: const Icon(Icons.fingerprint),
            title: const Text('Biometric Authentication'),
            trailing: Switch(
              value: false,
              onChanged: (value) {
                // TODO: implement biometric toggle
              },
            ),
          ),

          const Divider(),
          const _SectionHeader(title: 'About'),
          const ListTile(
            leading: Icon(Icons.info_outline),
            title: Text('Version'),
            subtitle: Text('1.0.0'),
          ),

          const Divider(),
          const SizedBox(height: 16),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: OutlinedButton(
              onPressed: () => _showLogoutDialog(context),
              style: OutlinedButton.styleFrom(
                foregroundColor: Colors.red,
                padding: const EdgeInsets.symmetric(vertical: 16),
              ),
              child: const Text('Remove Account'),
            ),
          ),
          const SizedBox(height: 32),
        ],
      ),
    );
  }

  void _showLogoutDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Remove Account'),
        content: const Text(
          'This will deactivate your device and remove all data. You will need to register again.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              Navigator.pop(ctx);
              context.read<AppProvider>().logout();
            },
            style: FilledButton.styleFrom(backgroundColor: Colors.red),
            child: const Text('Remove'),
          ),
        ],
      ),
    );
  }
}

class _SectionHeader extends StatelessWidget {
  final String title;
  const _SectionHeader({required this.title});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 4),
      child: Text(
        title,
        style: Theme.of(context).textTheme.labelLarge?.copyWith(
          color: Colors.blue[700],
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }
}
