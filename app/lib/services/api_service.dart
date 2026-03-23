import 'package:dio/dio.dart';
import '../models/session.dart';

class ApiService {
  final Dio _dio;

  ApiService({String baseUrl = 'http://10.0.2.2:3000'})
    : _dio = Dio(
        BaseOptions(
          baseUrl: baseUrl,
          connectTimeout: const Duration(seconds: 10),
          receiveTimeout: const Duration(seconds: 120),
          headers: {'Content-Type': 'application/json'},
        ),
      );

  // ── Device registration ──

  Future<DeviceRegistration> registerDevice({
    required String semanticId,
    required String fcmToken,
    required String platform,
    String? deviceName,
  }) async {
    final response = await _dio.post(
      '/app/v1/devices/register',
      data: {
        'semanticId': semanticId,
        'fcmToken': fcmToken,
        'platform': platform,
        'deviceName': ?deviceName,
      },
    );
    return DeviceRegistration.fromJson(response.data);
  }

  Future<void> updateDevice({
    required String deviceId,
    String? fcmToken,
    String? deviceName,
  }) async {
    await _dio.put(
      '/app/v1/devices/$deviceId',
      data: {'fcmToken': ?fcmToken, 'deviceName': ?deviceName},
    );
  }

  Future<void> deactivateDevice(String deviceId) async {
    await _dio.delete('/app/v1/devices/$deviceId');
  }

  // ── Sessions ──

  Future<List<PendingSession>> getPendingSessions(String deviceId) async {
    final response = await _dio.get(
      '/app/v1/sessions/pending',
      queryParameters: {'deviceId': deviceId},
    );
    final sessions = response.data['sessions'] as List;
    return sessions
        .map((s) => PendingSession.fromJson(s as Map<String, dynamic>))
        .toList();
  }

  Future<SessionDetail> getSessionDetail(String sessionId) async {
    final response = await _dio.get('/app/v1/sessions/$sessionId');
    return SessionDetail.fromJson(response.data);
  }

  Future<SessionActionResponse> confirmSession({
    required String sessionId,
    required String signatureValue,
    String? userChallenge,
    String? interactionTypeUsed,
    String? deviceIpAddress,
  }) async {
    final response = await _dio.post(
      '/app/v1/sessions/$sessionId/confirm',
      data: {
        'signatureValue': signatureValue,
        'userChallenge': ?userChallenge,
        'interactionTypeUsed': ?interactionTypeUsed,
        'deviceIpAddress': ?deviceIpAddress,
      },
    );
    return SessionActionResponse.fromJson(response.data);
  }

  Future<SessionActionResponse> refuseSession(String sessionId) async {
    final response = await _dio.post('/app/v1/sessions/$sessionId/refuse');
    return SessionActionResponse.fromJson(response.data);
  }
}
