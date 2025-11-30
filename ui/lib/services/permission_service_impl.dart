import 'dart:developer';

import 'package:permission_handler/permission_handler.dart';

import 'permission_service.dart';

/// PermissionService that delegates to permission_handler.
class PermissionServiceImpl implements PermissionService {
  PermissionServiceImpl({Permission? microphonePermission})
    : _microphonePermission = microphonePermission ?? Permission.microphone;

  final Permission _microphonePermission;

  @override
  Future<PermissionResult> requestMicrophone() async {
    final status = await _microphonePermission.request();
    final rationale = await _microphonePermission.shouldShowRequestRationale;
    final result = _mapStatus(status, rationale);
    _trace('permission.request', result);
    return result;
  }

  @override
  Future<PermissionResult> checkMicrophone() async {
    final status = await _microphonePermission.status;
    final rationale = await _microphonePermission.shouldShowRequestRationale;
    final result = _mapStatus(status, rationale);
    _trace('permission.check', result);
    return result;
  }

  PermissionResult _mapStatus(PermissionStatus status, bool rationale) {
    switch (status) {
      case PermissionStatus.granted:
        return const PermissionResult(state: PermissionState.granted);
      case PermissionStatus.denied:
        return PermissionResult(
          state: PermissionState.denied,
          shouldShowRationale: rationale,
        );
      case PermissionStatus.limited:
      case PermissionStatus.restricted:
        return PermissionResult(
          state: PermissionState.restricted,
          shouldShowRationale: rationale,
        );
      case PermissionStatus.permanentlyDenied:
        return const PermissionResult(
          state: PermissionState.permanentlyDenied,
          shouldShowRationale: false,
        );
      case PermissionStatus.provisional:
        return PermissionResult(
          state: PermissionState.restricted,
          shouldShowRationale: rationale,
        );
    }
  }

  void _trace(String event, PermissionResult result) {
    log(
      event,
      name: 'permission_service',
      error: {
        'state': result.state.name,
        'rationale': result.shouldShowRationale,
      },
    );
  }
}
