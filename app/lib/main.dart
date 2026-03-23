import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'providers/app_provider.dart';
import 'screens/home_screen.dart';
import 'screens/onboarding_screen.dart';
import 'screens/scanner_screen.dart';
import 'screens/session_screen.dart';
import 'screens/settings_screen.dart';
import 'services/api_service.dart';
import 'services/notification_service.dart';
import 'services/secure_storage_service.dart';

@pragma('vm:entry-point')
Future<void> _firebaseBackgroundHandler(RemoteMessage message) async {
  await Firebase.initializeApp();
}

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await Firebase.initializeApp();
  FirebaseMessaging.onBackgroundMessage(_firebaseBackgroundHandler);

  final apiService = ApiService();
  final storageService = SecureStorageService();
  final notificationService = NotificationService();
  await notificationService.initialize();

  final appProvider = AppProvider(
    api: apiService,
    storage: storageService,
    notifications: notificationService,
  );
  await appProvider.initialize();

  runApp(SmartIdApp(provider: appProvider));
}

class SmartIdApp extends StatelessWidget {
  final AppProvider provider;

  const SmartIdApp({super.key, required this.provider});

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider.value(
      value: provider,
      child: MaterialApp(
        title: 'SmartID',
        debugShowCheckedModeBanner: false,
        theme: ThemeData(
          colorScheme: ColorScheme.fromSeed(
            seedColor: Colors.blue,
            brightness: Brightness.light,
          ),
          useMaterial3: true,
          appBarTheme: const AppBarTheme(centerTitle: true),
        ),
        home: Consumer<AppProvider>(
          builder: (context, provider, _) {
            if (!provider.isOnboarded) {
              return const OnboardingScreen();
            }
            // If there's an active session from push notification, go there
            if (provider.activeSessionId != null) {
              return SessionScreen(sessionId: provider.activeSessionId!);
            }
            return const HomeScreen();
          },
        ),
        onGenerateRoute: _onGenerateRoute,
      ),
    );
  }

  Route<dynamic>? _onGenerateRoute(RouteSettings settings) {
    final uri = Uri.parse(settings.name ?? '');

    if (uri.path == '/scanner') {
      return MaterialPageRoute(builder: (_) => const ScannerScreen());
    }

    if (uri.path == '/settings') {
      return MaterialPageRoute(builder: (_) => const SettingsScreen());
    }

    if (uri.pathSegments.length == 2 && uri.pathSegments[0] == 'session') {
      return MaterialPageRoute(
        builder: (_) => SessionScreen(sessionId: uri.pathSegments[1]),
      );
    }

    return null;
  }
}
