import 'dart:async';

import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../data/repository/authentication/digid_auth_repository.dart';
import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../domain/usecase/auth/get_digid_auth_url_usecase.dart';
import '../../../../domain/usecase/auth/observe_digid_auth_status_usecase.dart';
import '../../../../domain/usecase/card/get_pid_issuance_response_usecase.dart';
import '../../../../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../../../../domain/usecase/card/wallet_add_issued_cards_usecase.dart';
import '../../../../util/extension/bloc_extension.dart';
import '../../../../wallet_constants.dart';

part 'wallet_personalize_event.dart';
part 'wallet_personalize_state.dart';

class WalletPersonalizeBloc extends Bloc<WalletPersonalizeEvent, WalletPersonalizeState> {
  final GetPidIssuanceResponseUseCase getPidIssuanceResponseUseCase;
  final WalletAddIssuedCardsUseCase walletAddIssuedCardsUseCase;
  final GetWalletCardsUseCase getWalletCardsUseCase;
  final GetDigidAuthUrlUseCase getDigidAuthUrlUseCase;
  final ObserveDigidAuthStatusUseCase observeDigidAuthStatusUseCase;

  StreamSubscription? _digidAuthStatusSubscription;

  WalletPersonalizeBloc(
    this.getPidIssuanceResponseUseCase,
    this.walletAddIssuedCardsUseCase,
    this.getWalletCardsUseCase,
    this.getDigidAuthUrlUseCase,
    this.observeDigidAuthStatusUseCase,
  ) : super(const WalletPersonalizeInitial()) {
    on<WalletPersonalizeLoginWithDigidClicked>(_onLoginWithDigidClicked);
    on<WalletPersonalizeLoginWithDigidSucceeded>(_onLoginWithDigidSucceeded);
    on<WalletPersonalizeLoginWithDigidFailed>(_onLoginWithDigidFailed);
    on<WalletPersonalizeOfferingVerified>(_onOfferingVerified);
    on<WalletPersonalizePinConfirmed>(_onPinConfirmed);
    on<WalletPersonalizeOnBackPressed>(_onBackPressed);
    on<WalletPersonalizeOnRetryClicked>(_onRetryClicked);
    on<WalletPersonalizeAuthInProgress>(_onAuthInProgress);

    _digidAuthStatusSubscription = observeDigidAuthStatusUseCase.invoke().listen(_handleDigidAuthStatusUpdate);
  }

  void _handleDigidAuthStatusUpdate(event) {
    if (state is WalletPersonalizeDigidFailure) return; // Don't navigate when user cancelled.
    switch (event) {
      case DigidAuthStatus.success:
        add(WalletPersonalizeLoginWithDigidSucceeded());
        break;
      case DigidAuthStatus.error:
        add(WalletPersonalizeLoginWithDigidFailed());
        break;
      case DigidAuthStatus.authenticating:
        add(WalletPersonalizeAuthInProgress());
        break;
    }
  }

  void _onLoginWithDigidClicked(event, emit) async {
    try {
      emit(const WalletPersonalizeLoadingIssuanceUrl());
      String url = await getDigidAuthUrlUseCase.invoke();
      emit(WalletPersonalizeConnectDigid(url));
    } catch (ex, stack) {
      Fimber.e('Failed to get authentication url', ex: ex, stacktrace: stack);
      handleError(
        ex,
        onUnhandledError: (ex) => emit(WalletPersonalizeDigidFailure()),
      );
    }
  }

  void _onLoginWithDigidSucceeded(event, emit) async {
    try {
      final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
      final allAttributes = issuanceResponse.cards.map((e) => e.attributes).flattened;
      emit(WalletPersonalizeCheckData(availableAttributes: allAttributes.toList()));
    } catch (ex, stack) {
      Fimber.e('Failed to get PID', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }

  void _onLoginWithDigidFailed(event, emit) async => emit(WalletPersonalizeDigidFailure());

  void _onOfferingVerified(WalletPersonalizeOfferingVerified event, emit) async {
    emit(const WalletPersonalizeConfirmPin());
  }

  void _onRetryClicked(event, emit) async => emit(const WalletPersonalizeInitial());

  void _onAuthInProgress(event, emit) async => emit(const WalletPersonalizeAuthenticating());

  void _onBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is WalletPersonalizeConfirmPin) {
        final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
        final allAttributes = issuanceResponse.cards.map((e) => e.attributes).flattened;
        emit(
          WalletPersonalizeCheckData(
            didGoBack: true,
            availableAttributes: allAttributes.toList(),
          ),
        );
      }
    }
  }

  Future<void> _onPinConfirmed(event, emit) async {
    final state = this.state;
    if (state is WalletPersonalizeConfirmPin) {
      emit(const WalletPersonalizeLoadInProgress(5));
      await Future.delayed(kDefaultMockDelay);
      try {
        final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
        await walletAddIssuedCardsUseCase.invoke(issuanceResponse.cards, issuanceResponse.organization);
        await _loadCardsAndEmitSuccessState(event, emit);
      } catch (ex, stack) {
        Fimber.e('Failed to add cards to wallet', ex: ex, stacktrace: stack);
        emit(WalletPersonalizeFailure());
      }
    }
  }

  Future<void> _loadCardsAndEmitSuccessState(event, emit) async {
    try {
      final cards = await getWalletCardsUseCase.invoke();
      emit(WalletPersonalizeSuccess(cards));
    } catch (ex, stack) {
      Fimber.e('Failed to fetch cards from wallet', ex: ex, stacktrace: stack);
      emit(WalletPersonalizeFailure());
    }
  }

  @override
  Future<void> close() async {
    _digidAuthStatusSubscription?.cancel();
    super.close();
  }
}
