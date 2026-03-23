import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';

class NotificationService {
  final FirebaseMessaging _messaging = FirebaseMessaging.instance;
  final FlutterLocalNotificationsPlugin _localNotifications =
      FlutterLocalNotificationsPlugin();

  Function(String sessionId, String sessionKind)? onSessionNotification;

  Future<void> initialize() async {
    // Request permission
    await _messaging.requestPermission(alert: true, badge: true, sound: true);

    // Local notifications setup
    const androidSettings = AndroidInitializationSettings(
      '@mipmap/ic_launcher',
    );
    const iosSettings = DarwinInitializationSettings();
    await _localNotifications.initialize(
      const InitializationSettings(android: androidSettings, iOS: iosSettings),
      onDidReceiveNotificationResponse: _onLocalNotificationTapped,
    );

    // Foreground messages
    FirebaseMessaging.onMessage.listen(_handleForegroundMessage);

    // Background/terminated tap
    FirebaseMessaging.onMessageOpenedApp.listen(_handleNotificationTap);

    // Check if app was opened from a notification
    final initialMessage = await _messaging.getInitialMessage();
    if (initialMessage != null) {
      _handleNotificationTap(initialMessage);
    }
  }

  Future<String?> getToken() => _messaging.getToken();

  Stream<String> get onTokenRefresh => _messaging.onTokenRefresh;

  void _handleForegroundMessage(RemoteMessage message) {
    final data = message.data;
    final notification = message.notification;

    // Show local notification
    if (notification != null) {
      _localNotifications.show(
        message.hashCode,
        notification.title ?? 'SmartID',
        notification.body ?? 'New request',
        const NotificationDetails(
          android: AndroidNotificationDetails(
            'smartid_sessions',
            'Session Requests',
            channelDescription: 'Authentication and signing requests',
            importance: Importance.high,
            priority: Priority.high,
          ),
          iOS: DarwinNotificationDetails(),
        ),
        payload: '${data['sessionId']}|${data['sessionKind']}',
      );
    }

    // Notify listeners
    final sessionId = data['sessionId'] as String?;
    final sessionKind = data['sessionKind'] as String?;
    if (sessionId != null && sessionKind != null) {
      onSessionNotification?.call(sessionId, sessionKind);
    }
  }

  void _handleNotificationTap(RemoteMessage message) {
    final sessionId = message.data['sessionId'] as String?;
    final sessionKind = message.data['sessionKind'] as String?;
    if (sessionId != null && sessionKind != null) {
      onSessionNotification?.call(sessionId, sessionKind);
    }
  }

  void _onLocalNotificationTapped(NotificationResponse response) {
    final payload = response.payload;
    if (payload != null && payload.contains('|')) {
      final parts = payload.split('|');
      onSessionNotification?.call(parts[0], parts[1]);
    }
  }
}
