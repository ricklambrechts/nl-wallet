import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';

void main() {
  late AppLifecycleService service;

  setUp(() {
    service = AppLifecycleService();
  });

  group('AppLifecycle Changes ', () {
    test('When notifyStateChanged is called the service should reflect this change', () async {
      expectLater(
        service.observe(),
        emitsInOrder(
          [
            AppLifecycleState.inactive,
            AppLifecycleState.resumed,
            AppLifecycleState.paused,
            AppLifecycleState.detached,
          ],
        ),
      );
      service.notifyStateChanged(AppLifecycleState.inactive);
      service.notifyStateChanged(AppLifecycleState.resumed);
      service.notifyStateChanged(AppLifecycleState.paused);
      service.notifyStateChanged(AppLifecycleState.detached);
    });
  });
}
