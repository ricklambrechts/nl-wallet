import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/dialog/reset_wallet_dialog.dart';
import '../common/page/page_illustration.dart';
import '../common/widget/button/confirm/confirm_button.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../forgot_pin/forgot_pin_screen.dart';
import 'argument/pin_timeout_screen_argument.dart';
import 'widget/pin_timeout_description.dart';

class PinTimeoutScreen extends StatelessWidget {
  static PinTimeoutScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return PinTimeoutScreenArgument.fromMap(args as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError(
          'Make sure to pass in [PinTimeoutScreenArgument].toMap() when opening the PinTimeoutScreen');
    }
  }

  final DateTime expiryTime;

  const PinTimeoutScreen({
    required this.expiryTime,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: Scrollbar(
                thumbVisibility: true,
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: context.l10n.pinTimeoutScreenHeadline,
                      actions: const [HelpIconButton()],
                    ),
                    SliverPadding(
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                      sliver: SliverToBoxAdapter(
                        child: PinTimeoutDescription(
                          expiryTime: expiryTime,
                          onExpire: () => _onTimeoutExpired(context),
                        ),
                      ),
                    ),
                    const SliverSizedBox(height: 24),
                    const SliverPadding(
                      padding: EdgeInsets.symmetric(horizontal: 16),
                      sliver: SliverToBoxAdapter(
                        child: PageIllustration(
                          asset: WalletAssets.svg_blocked_temporary,
                          padding: EdgeInsets.zero,
                        ),
                      ),
                    ),
                    const SliverSizedBox(height: 24),
                  ],
                ),
              ),
            ),
            const Divider(height: 1),
            ConfirmButtons(
              forceVertical: !context.isLandscape,
              primaryButton: ConfirmButton(
                text: context.l10n.pinTimeoutScreenClearWalletCta,
                onPressed: () => ResetWalletDialog.show(context),
                icon: Icons.arrow_forward_outlined,
                buttonType: ConfirmButtonType.primary,
              ),
              secondaryButton: ConfirmButton(
                text: context.l10n.pinTimeoutScreenForgotPinCta,
                onPressed: () => ForgotPinScreen.show(context),
                icon: Icons.arrow_forward_outlined,
                buttonType: ConfirmButtonType.text,
              ),
            )
          ],
        ),
      ),
    );
  }

  void _onTimeoutExpired(BuildContext context) {
    // Avoid navigating if the timeout screen is not shown, this will
    // still be triggered if the user navigates back to this screen.
    if (ModalRoute.of(context)?.isCurrent != true) return;
    Navigator.pushNamedAndRemoveUntil(
      context,
      WalletRoutes.splashRoute,
      ModalRoute.withName(WalletRoutes.splashRoute),
    );
  }

  static void show(BuildContext context, DateTime expiryTime) {
    Navigator.restorablePushReplacementNamed(
      context,
      WalletRoutes.pinTimeoutRoute,
      arguments: PinTimeoutScreenArgument(expiryTime: expiryTime).toMap(),
    );
  }
}
