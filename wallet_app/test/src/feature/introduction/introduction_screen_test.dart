import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/introduction/introduction_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';

void main() {
  DeviceBuilder deviceBuilder(WidgetTester tester) {
    return DeviceUtils.accessibilityDeviceBuilder
      ..addScenario(
        widget: const IntroductionScreen(),
        name: 'page_1',
      )
      ..addScenario(
        widget: const IntroductionScreen(),
        name: 'page_2',
        onCreate: (scenarioWidgetKey) async {
          await _skipPage(scenarioWidgetKey, tester);
        },
      )
      ..addScenario(
        widget: const IntroductionScreen(),
        name: 'page_3',
        onCreate: (scenarioWidgetKey) async {
          await _skipPage(scenarioWidgetKey, tester);
          await _skipPage(scenarioWidgetKey, tester);
        },
      )
      ..addScenario(
        widget: const IntroductionScreen(),
        name: 'page_4',
        onCreate: (scenarioWidgetKey) async {
          await _skipPage(scenarioWidgetKey, tester);
          await _skipPage(scenarioWidgetKey, tester);
          await _skipPage(scenarioWidgetKey, tester);
        },
      );
  }

  group('Golden Tests', () {
    testGoldens('Accessibility Light Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'accessibility_light');
    });

    testGoldens('Accessibility Dark Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'accessibility_dark');
    });
  });
}

Future<void> _skipPage(Key scenarioWidgetKey, WidgetTester tester) async {
  final finder = find.descendant(
    of: find.byKey(scenarioWidgetKey),
    matching: find.byKey(const Key('introductionNextPageCta')),
  );
  expect(finder, findsOneWidget);

  await tester.tap(finder);
  await tester.pumpAndSettle();
}
