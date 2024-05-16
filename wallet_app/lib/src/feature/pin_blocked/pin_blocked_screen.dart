import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/dialog/reset_wallet_dialog.dart';
import '../common/page/page_illustration.dart';
import '../common/widget/button/confirm/confirm_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/text/body_text.dart';

class PinBlockedScreen extends StatelessWidget {
  const PinBlockedScreen({
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
                      title: context.l10n.pinBlockedScreenHeadline,
                      actions: const [HelpIconButton()],
                    ),
                    SliverPadding(
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                      sliver: SliverToBoxAdapter(
                        child: BodyText(context.l10n.pinBlockedScreenDescription),
                      ),
                    ),
                    const SliverSizedBox(height: 24),
                    const SliverPadding(
                      padding: EdgeInsets.symmetric(horizontal: 16),
                      sliver: SliverToBoxAdapter(
                        child: PageIllustration(
                          asset: WalletAssets.svg_blocked_final,
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
            SizedBox(height: context.orientationBasedVerticalPadding),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: ConfirmButton(
                text: context.l10n.pinBlockedScreenResetWalletCta,
                icon: Icons.arrow_forward_outlined,
                buttonType: ConfirmButtonType.primary,
                onPressed: () => ResetWalletDialog.show(context),
              ),
            ),
            SizedBox(height: context.orientationBasedVerticalPadding),
          ],
        ),
      ),
    );
  }

  static void show(BuildContext context) {
    // Remove all routes and only keep the pinBlocked route
    Navigator.pushNamedAndRemoveUntil(context, WalletRoutes.pinBlockedRoute, (Route<dynamic> route) => false);
  }
}
