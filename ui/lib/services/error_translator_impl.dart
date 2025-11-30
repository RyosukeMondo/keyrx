import 'dart:async';

import 'audio_service.dart';
import 'error_translator.dart';
import 'permission_service.dart';

/// Default implementation that maps internal errors to user-friendly messages.
class ErrorTranslatorImpl implements ErrorTranslator {
  const ErrorTranslatorImpl();

  @override
  UserMessage translate(Object error) {
    if (error is AudioFailure) {
      return _translateAudioFailure(error);
    }

    if (error is PermissionDeniedError) {
      return _translatePermission(error);
    }

    if (error is TimeoutException) {
      return const UserMessage(
        title: 'Operation timed out',
        body: 'The request took too long. Please try again.',
        category: MessageCategory.warning,
      );
    }

    if (error is StateError) {
      return const UserMessage(
        title: 'Engine not ready',
        body: 'The audio engine is not ready yet. Please try again.',
      );
    }

    return const UserMessage(
      title: 'Something went wrong',
      body: 'An unexpected error occurred. Please retry or restart the app.',
    );
  }

  UserMessage _translateAudioFailure(AudioFailure failure) {
    switch (failure.code) {
      case AudioErrorCode.permissionDenied:
        return const UserMessage(
          title: 'Microphone permission required',
          body:
              'Microphone access is needed to start audio processing. Please allow it in system settings.',
        );
      case AudioErrorCode.invalidBpm:
        return const UserMessage(
          title: 'Invalid tempo',
          body: 'Please choose a BPM greater than zero.',
          category: MessageCategory.warning,
        );
      case AudioErrorCode.notInitialized:
        return const UserMessage(
          title: 'Engine not ready',
          body:
              'Audio engine failed to initialize. Ensure dependencies are installed and try again.',
        );
      case AudioErrorCode.streamUnavailable:
        return const UserMessage(
          title: 'Live results unavailable',
          body: 'Classification stream is not available right now.',
          category: MessageCategory.warning,
        );
      case AudioErrorCode.startFailed:
        return const UserMessage(
          title: 'Unable to start audio',
          body:
              'Starting the audio engine failed. Check your input device and try again.',
        );
      case AudioErrorCode.stopFailed:
        return const UserMessage(
          title: 'Unable to stop audio',
          body: 'Stopping the audio engine failed. Please retry.',
          category: MessageCategory.warning,
        );
    }
  }

  UserMessage _translatePermission(PermissionDeniedError error) {
    switch (error.state) {
      case PermissionState.denied:
        return const UserMessage(
          title: 'Permission denied',
          body:
              'Microphone permission was denied. Please allow access to continue.',
          category: MessageCategory.warning,
        );
      case PermissionState.permanentlyDenied:
        return const UserMessage(
          title: 'Permission blocked',
          body:
              'Microphone permission is blocked. Open system settings to enable it.',
        );
      case PermissionState.restricted:
        return const UserMessage(
          title: 'Permission restricted',
          body:
              'Microphone access is restricted by the system or policy. Please review your device settings.',
        );
      case PermissionState.granted:
        return const UserMessage(
          title: 'Permission granted',
          body: 'Microphone access granted.',
          category: MessageCategory.info,
        );
    }
  }
}
