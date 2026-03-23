class PendingSession {
  final String sessionId;
  final String kind;
  final String? relyingPartyName;
  final String createdAt;

  PendingSession({
    required this.sessionId,
    required this.kind,
    this.relyingPartyName,
    required this.createdAt,
  });

  factory PendingSession.fromJson(Map<String, dynamic> json) {
    return PendingSession(
      sessionId: json['sessionId'] as String,
      kind: json['kind'] as String,
      relyingPartyName: json['relyingPartyName'] as String?,
      createdAt: json['createdAt'] as String,
    );
  }

  String get kindDisplay {
    switch (kind) {
      case 'authentication':
        return 'Authentication';
      case 'signing':
        return 'Signing';
      case 'certificate_choice':
        return 'Certificate Choice';
      default:
        return kind;
    }
  }
}

class SessionDetail {
  final String sessionId;
  final String kind;
  final String state;
  final String? relyingPartyName;
  final String? interactions;
  final VerificationCode? vc;
  final String? hashAlgorithm;
  final String createdAt;

  SessionDetail({
    required this.sessionId,
    required this.kind,
    required this.state,
    this.relyingPartyName,
    this.interactions,
    this.vc,
    this.hashAlgorithm,
    required this.createdAt,
  });

  factory SessionDetail.fromJson(Map<String, dynamic> json) {
    return SessionDetail(
      sessionId: json['sessionId'] as String,
      kind: json['kind'] as String,
      state: json['state'] as String,
      relyingPartyName: json['relyingPartyName'] as String?,
      interactions: json['interactions'] as String?,
      vc: json['vc'] != null
          ? VerificationCode.fromJson(json['vc'] as Map<String, dynamic>)
          : null,
      hashAlgorithm: json['hashAlgorithm'] as String?,
      createdAt: json['createdAt'] as String,
    );
  }

  bool get isRunning => state == 'RUNNING';
}

class VerificationCode {
  final String type;
  final String value;

  VerificationCode({required this.type, required this.value});

  factory VerificationCode.fromJson(Map<String, dynamic> json) {
    return VerificationCode(
      type: json['type'] as String,
      value: json['value'] as String,
    );
  }
}

class SessionActionResponse {
  final String sessionId;
  final String state;
  final String endResult;
  final String? documentNumber;

  SessionActionResponse({
    required this.sessionId,
    required this.state,
    required this.endResult,
    this.documentNumber,
  });

  factory SessionActionResponse.fromJson(Map<String, dynamic> json) {
    return SessionActionResponse(
      sessionId: json['sessionId'] as String,
      state: json['state'] as String,
      endResult: json['endResult'] as String,
      documentNumber: json['documentNumber'] as String?,
    );
  }
}

class DeviceRegistration {
  final String deviceId;
  final String accountId;
  final String documentNumber;

  DeviceRegistration({
    required this.deviceId,
    required this.accountId,
    required this.documentNumber,
  });

  factory DeviceRegistration.fromJson(Map<String, dynamic> json) {
    return DeviceRegistration(
      deviceId: json['deviceId'] as String,
      accountId: json['accountId'] as String,
      documentNumber: json['documentNumber'] as String,
    );
  }
}
