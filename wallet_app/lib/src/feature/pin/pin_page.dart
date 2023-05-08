import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_constants.dart';
import '../../wallet_routes.dart';
import '../common/widget/button/text_icon_button.dart';
import '../common/widget/wallet_logo.dart';
import '../forgot_pin/forgot_pin_screen.dart';
import 'bloc/pin_bloc.dart';
import 'widget/pin_field.dart';
import 'widget/pin_keyboard.dart';

/// Signature for a function that creates a widget while providing the leftover pin attempts.
/// [attempts] being null indicates that this is the first attempt.
typedef PinHeaderBuilder = Widget Function(BuildContext context, int? attempts);

/// Provides pin validation and renders any errors based on the state from the nearest [PinBloc].
class PinPage extends StatelessWidget {
  final VoidCallback? onPinValidated;
  final PinHeaderBuilder? headerBuilder;

  const PinPage({
    this.onPinValidated,
    this.headerBuilder,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocListener<PinBloc, PinState>(
      listener: (context, state) {
        if (state is PinValidateSuccess) onPinValidated?.call();
        if (state is PinValidateBlocked) {
          Navigator.pushNamedAndRemoveUntil(
            context,
            WalletRoutes.splashRoute,
            ModalRoute.withName(WalletRoutes.splashRoute),
          );
        }
      },
      child: OrientationBuilder(
        builder: (context, orientation) {
          switch (orientation) {
            case Orientation.portrait:
              return _buildPortrait();
            case Orientation.landscape:
              return _buildLandscape();
          }
        },
      ),
    );
  }

  Widget _buildPortrait() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        if (headerBuilder == null) const Spacer(),
        _buildHeader(headerBuilder ?? _defaultHeaderBuilder),
        const Spacer(),
        _buildPinField(),
        const SizedBox(height: 18),
        _buildForgotCodeButton(),
        const Spacer(),
        _buildPinKeyboard(),
      ],
    );
  }

  Widget _buildLandscape() {
    return Row(
      children: [
        Expanded(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              _buildHeader(headerBuilder ?? _buildTextHeader),
              const SizedBox(height: 24),
              _buildPinField(),
              const SizedBox(height: 18),
              _buildForgotCodeButton(),
            ],
          ),
        ),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 16),
            child: _buildPinKeyboard(),
          ),
        ),
      ],
    );
  }

  Widget _buildHeader(PinHeaderBuilder builder) {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        if (state is PinValidateFailure) {
          return builder(context, state.leftoverAttempts);
        } else {
          return builder(context, null);
        }
      },
    );
  }

  Widget _defaultHeaderBuilder(BuildContext context, int? attempts) {
    return Column(
      children: [
        const WalletLogo(size: 80),
        const SizedBox(height: 24),
        _buildTextHeader(context, attempts),
      ],
    );
  }

  Widget _buildTextHeader(BuildContext context, int? attempts) {
    if (attempts == null) {
      return Column(
        children: [
          Text(
            AppLocalizations.of(context).pinScreenHeader,
            style: Theme.of(context).textTheme.displaySmall,
            textAlign: TextAlign.center,
          ),
          Text(
            '' /* makes sure the UI doesn't jump around */,
            style: Theme.of(context).textTheme.bodyLarge,
          ),
        ],
      );
    } else {
      return Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            AppLocalizations.of(context).pinScreenErrorHeader,
            style: Theme.of(context).textTheme.displaySmall?.copyWith(color: Theme.of(context).colorScheme.error),
            textAlign: TextAlign.center,
          ),
          Text(
            AppLocalizations.of(context).pinScreenAttemptsCount(attempts),
            style: Theme.of(context).textTheme.bodyLarge?.copyWith(color: Theme.of(context).colorScheme.error),
            textAlign: TextAlign.center,
          ),
        ],
      );
    }
  }

  Widget _buildPinField() {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        return PinField(
          digits: kPinDigits,
          enteredDigits: _resolveEnteredDigits(state),
        );
      },
    );
  }

  Widget _buildForgotCodeButton() {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        final buttonEnabled = state is PinEntryInProgress || state is PinValidateFailure;
        return TextIconButton(
          onPressed: buttonEnabled ? () => ForgotPinScreen.show(context) : null,
          child: Text(AppLocalizations.of(context).pinScreenForgotPinCta),
        );
      },
    );
  }

  Widget _buildPinKeyboard() {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        return AnimatedOpacity(
          duration: kDefaultAnimationDuration,
          opacity: state is PinValidateInProgress ? 0.3 : 1,
          child: PinKeyboard(
            onKeyPressed:
                _digitKeysEnabled(state) ? (digit) => context.read<PinBloc>().add(PinDigitPressed(digit)) : null,
            onBackspacePressed:
                _backspaceKeyEnabled(state) ? () => context.read<PinBloc>().add(const PinBackspacePressed()) : null,
          ),
        );
      },
    );
  }

  bool _digitKeysEnabled(PinState state) {
    if (state is PinEntryInProgress) return true;
    if (state is PinValidateFailure) return true;
    return false;
  }

  bool _backspaceKeyEnabled(PinState state) {
    if (state is PinEntryInProgress) return true;
    if (state is PinValidateFailure) return true;
    return false;
  }

  int _resolveEnteredDigits(PinState state) {
    if (state is PinEntryInProgress) return state.enteredDigits;
    if (state is PinValidateInProgress) return kPinDigits;
    if (state is PinValidateSuccess) return kPinDigits;
    if (state is PinValidateFailure) return kPinDigits;
    return 0;
  }
}
