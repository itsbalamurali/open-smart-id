import 'package:flutter/foundation.dart';
import '../models/session.dart';
import '../services/api_service.dart';
import '../services/notification_service.dart';
import '../services/secure_storage_service.dart';

class AppProvider extends ChangeNotifier {
  final ApiService api;
  final SecureStorageService storage;
  final NotificationService notifications;

  bool _isOnboarded = false;
  bool _isLoading = false;
  String? _deviceId;
  String? _accountId;
  String? _documentNumber;
  String? _semanticId;
  List<PendingSession> _pendingSessions = [];
  String? _activeSessionId;
  String? _error;

  AppProvider({
    required this.api,
    required this.storage,
    required this.notifications,
  });

  bool get isOnboarded => _isOnboarded;
  bool get isLoading => _isLoading;
  String? get deviceId => _deviceId;
  String? get accountId => _accountId;
  String? get documentNumber => _documentNumber;
  String? get semanticId => _semanticId;
  List<PendingSession> get pendingSessions => _pendingSessions;
  String? get activeSessionId => _activeSessionId;
  String? get error => _error;

  Future<void> initialize() async {
    _isOnboarded = await storage.isOnboarded();
    if (_isOnboarded) {
      _deviceId = await storage.getDeviceId();
      _accountId = await storage.getAccountId();
      _documentNumber = await storage.getDocumentNumber();
      _semanticId = await storage.getSemanticId();
    }

    // Listen for FCM push notifications
    notifications.onSessionNotification = (sessionId, kind) {
      _activeSessionId = sessionId;
      notifyListeners();
      refreshPendingSessions();
    };

    // Listen for token refresh
    notifications.onTokenRefresh.listen((newToken) async {
      if (_deviceId != null) {
        try {
          await api.updateDevice(deviceId: _deviceId!, fcmToken: newToken);
        } catch (_) {}
      }
    });

    notifyListeners();
  }

  Future<void> register({
    required String semanticId,
    required String pin,
  }) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      final fcmToken = await notifications.getToken();
      if (fcmToken == null) throw Exception('Failed to get FCM token');

      final platform = defaultTargetPlatform == TargetPlatform.iOS
          ? 'ios'
          : 'android';

      final registration = await api.registerDevice(
        semanticId: semanticId,
        fcmToken: fcmToken,
        platform: platform,
      );

      await storage.saveRegistration(
        deviceId: registration.deviceId,
        accountId: registration.accountId,
        documentNumber: registration.documentNumber,
        semanticId: semanticId,
      );
      await storage.savePin(pin);

      _deviceId = registration.deviceId;
      _accountId = registration.accountId;
      _documentNumber = registration.documentNumber;
      _semanticId = semanticId;
      _isOnboarded = true;
    } catch (e) {
      _error = e.toString();
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<void> refreshPendingSessions() async {
    if (_deviceId == null) return;
    try {
      _pendingSessions = await api.getPendingSessions(_deviceId!);
    } catch (e) {
      _pendingSessions = [];
    }
    notifyListeners();
  }

  Future<SessionDetail> getSessionDetail(String sessionId) {
    return api.getSessionDetail(sessionId);
  }

  Future<SessionActionResponse> confirmSession({
    required String sessionId,
    required String signatureValue,
    String? userChallenge,
    String? interactionTypeUsed,
  }) async {
    final result = await api.confirmSession(
      sessionId: sessionId,
      signatureValue: signatureValue,
      userChallenge: userChallenge,
      interactionTypeUsed: interactionTypeUsed,
    );
    _activeSessionId = null;
    await refreshPendingSessions();
    return result;
  }

  Future<SessionActionResponse> refuseSession(String sessionId) async {
    final result = await api.refuseSession(sessionId);
    _activeSessionId = null;
    await refreshPendingSessions();
    return result;
  }

  void clearActiveSession() {
    _activeSessionId = null;
    notifyListeners();
  }

  void clearError() {
    _error = null;
    notifyListeners();
  }

  Future<void> logout() async {
    if (_deviceId != null) {
      try {
        await api.deactivateDevice(_deviceId!);
      } catch (_) {}
    }
    await storage.clear();
    _isOnboarded = false;
    _deviceId = null;
    _accountId = null;
    _documentNumber = null;
    _semanticId = null;
    _pendingSessions = [];
    notifyListeners();
  }
}
