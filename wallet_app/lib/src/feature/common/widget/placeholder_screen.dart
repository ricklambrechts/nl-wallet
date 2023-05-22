import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import 'button/bottom_back_button.dart';

const _kPlaceholderGenericIllustration = 'assets/non-free/images/placeholder_generic_illustration.png';
const _kPlaceholderContractIllustration = 'assets/non-free/images/placeholder_contract_illustration.png';

enum PlaceholderType { generic, contract }

class PlaceholderScreen extends StatelessWidget {
  final PlaceholderType type;

  const PlaceholderScreen({required this.type, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('placeholderScreenKey'),
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).placeholderScreenTitle),
      ),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return Column(
      children: [
        const Spacer(),
        Image.asset(
          _imageAssetName(),
          scale: context.isLandscape ? 1.5 : 1,
        ),
        const SizedBox(height: 48),
        Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            _informTitle(context),
            style: Theme.of(context).textTheme.displayMedium,
            textAlign: TextAlign.center,
          ),
        ),
        const Spacer(flex: 2),
        const BottomBackButton(),
      ],
    );
  }

  String _imageAssetName() {
    switch (type) {
      case PlaceholderType.generic:
        return _kPlaceholderGenericIllustration;
      case PlaceholderType.contract:
        return _kPlaceholderContractIllustration;
    }
  }

  String _informTitle(BuildContext context) {
    switch (type) {
      case PlaceholderType.generic:
        return AppLocalizations.of(context).placeholderScreenGenericInformTitle;
      case PlaceholderType.contract:
        return AppLocalizations.of(context).placeholderScreenContractInformTitle;
    }
  }

  static void show(BuildContext context, {bool secured = true, PlaceholderType type = PlaceholderType.generic}) {
    Navigator.push(
      context,
      secured
          ? SecuredPageRoute(builder: (c) => PlaceholderScreen(type: type))
          : MaterialPageRoute(builder: (c) => PlaceholderScreen(type: type)),
    );
  }
}
