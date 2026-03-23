import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/session.dart';
import '../providers/app_provider.dart';

class SessionScreen extends StatefulWidget {
  final String sessionId;

  const SessionScreen({super.key, required this.sessionId});

  @override
  State<SessionScreen> createState() => _SessionScreenState();
}

class _SessionScreenState extends State<SessionScreen> {
  SessionDetail? _detail;
  bool _isLoading = true;
  bool _isActing = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadSession();
  }

  Future<void> _loadSession() async {
    try {
      final detail = await context.read<AppProvider>().getSessionDetail(
        widget.sessionId,
      );
      setState(() {
        _detail = detail;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Request Details')),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
          ? Center(child: Text('Error: $_error'))
          : _buildDetail(),
    );
  }

  Widget _buildDetail() {
    final detail = _detail!;
    final isAuth = detail.kind == 'authentication';

    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Header
          Icon(
            isAuth ? Icons.login : Icons.draw,
            size: 48,
            color: isAuth ? Colors.blue : Colors.green,
          ),
          const SizedBox(height: 16),
          Text(
            isAuth ? 'Authentication Request' : 'Signing Request',
            style: Theme.of(context).textTheme.headlineSmall,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 24),

          // RP name
          _DetailRow(
            label: 'Service',
            value: detail.relyingPartyName ?? 'Unknown',
          ),
          const Divider(),

          // Hash algorithm
          if (detail.hashAlgorithm != null) ...[
            _DetailRow(label: 'Algorithm', value: detail.hashAlgorithm!),
            const Divider(),
          ],

          // Verification code
          if (detail.vc != null) ...[
            const SizedBox(height: 16),
            Text(
              'Verification Code',
              style: Theme.of(context).textTheme.titleMedium,
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 8),
            Text(
              detail.vc!.value,
              style: Theme.of(context).textTheme.displayMedium?.copyWith(
                fontWeight: FontWeight.bold,
                letterSpacing: 8,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 16),
          ],

          // Interactions text
          if (detail.interactions != null) ...[
            _buildInteractionsText(detail.interactions!),
            const SizedBox(height: 16),
          ],

          const Spacer(),

          // Action buttons
          if (detail.isRunning && !_isActing) ...[
            FilledButton(
              onPressed: _onConfirm,
              style: FilledButton.styleFrom(
                padding: const EdgeInsets.symmetric(vertical: 16),
              ),
              child: const Text('Confirm', style: TextStyle(fontSize: 18)),
            ),
            const SizedBox(height: 12),
            OutlinedButton(
              onPressed: _onRefuse,
              style: OutlinedButton.styleFrom(
                padding: const EdgeInsets.symmetric(vertical: 16),
                foregroundColor: Colors.red,
              ),
              child: const Text('Refuse', style: TextStyle(fontSize: 18)),
            ),
          ],
          if (_isActing) const Center(child: CircularProgressIndicator()),
        ],
      ),
    );
  }

  Widget _buildInteractionsText(String interactionsBase64) {
    try {
      final decoded = utf8.decode(base64.decode(interactionsBase64));
      final interactions = jsonDecode(decoded) as List;
      if (interactions.isEmpty) return const SizedBox.shrink();

      final first = interactions[0] as Map<String, dynamic>;
      final text = first['displayText200'] ?? first['displayText60'] ?? '';
      if (text.isEmpty) return const SizedBox.shrink();

      return Card(
        color: Colors.grey[100],
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Text(text, style: Theme.of(context).textTheme.bodyLarge),
        ),
      );
    } catch (_) {
      return const SizedBox.shrink();
    }
  }

  Future<void> _onConfirm() async {
    setState(() => _isActing = true);
    try {
      // In production: prompt for PIN, then sign with device key.
      // For now, send a placeholder signature.
      final result = await context.read<AppProvider>().confirmSession(
        sessionId: widget.sessionId,
        signatureValue: base64.encode(utf8.encode('device-signature')),
        interactionTypeUsed: 'displayTextAndPIN',
      );

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Confirmed: ${result.endResult}')),
        );
        Navigator.pop(context);
      }
    } catch (e) {
      if (mounted) {
        setState(() => _isActing = false);
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Error: $e')));
      }
    }
  }

  Future<void> _onRefuse() async {
    setState(() => _isActing = true);
    try {
      await context.read<AppProvider>().refuseSession(widget.sessionId);
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(const SnackBar(content: Text('Request refused')));
        Navigator.pop(context);
      }
    } catch (e) {
      if (mounted) {
        setState(() => _isActing = false);
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Error: $e')));
      }
    }
  }
}

class _DetailRow extends StatelessWidget {
  final String label;
  final String value;

  const _DetailRow({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: TextStyle(color: Colors.grey[600])),
          Text(value, style: const TextStyle(fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }
}
