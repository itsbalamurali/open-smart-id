import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/app_provider.dart';
import '../models/session.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<AppProvider>().refreshPendingSessions();
    });
  }

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<AppProvider>();

    return Scaffold(
      appBar: AppBar(
        title: const Text('SmartID'),
        actions: [
          IconButton(
            icon: const Icon(Icons.qr_code_scanner),
            onPressed: () => Navigator.pushNamed(context, '/scanner'),
            tooltip: 'Scan QR Code',
          ),
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: () => Navigator.pushNamed(context, '/settings'),
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: provider.refreshPendingSessions,
        child: provider.pendingSessions.isEmpty
            ? _buildEmptyState()
            : _buildSessionList(provider.pendingSessions),
      ),
    );
  }

  Widget _buildEmptyState() {
    return ListView(
      children: [
        SizedBox(
          height: MediaQuery.of(context).size.height * 0.6,
          child: Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(
                  Icons.check_circle_outline,
                  size: 64,
                  color: Colors.grey[400],
                ),
                const SizedBox(height: 16),
                Text(
                  'No pending requests',
                  style: Theme.of(
                    context,
                  ).textTheme.titleMedium?.copyWith(color: Colors.grey[600]),
                ),
                const SizedBox(height: 8),
                Text(
                  'Pull down to refresh',
                  style: TextStyle(color: Colors.grey[400]),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildSessionList(List<PendingSession> sessions) {
    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: sessions.length,
      itemBuilder: (context, index) {
        final session = sessions[index];
        return _SessionCard(session: session);
      },
    );
  }
}

class _SessionCard extends StatelessWidget {
  final PendingSession session;

  const _SessionCard({required this.session});

  @override
  Widget build(BuildContext context) {
    final isAuth = session.kind == 'authentication';

    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: isAuth ? Colors.blue[100] : Colors.green[100],
          child: Icon(
            isAuth ? Icons.login : Icons.draw,
            color: isAuth ? Colors.blue[700] : Colors.green[700],
          ),
        ),
        title: Text(session.relyingPartyName ?? 'Unknown'),
        subtitle: Text(session.kindDisplay),
        trailing: const Icon(Icons.chevron_right),
        onTap: () =>
            Navigator.pushNamed(context, '/session/${session.sessionId}'),
      ),
    );
  }
}
