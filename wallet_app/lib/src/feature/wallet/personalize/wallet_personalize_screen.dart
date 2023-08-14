import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../../environment.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/mapper/pid_attributes_mapper.dart';
import '../../../wallet_constants.dart';
import '../../common/page/flow_terminal_page.dart';
import '../../common/page/generic_loading_page.dart';
import '../../common/sheet/confirm_action_sheet.dart';
import '../../common/widget/animated_linear_progress_indicator.dart';
import '../../common/widget/button/animated_visibility_back_button.dart';
import '../../common/widget/fake_paging_animated_switcher.dart';
import '../../digid_help/digid_help_screen.dart';
import '../../mock_digid/mock_digid_screen.dart';
import '../../wallet/personalize/bloc/wallet_personalize_bloc.dart';
import 'page/wallet_personalize_check_data_offering_page.dart';
import 'page/wallet_personalize_confirm_pin_page.dart';
import 'page/wallet_personalize_digid_error_page.dart';
import 'page/wallet_personalize_intro_page.dart';
import 'page/wallet_personalize_success_page.dart';
import 'wallet_personalize_no_digid_screen.dart';

class WalletPersonalizeScreen extends StatelessWidget {
  const WalletPersonalizeScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'wallet_personalize_scaffold',
      appBar: AppBar(
        leading: _buildBackButton(context),
        title: Text(context.l10n.walletPersonalizeScreenTitle),
      ),
      body: WillPopScope(
        onWillPop: () async {
          if (context.bloc.state.canGoBack) {
            context.bloc.add(WalletPersonalizeOnBackPressed());
          } else {
            return _showExitSheet(context);
          }
          return false;
        },
        child: Column(
          children: [
            _buildStepper(),
            Expanded(
              child: SafeArea(
                child: _buildPage(),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBackButton(BuildContext context) {
    return BlocBuilder<WalletPersonalizeBloc, WalletPersonalizeState>(
      builder: (context, state) {
        return AnimatedVisibilityBackButton(
          visible: state.canGoBack,
          onPressed: () => context.bloc.add(WalletPersonalizeOnBackPressed()),
        );
      },
    );
  }

  Widget _buildStepper() {
    return BlocBuilder<WalletPersonalizeBloc, WalletPersonalizeState>(
      buildWhen: (prev, current) => prev.stepperProgress != current.stepperProgress,
      builder: (context, state) => AnimatedLinearProgressIndicator(progress: state.stepperProgress),
    );
  }

  Widget _buildPage() {
    return BlocConsumer<WalletPersonalizeBloc, WalletPersonalizeState>(
      listener: (context, state) {
        _closeOpenDialogs(context);
        if (state is WalletPersonalizeConnectDigid) _loginWithDigid(context, state.authUrl);
      },
      builder: (context, state) {
        Widget result = switch (state) {
          WalletPersonalizeInitial() => _buildWalletIntroPage(context),
          WalletPersonalizeConnectDigid() => _buildAuthenticatingWithDigid(context),
          WalletPersonalizeAuthenticating() => _buildAuthenticatingWithDigid(context),
          WalletPersonalizeLoadInProgress() => _buildLoading(context),
          WalletPersonalizeCheckData() => _buildCheckDataOfferingPage(context, state),
          WalletPersonalizeConfirmPin() => _buildConfirmPinPage(context, state),
          WalletPersonalizeSuccess() => _buildSuccessPage(context, state),
          WalletPersonalizeFailure() => _buildErrorPage(context),
          WalletPersonalizeDigidFailure() => _buildDigidErrorPage(context),
        };
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
  }

  /// Closes any dialogs opened on top of this [WalletPersonalizeScreen], ignored if none exist.
  void _closeOpenDialogs(BuildContext context) => Navigator.popUntil(context, (route) => route is! DialogRoute);

  Widget _buildCheckDataOfferingPage(BuildContext context, WalletPersonalizeCheckData state) {
    return WalletPersonalizeCheckDataOfferingPage(
      onAcceptPressed: () => context.bloc.add(WalletPersonalizeOfferingVerified()),
      attributes: PidAttributeMapper.map(state.availableAttributes),
    );
  }

  Widget _buildLoading(BuildContext context, {VoidCallback? onCancel}) {
    return GenericLoadingPage(
      title: context.l10n.walletPersonalizeScreenLoadingTitle,
      description: context.l10n.walletPersonalizeScreenLoadingSubtitle,
      onCancel: onCancel,
    );
  }

  Widget _buildAuthenticatingWithDigid(BuildContext context) {
    return GenericLoadingPage(
      title: context.l10n.walletPersonalizeScreenDigidLoadingTitle,
      description: context.l10n.walletPersonalizeScreenDigidLoadingSubtitle,
      cancelCta: context.l10n.walletPersonalizeScreenDigidLoadingStopCta,
      onCancel: () async {
        final bloc = context.bloc;
        final cancelled = await _showStopDigidLoginDialog(context);
        if (cancelled) bloc.add(WalletPersonalizeLoginWithDigidFailed());
      },
    );
  }

  Future<bool> _showStopDigidLoginDialog(BuildContext context) async {
    /// This check helps avoid a race condition where the dialog is opened after the state change, meaning it would
    /// not be closed by [_closeOpenDialogs] as intended.
    final isAuthenticating = context.bloc.state is WalletPersonalizeAuthenticating;
    final isConnectingToDigid = context.bloc.state is WalletPersonalizeConnectDigid;
    final shouldShowDialog = isAuthenticating || isConnectingToDigid;
    if (!shouldShowDialog) return false;

    final result = await showDialog<bool?>(
      context: context,
      builder: (BuildContext context) {
        return AlertDialog(
          title: Text(context.l10n.walletPersonalizeScreenStopDigidDialogTitle),
          content: Text(context.l10n.walletPersonalizeScreenStopDigidDialogSubtitle),
          actions: <Widget>[
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: Text(context.l10n.walletPersonalizeScreenStopDigidDialogNegativeCta),
            ),
            TextButton(
              style: Theme.of(context)
                  .textButtonTheme
                  .style
                  ?.copyWith(foregroundColor: MaterialStatePropertyAll(Theme.of(context).colorScheme.error)),
              onPressed: () => Navigator.pop(context, true),
              child: Text(context.l10n.walletPersonalizeScreenStopDigidDialogPositiveCta),
            ),
          ],
        );
      },
    );
    return result == true;
  }

  Widget _buildWalletIntroPage(BuildContext context) {
    return WalletPersonalizeIntroPage(
      onLoginWithDigidPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
      onNoDigidPressed: () => WalletPersonalizeNoDigidScreen.show(context),
    );
  }

  void _loginWithDigid(BuildContext context, String authUrl) async {
    final bloc = context.bloc;
    if (Environment.mockRepositories && !Environment.isTest) {
      // Perform the mock DigiD flow
      final loginSucceeded = (await MockDigidScreen.mockLogin(context)) == true;
      await Future.delayed(kDefaultMockDelay);
      if (loginSucceeded) {
        bloc.add(WalletPersonalizeLoginWithDigidSucceeded());
      } else {
        bloc.add(WalletPersonalizeLoginWithDigidFailed());
      }
    } else {
      try {
        launchUrlString(authUrl, mode: LaunchMode.externalApplication);
      } catch (ex) {
        Fimber.e('Failed to open auth url: $authUrl', ex: ex);
        bloc.add(WalletPersonalizeLoginWithDigidFailed());
      }
    }
  }

  Widget _buildSuccessPage(BuildContext context, WalletPersonalizeSuccess state) {
    return WalletPersonalizeSuccessPage(
      onContinuePressed: () => Navigator.restorablePushReplacementNamed(context, WalletRoutes.homeRoute),
      cards: state.cardFronts,
    );
  }

  Widget _buildErrorPage(BuildContext context) {
    return FlowTerminalPage(
      icon: Icons.not_interested,
      iconColor: context.theme.primaryColorDark,
      title: context.l10n.walletPersonalizeScreenErrorTitle,
      description: context.l10n.walletPersonalizeScreenErrorDescription,
      closeButtonCta: context.l10n.walletPersonalizeScreenErrorRetryCta,
      onClosePressed: () => context.bloc.add(WalletPersonalizeOnRetryClicked()),
    );
  }

  Widget _buildDigidErrorPage(BuildContext context) {
    return WalletPersonalizeDigidErrorPage(
      onRetryPressed: () => context.bloc.add(WalletPersonalizeLoginWithDigidClicked()),
      onHelpPressed: () => DigidHelpScreen.show(context, title: context.l10n.walletPersonalizeScreenTitle),
    );
  }

  ///FIXME: Temporary solution to make sure the user doesn't accidentally cancel the creation flow but can still exit.
  Future<bool> _showExitSheet(BuildContext context) {
    return ConfirmActionSheet.show(
      context,
      title: context.l10n.walletPersonalizeScreenExitSheetTitle,
      description: context.l10n.walletPersonalizeScreenExitSheetDescription,
      cancelButtonText: context.l10n.walletPersonalizeScreenExitSheetCancelCta,
      confirmButtonText: context.l10n.walletPersonalizeScreenExitSheetConfirmCta,
      confirmButtonColor: context.colorScheme.error,
    );
  }

  Widget _buildConfirmPinPage(BuildContext context, WalletPersonalizeConfirmPin state) {
    return WalletPersonalizeConfirmPinPage(
      onPinValidated: () => context.bloc.add(WalletPersonalizePinConfirmed()),
    );
  }
}

extension _WalletPersonalizeScreenExtension on BuildContext {
  WalletPersonalizeBloc get bloc => read<WalletPersonalizeBloc>();
}
