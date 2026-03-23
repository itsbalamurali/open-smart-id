import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class SecureStorageService {
  static const _storage = FlutterSecureStorage();

  static const _keyDeviceId = 'device_id';
  static const _keyAccountId = 'account_id';
  static const _keyDocumentNumber = 'document_number';
  static const _keySemanticId = 'semantic_id';
  static const _keyOnboarded = 'onboarded';
  static const _keyPin = 'pin';

  // Device registration
  Future<void> saveRegistration({
    required String deviceId,
    required String accountId,
    required String documentNumber,
    required String semanticId,
  }) async {
    await _storage.write(key: _keyDeviceId, value: deviceId);
    await _storage.write(key: _keyAccountId, value: accountId);
    await _storage.write(key: _keyDocumentNumber, value: documentNumber);
    await _storage.write(key: _keySemanticId, value: semanticId);
    await _storage.write(key: _keyOnboarded, value: 'true');
  }

  Future<String?> getDeviceId() => _storage.read(key: _keyDeviceId);
  Future<String?> getAccountId() => _storage.read(key: _keyAccountId);
  Future<String?> getDocumentNumber() => _storage.read(key: _keyDocumentNumber);
  Future<String?> getSemanticId() => _storage.read(key: _keySemanticId);
  Future<bool> isOnboarded() async =>
      await _storage.read(key: _keyOnboarded) == 'true';

  // PIN
  Future<void> savePin(String pin) => _storage.write(key: _keyPin, value: pin);
  Future<String?> getPin() => _storage.read(key: _keyPin);
  Future<bool> hasPin() async => await _storage.read(key: _keyPin) != null;

  // Clear all
  Future<void> clear() => _storage.deleteAll();
}
