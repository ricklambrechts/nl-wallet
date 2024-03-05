import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/app/check_is_app_initialized_usecase.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/pin/pin_overlay.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';

void main() {
  late IsWalletInitializedUseCase isWalletInitializedUseCase;

  setUp(() {
    isWalletInitializedUseCase = Mocks.create();
    when(isWalletInitializedUseCase.invoke()).thenAnswer((realInvocation) async => true);
  });

  testWidgets('verify PinOverlay shows child when status is unlocked & registered', (tester) async {
    await tester.pumpWidget(
      RepositoryProvider<IsWalletInitializedUseCase>(
        create: (context) => isWalletInitializedUseCase,
        child: WalletAppTestWidget(
          child: PinOverlay(
            bloc: PinBloc(Mocks.create()),
            isLockedStream: Stream.value(false),
            child: const Text('child'),
          ),
        ),
      ),
    );

    // Make sure stream is processed
    await tester.pumpAndSettle();

    // Setup finders
    final titleFinder = find.text('child');

    // Verify all expected widgets show up once
    expect(titleFinder, findsOneWidget);
  });

  testWidgets('verify PinOverlay hides child when status is locked & registered', (tester) async {
    await tester.pumpWidget(
      RepositoryProvider<IsWalletInitializedUseCase>(
        create: (context) => isWalletInitializedUseCase,
        child: WalletAppTestWidget(
          child: PinOverlay(
            bloc: PinBloc(Mocks.create()),
            isLockedStream: Stream.value(true),
            child: const Text('child'),
          ),
        ),
      ),
    );

    // Make sure stream is processed
    await tester.pumpAndSettle();

    // Setup finders
    final titleFinder = find.text('child');

    // Verify the locked widget is NOT shown
    expect(titleFinder, findsNothing);
  });

  testWidgets('verify PinOverlay shows child when status is locked & NOT registered', (tester) async {
    when(isWalletInitializedUseCase.invoke()).thenAnswer((realInvocation) async => false);

    await tester.pumpWidget(
      RepositoryProvider<IsWalletInitializedUseCase>(
        create: (context) => isWalletInitializedUseCase,
        child: WalletAppTestWidget(
          child: PinOverlay(
            bloc: PinBloc(Mocks.create()),
            isLockedStream: Stream.value(true),
            child: const Text('child'),
          ),
        ),
      ),
    );

    // Make sure stream is processed
    await tester.pumpAndSettle();

    // Setup finders
    final titleFinder = find.text('child');

    // Verify all expected widgets show up once
    expect(titleFinder, findsOneWidget);
  });
}
