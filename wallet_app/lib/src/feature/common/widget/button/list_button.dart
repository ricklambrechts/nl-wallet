import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import 'button_content.dart';

const _kButtonHeight = 72.0;
const _kLandscapeButtonHeight = 56.0;

const _kVerticalPadding = 24.0;
const _kLandscapeVerticalPadding = 16.0;

/// A Button that spans the full width of the screen and wraps the [child] with optional bottom and top dividers.
class ListButton extends StatelessWidget {
  final VoidCallback? onPressed;
  final DividerSide dividerSide;
  final Text text;
  final Widget? icon;
  final IconPosition iconPosition;
  final MainAxisAlignment mainAxisAlignment;

  const ListButton({
    required this.text,
    this.icon = const Icon(Icons.arrow_forward_outlined),
    this.onPressed,
    this.dividerSide = DividerSide.horizontal,
    this.iconPosition = IconPosition.end,
    this.mainAxisAlignment = MainAxisAlignment.start,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: _resolveDividerDecoration(context),
      child: TextButton(
        style: _resolveButtonStyle(context),
        onPressed: onPressed,
        child: _buildContent(),
      ),
    );
  }

  ButtonStyle _resolveButtonStyle(BuildContext context) => context.theme.textButtonTheme.style!.copyWith(
        minimumSize: MaterialStateProperty.all(
          Size(0, context.isLandscape ? _kLandscapeButtonHeight : _kButtonHeight),
        ),
        shape: MaterialStateProperty.all(
          const LinearBorder(),
        ),
        padding: MaterialStateProperty.all(
          EdgeInsets.symmetric(
              horizontal: 16, vertical: context.isLandscape ? _kLandscapeVerticalPadding : _kVerticalPadding),
        ),
      );

  BoxDecoration _resolveDividerDecoration(BuildContext context) => BoxDecoration(
          border: Border(
        top: dividerSide.top
            ? BorderSide(
                color: context.theme.dividerTheme.color!,
                width: context.theme.dividerTheme.thickness!,
              )
            : BorderSide.none,
        bottom: dividerSide.bottom
            ? BorderSide(
                color: context.theme.dividerTheme.color!,
                width: context.theme.dividerTheme.thickness!,
              )
            : BorderSide.none,
      ));

  ButtonContent _buildContent() => ButtonContent(
        text: text,
        icon: icon,
        iconPosition: iconPosition,
        mainAxisAlignment: mainAxisAlignment,
      );
}

enum DividerSide { none, top, bottom, horizontal }

extension DividerSideExtension on DividerSide {
  bool get top => this == DividerSide.top || this == DividerSide.horizontal;

  bool get bottom => this == DividerSide.bottom || this == DividerSide.horizontal;
}
