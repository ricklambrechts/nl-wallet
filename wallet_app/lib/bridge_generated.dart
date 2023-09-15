// AUTO GENERATED FILE, DO NOT EDIT.
// Generated by `flutter_rust_bridge`@ 1.78.0.
// ignore_for_file: non_constant_identifier_names, unused_element, duplicate_ignore, directives_ordering, curly_braces_in_flow_control_structures, unnecessary_lambdas, slash_for_doc_comments, prefer_const_literals_to_create_immutables, implicit_dynamic_list_literal, duplicate_import, unused_import, unnecessary_import, prefer_single_quotes, prefer_const_constructors, use_super_parameters, always_use_package_imports, annotate_overrides, invalid_use_of_protected_member, constant_identifier_names, invalid_use_of_internal_member, prefer_is_empty, unnecessary_const

import 'dart:convert';
import 'dart:async';
import 'package:meta/meta.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:uuid/uuid.dart';
import 'package:freezed_annotation/freezed_annotation.dart' hide protected;

import 'dart:ffi' as ffi;

part 'bridge_generated.freezed.dart';

abstract class WalletCore {
  Future<void> init({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kInitConstMeta;

  Future<bool> isInitialized({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kIsInitializedConstMeta;

  Future<PinValidationResult> isValidPin({required String pin, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kIsValidPinConstMeta;

  Stream<bool> setLockStream({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kSetLockStreamConstMeta;

  Future<void> clearLockStream({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kClearLockStreamConstMeta;

  Stream<FlutterConfiguration> setConfigurationStream({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kSetConfigurationStreamConstMeta;

  Future<void> clearConfigurationStream({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kClearConfigurationStreamConstMeta;

  Stream<List<Card>> setCardsStream({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kSetCardsStreamConstMeta;

  Future<void> clearCardsStream({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kClearCardsStreamConstMeta;

  Future<WalletUnlockResult> unlockWallet({required String pin, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kUnlockWalletConstMeta;

  Future<void> lockWallet({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kLockWalletConstMeta;

  Future<bool> hasRegistration({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kHasRegistrationConstMeta;

  Future<void> register({required String pin, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kRegisterConstMeta;

  Future<String> createPidIssuanceRedirectUri({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kCreatePidIssuanceRedirectUriConstMeta;

  Future<void> cancelPidIssuance({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kCancelPidIssuanceConstMeta;

  Future<void> rejectPidIssuance({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kRejectPidIssuanceConstMeta;

  Future<void> acceptPidIssuance({required String pin, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kAcceptPidIssuanceConstMeta;

  Stream<ProcessUriEvent> processUri({required String uri, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kProcessUriConstMeta;
}

class Card {
  final int id;
  final String docType;
  final String issuer;
  final List<CardAttribute> attributes;

  const Card({
    required this.id,
    required this.docType,
    required this.issuer,
    required this.attributes,
  });
}

class CardAttribute {
  final String key;
  final List<LocalizedString> labels;
  final CardValue value;

  const CardAttribute({
    required this.key,
    required this.labels,
    required this.value,
  });
}

@freezed
class CardValue with _$CardValue {
  const factory CardValue.string({
    required String value,
  }) = CardValue_String;
  const factory CardValue.integer({
    required int value,
  }) = CardValue_Integer;
  const factory CardValue.double({
    required double value,
  }) = CardValue_Double;
  const factory CardValue.boolean({
    required bool value,
  }) = CardValue_Boolean;
  const factory CardValue.date({
    required String value,
  }) = CardValue_Date;
}

class FlutterConfiguration {
  final int inactiveLockTimeout;
  final int backgroundLockTimeout;

  const FlutterConfiguration({
    required this.inactiveLockTimeout,
    required this.backgroundLockTimeout,
  });
}

class LocalizedString {
  final String language;
  final String value;

  const LocalizedString({
    required this.language,
    required this.value,
  });
}

@freezed
class PidIssuanceEvent with _$PidIssuanceEvent {
  const factory PidIssuanceEvent.authenticating() = PidIssuanceEvent_Authenticating;
  const factory PidIssuanceEvent.success({
    required List<Card> previewCards,
  }) = PidIssuanceEvent_Success;
  const factory PidIssuanceEvent.error({
    required String data,
  }) = PidIssuanceEvent_Error;
}

enum PinValidationResult {
  Ok,
  TooFewUniqueDigits,
  SequentialDigits,
  OtherIssue,
}

@freezed
class ProcessUriEvent with _$ProcessUriEvent {
  const factory ProcessUriEvent.pidIssuance({
    required PidIssuanceEvent event,
  }) = ProcessUriEvent_PidIssuance;
  const factory ProcessUriEvent.unknownUri() = ProcessUriEvent_UnknownUri;
}

@freezed
class WalletUnlockResult with _$WalletUnlockResult {
  const factory WalletUnlockResult.ok() = WalletUnlockResult_Ok;
  const factory WalletUnlockResult.incorrectPin({
    required int leftoverAttempts,
    required bool isFinalAttempt,
  }) = WalletUnlockResult_IncorrectPin;
  const factory WalletUnlockResult.timeout({
    required int timeoutMillis,
  }) = WalletUnlockResult_Timeout;
  const factory WalletUnlockResult.blocked() = WalletUnlockResult_Blocked;
}

class WalletCoreImpl implements WalletCore {
  final WalletCorePlatform _platform;
  factory WalletCoreImpl(ExternalLibrary dylib) => WalletCoreImpl.raw(WalletCorePlatform(dylib));

  /// Only valid on web/WASM platforms.
  factory WalletCoreImpl.wasm(FutureOr<WasmModule> module) => WalletCoreImpl(module as ExternalLibrary);
  WalletCoreImpl.raw(this._platform);
  Future<void> init({dynamic hint}) {
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_init(port_),
      parseSuccessData: _wire2api_unit,
      constMeta: kInitConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kInitConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "init",
        argNames: [],
      );

  Future<bool> isInitialized({dynamic hint}) {
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_is_initialized(port_),
      parseSuccessData: _wire2api_bool,
      constMeta: kIsInitializedConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kIsInitializedConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "is_initialized",
        argNames: [],
      );

  Future<PinValidationResult> isValidPin({required String pin, dynamic hint}) {
    var arg0 = _platform.api2wire_String(pin);
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_is_valid_pin(port_, arg0),
      parseSuccessData: _wire2api_pin_validation_result,
      constMeta: kIsValidPinConstMeta,
      argValues: [pin],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kIsValidPinConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "is_valid_pin",
        argNames: ["pin"],
      );

  Stream<bool> setLockStream({dynamic hint}) {
    return _platform.executeStream(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_set_lock_stream(port_),
      parseSuccessData: _wire2api_bool,
      constMeta: kSetLockStreamConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kSetLockStreamConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "set_lock_stream",
        argNames: [],
      );

  Future<void> clearLockStream({dynamic hint}) {
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_clear_lock_stream(port_),
      parseSuccessData: _wire2api_unit,
      constMeta: kClearLockStreamConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kClearLockStreamConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "clear_lock_stream",
        argNames: [],
      );

  Stream<FlutterConfiguration> setConfigurationStream({dynamic hint}) {
    return _platform.executeStream(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_set_configuration_stream(port_),
      parseSuccessData: _wire2api_flutter_configuration,
      constMeta: kSetConfigurationStreamConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kSetConfigurationStreamConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "set_configuration_stream",
        argNames: [],
      );

  Future<void> clearConfigurationStream({dynamic hint}) {
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_clear_configuration_stream(port_),
      parseSuccessData: _wire2api_unit,
      constMeta: kClearConfigurationStreamConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kClearConfigurationStreamConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "clear_configuration_stream",
        argNames: [],
      );

  Stream<List<Card>> setCardsStream({dynamic hint}) {
    return _platform.executeStream(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_set_cards_stream(port_),
      parseSuccessData: _wire2api_list_card,
      constMeta: kSetCardsStreamConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kSetCardsStreamConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "set_cards_stream",
        argNames: [],
      );

  Future<void> clearCardsStream({dynamic hint}) {
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_clear_cards_stream(port_),
      parseSuccessData: _wire2api_unit,
      constMeta: kClearCardsStreamConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kClearCardsStreamConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "clear_cards_stream",
        argNames: [],
      );

  Future<WalletUnlockResult> unlockWallet({required String pin, dynamic hint}) {
    var arg0 = _platform.api2wire_String(pin);
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_unlock_wallet(port_, arg0),
      parseSuccessData: _wire2api_wallet_unlock_result,
      constMeta: kUnlockWalletConstMeta,
      argValues: [pin],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kUnlockWalletConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "unlock_wallet",
        argNames: ["pin"],
      );

  Future<void> lockWallet({dynamic hint}) {
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_lock_wallet(port_),
      parseSuccessData: _wire2api_unit,
      constMeta: kLockWalletConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kLockWalletConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "lock_wallet",
        argNames: [],
      );

  Future<bool> hasRegistration({dynamic hint}) {
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_has_registration(port_),
      parseSuccessData: _wire2api_bool,
      constMeta: kHasRegistrationConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kHasRegistrationConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "has_registration",
        argNames: [],
      );

  Future<void> register({required String pin, dynamic hint}) {
    var arg0 = _platform.api2wire_String(pin);
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_register(port_, arg0),
      parseSuccessData: _wire2api_unit,
      constMeta: kRegisterConstMeta,
      argValues: [pin],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kRegisterConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "register",
        argNames: ["pin"],
      );

  Future<String> createPidIssuanceRedirectUri({dynamic hint}) {
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_create_pid_issuance_redirect_uri(port_),
      parseSuccessData: _wire2api_String,
      constMeta: kCreatePidIssuanceRedirectUriConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kCreatePidIssuanceRedirectUriConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "create_pid_issuance_redirect_uri",
        argNames: [],
      );

  Future<void> cancelPidIssuance({dynamic hint}) {
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_cancel_pid_issuance(port_),
      parseSuccessData: _wire2api_unit,
      constMeta: kCancelPidIssuanceConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kCancelPidIssuanceConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "cancel_pid_issuance",
        argNames: [],
      );

  Future<void> rejectPidIssuance({dynamic hint}) {
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_reject_pid_issuance(port_),
      parseSuccessData: _wire2api_unit,
      constMeta: kRejectPidIssuanceConstMeta,
      argValues: [],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kRejectPidIssuanceConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "reject_pid_issuance",
        argNames: [],
      );

  Future<void> acceptPidIssuance({required String pin, dynamic hint}) {
    var arg0 = _platform.api2wire_String(pin);
    return _platform.executeNormal(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_accept_pid_issuance(port_, arg0),
      parseSuccessData: _wire2api_unit,
      constMeta: kAcceptPidIssuanceConstMeta,
      argValues: [pin],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kAcceptPidIssuanceConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "accept_pid_issuance",
        argNames: ["pin"],
      );

  Stream<ProcessUriEvent> processUri({required String uri, dynamic hint}) {
    var arg0 = _platform.api2wire_String(uri);
    return _platform.executeStream(FlutterRustBridgeTask(
      callFfi: (port_) => _platform.inner.wire_process_uri(port_, arg0),
      parseSuccessData: _wire2api_process_uri_event,
      constMeta: kProcessUriConstMeta,
      argValues: [uri],
      hint: hint,
    ));
  }

  FlutterRustBridgeTaskConstMeta get kProcessUriConstMeta => const FlutterRustBridgeTaskConstMeta(
        debugName: "process_uri",
        argNames: ["uri"],
      );

  void dispose() {
    _platform.dispose();
  }
// Section: wire2api

  String _wire2api_String(dynamic raw) {
    return raw as String;
  }

  bool _wire2api_bool(dynamic raw) {
    return raw as bool;
  }

  PidIssuanceEvent _wire2api_box_autoadd_pid_issuance_event(dynamic raw) {
    return _wire2api_pid_issuance_event(raw);
  }

  Card _wire2api_card(dynamic raw) {
    final arr = raw as List<dynamic>;
    if (arr.length != 4) throw Exception('unexpected arr length: expect 4 but see ${arr.length}');
    return Card(
      id: _wire2api_i64(arr[0]),
      docType: _wire2api_String(arr[1]),
      issuer: _wire2api_String(arr[2]),
      attributes: _wire2api_list_card_attribute(arr[3]),
    );
  }

  CardAttribute _wire2api_card_attribute(dynamic raw) {
    final arr = raw as List<dynamic>;
    if (arr.length != 3) throw Exception('unexpected arr length: expect 3 but see ${arr.length}');
    return CardAttribute(
      key: _wire2api_String(arr[0]),
      labels: _wire2api_list_localized_string(arr[1]),
      value: _wire2api_card_value(arr[2]),
    );
  }

  CardValue _wire2api_card_value(dynamic raw) {
    switch (raw[0]) {
      case 0:
        return CardValue_String(
          value: _wire2api_String(raw[1]),
        );
      case 1:
        return CardValue_Integer(
          value: _wire2api_i64(raw[1]),
        );
      case 2:
        return CardValue_Double(
          value: _wire2api_f64(raw[1]),
        );
      case 3:
        return CardValue_Boolean(
          value: _wire2api_bool(raw[1]),
        );
      case 4:
        return CardValue_Date(
          value: _wire2api_String(raw[1]),
        );
      default:
        throw Exception("unreachable");
    }
  }

  double _wire2api_f64(dynamic raw) {
    return raw as double;
  }

  FlutterConfiguration _wire2api_flutter_configuration(dynamic raw) {
    final arr = raw as List<dynamic>;
    if (arr.length != 2) throw Exception('unexpected arr length: expect 2 but see ${arr.length}');
    return FlutterConfiguration(
      inactiveLockTimeout: _wire2api_u16(arr[0]),
      backgroundLockTimeout: _wire2api_u16(arr[1]),
    );
  }

  int _wire2api_i32(dynamic raw) {
    return raw as int;
  }

  int _wire2api_i64(dynamic raw) {
    return castInt(raw);
  }

  List<Card> _wire2api_list_card(dynamic raw) {
    return (raw as List<dynamic>).map(_wire2api_card).toList();
  }

  List<CardAttribute> _wire2api_list_card_attribute(dynamic raw) {
    return (raw as List<dynamic>).map(_wire2api_card_attribute).toList();
  }

  List<LocalizedString> _wire2api_list_localized_string(dynamic raw) {
    return (raw as List<dynamic>).map(_wire2api_localized_string).toList();
  }

  LocalizedString _wire2api_localized_string(dynamic raw) {
    final arr = raw as List<dynamic>;
    if (arr.length != 2) throw Exception('unexpected arr length: expect 2 but see ${arr.length}');
    return LocalizedString(
      language: _wire2api_String(arr[0]),
      value: _wire2api_String(arr[1]),
    );
  }

  PidIssuanceEvent _wire2api_pid_issuance_event(dynamic raw) {
    switch (raw[0]) {
      case 0:
        return PidIssuanceEvent_Authenticating();
      case 1:
        return PidIssuanceEvent_Success(
          previewCards: _wire2api_list_card(raw[1]),
        );
      case 2:
        return PidIssuanceEvent_Error(
          data: _wire2api_String(raw[1]),
        );
      default:
        throw Exception("unreachable");
    }
  }

  PinValidationResult _wire2api_pin_validation_result(dynamic raw) {
    return PinValidationResult.values[raw as int];
  }

  ProcessUriEvent _wire2api_process_uri_event(dynamic raw) {
    switch (raw[0]) {
      case 0:
        return ProcessUriEvent_PidIssuance(
          event: _wire2api_box_autoadd_pid_issuance_event(raw[1]),
        );
      case 1:
        return ProcessUriEvent_UnknownUri();
      default:
        throw Exception("unreachable");
    }
  }

  int _wire2api_u16(dynamic raw) {
    return raw as int;
  }

  int _wire2api_u64(dynamic raw) {
    return castInt(raw);
  }

  int _wire2api_u8(dynamic raw) {
    return raw as int;
  }

  Uint8List _wire2api_uint_8_list(dynamic raw) {
    return raw as Uint8List;
  }

  void _wire2api_unit(dynamic raw) {
    return;
  }

  WalletUnlockResult _wire2api_wallet_unlock_result(dynamic raw) {
    switch (raw[0]) {
      case 0:
        return WalletUnlockResult_Ok();
      case 1:
        return WalletUnlockResult_IncorrectPin(
          leftoverAttempts: _wire2api_u8(raw[1]),
          isFinalAttempt: _wire2api_bool(raw[2]),
        );
      case 2:
        return WalletUnlockResult_Timeout(
          timeoutMillis: _wire2api_u64(raw[1]),
        );
      case 3:
        return WalletUnlockResult_Blocked();
      default:
        throw Exception("unreachable");
    }
  }
}

// Section: api2wire

@protected
int api2wire_u8(int raw) {
  return raw;
}

// Section: finalizer

class WalletCorePlatform extends FlutterRustBridgeBase<WalletCoreWire> {
  WalletCorePlatform(ffi.DynamicLibrary dylib) : super(WalletCoreWire(dylib));

// Section: api2wire

  @protected
  ffi.Pointer<wire_uint_8_list> api2wire_String(String raw) {
    return api2wire_uint_8_list(utf8.encoder.convert(raw));
  }

  @protected
  ffi.Pointer<wire_uint_8_list> api2wire_uint_8_list(Uint8List raw) {
    final ans = inner.new_uint_8_list_0(raw.length);
    ans.ref.ptr.asTypedList(raw.length).setAll(0, raw);
    return ans;
  }
// Section: finalizer

// Section: api_fill_to_wire
}

// ignore_for_file: camel_case_types, non_constant_identifier_names, avoid_positional_boolean_parameters, annotate_overrides, constant_identifier_names

// AUTO GENERATED FILE, DO NOT EDIT.
//
// Generated by `package:ffigen`.
// ignore_for_file: type=lint

/// generated by flutter_rust_bridge
class WalletCoreWire implements FlutterRustBridgeWireBase {
  @internal
  late final dartApi = DartApiDl(init_frb_dart_api_dl);

  /// Holds the symbol lookup function.
  final ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName) _lookup;

  /// The symbols are looked up in [dynamicLibrary].
  WalletCoreWire(ffi.DynamicLibrary dynamicLibrary) : _lookup = dynamicLibrary.lookup;

  /// The symbols are looked up with [lookup].
  WalletCoreWire.fromLookup(ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName) lookup)
      : _lookup = lookup;

  void store_dart_post_cobject(
    DartPostCObjectFnType ptr,
  ) {
    return _store_dart_post_cobject(
      ptr,
    );
  }

  late final _store_dart_post_cobjectPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(DartPostCObjectFnType)>>('store_dart_post_cobject');
  late final _store_dart_post_cobject = _store_dart_post_cobjectPtr.asFunction<void Function(DartPostCObjectFnType)>();

  Object get_dart_object(
    int ptr,
  ) {
    return _get_dart_object(
      ptr,
    );
  }

  late final _get_dart_objectPtr = _lookup<ffi.NativeFunction<ffi.Handle Function(ffi.UintPtr)>>('get_dart_object');
  late final _get_dart_object = _get_dart_objectPtr.asFunction<Object Function(int)>();

  void drop_dart_object(
    int ptr,
  ) {
    return _drop_dart_object(
      ptr,
    );
  }

  late final _drop_dart_objectPtr = _lookup<ffi.NativeFunction<ffi.Void Function(ffi.UintPtr)>>('drop_dart_object');
  late final _drop_dart_object = _drop_dart_objectPtr.asFunction<void Function(int)>();

  int new_dart_opaque(
    Object handle,
  ) {
    return _new_dart_opaque(
      handle,
    );
  }

  late final _new_dart_opaquePtr = _lookup<ffi.NativeFunction<ffi.UintPtr Function(ffi.Handle)>>('new_dart_opaque');
  late final _new_dart_opaque = _new_dart_opaquePtr.asFunction<int Function(Object)>();

  int init_frb_dart_api_dl(
    ffi.Pointer<ffi.Void> obj,
  ) {
    return _init_frb_dart_api_dl(
      obj,
    );
  }

  late final _init_frb_dart_api_dlPtr =
      _lookup<ffi.NativeFunction<ffi.IntPtr Function(ffi.Pointer<ffi.Void>)>>('init_frb_dart_api_dl');
  late final _init_frb_dart_api_dl = _init_frb_dart_api_dlPtr.asFunction<int Function(ffi.Pointer<ffi.Void>)>();

  void wire_init(
    int port_,
  ) {
    return _wire_init(
      port_,
    );
  }

  late final _wire_initPtr = _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_init');
  late final _wire_init = _wire_initPtr.asFunction<void Function(int)>();

  void wire_is_initialized(
    int port_,
  ) {
    return _wire_is_initialized(
      port_,
    );
  }

  late final _wire_is_initializedPtr = _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_is_initialized');
  late final _wire_is_initialized = _wire_is_initializedPtr.asFunction<void Function(int)>();

  void wire_is_valid_pin(
    int port_,
    ffi.Pointer<wire_uint_8_list> pin,
  ) {
    return _wire_is_valid_pin(
      port_,
      pin,
    );
  }

  late final _wire_is_valid_pinPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64, ffi.Pointer<wire_uint_8_list>)>>('wire_is_valid_pin');
  late final _wire_is_valid_pin = _wire_is_valid_pinPtr.asFunction<void Function(int, ffi.Pointer<wire_uint_8_list>)>();

  void wire_set_lock_stream(
    int port_,
  ) {
    return _wire_set_lock_stream(
      port_,
    );
  }

  late final _wire_set_lock_streamPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_set_lock_stream');
  late final _wire_set_lock_stream = _wire_set_lock_streamPtr.asFunction<void Function(int)>();

  void wire_clear_lock_stream(
    int port_,
  ) {
    return _wire_clear_lock_stream(
      port_,
    );
  }

  late final _wire_clear_lock_streamPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_clear_lock_stream');
  late final _wire_clear_lock_stream = _wire_clear_lock_streamPtr.asFunction<void Function(int)>();

  void wire_set_configuration_stream(
    int port_,
  ) {
    return _wire_set_configuration_stream(
      port_,
    );
  }

  late final _wire_set_configuration_streamPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_set_configuration_stream');
  late final _wire_set_configuration_stream = _wire_set_configuration_streamPtr.asFunction<void Function(int)>();

  void wire_clear_configuration_stream(
    int port_,
  ) {
    return _wire_clear_configuration_stream(
      port_,
    );
  }

  late final _wire_clear_configuration_streamPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_clear_configuration_stream');
  late final _wire_clear_configuration_stream = _wire_clear_configuration_streamPtr.asFunction<void Function(int)>();

  void wire_set_cards_stream(
    int port_,
  ) {
    return _wire_set_cards_stream(
      port_,
    );
  }

  late final _wire_set_cards_streamPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_set_cards_stream');
  late final _wire_set_cards_stream = _wire_set_cards_streamPtr.asFunction<void Function(int)>();

  void wire_clear_cards_stream(
    int port_,
  ) {
    return _wire_clear_cards_stream(
      port_,
    );
  }

  late final _wire_clear_cards_streamPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_clear_cards_stream');
  late final _wire_clear_cards_stream = _wire_clear_cards_streamPtr.asFunction<void Function(int)>();

  void wire_unlock_wallet(
    int port_,
    ffi.Pointer<wire_uint_8_list> pin,
  ) {
    return _wire_unlock_wallet(
      port_,
      pin,
    );
  }

  late final _wire_unlock_walletPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64, ffi.Pointer<wire_uint_8_list>)>>('wire_unlock_wallet');
  late final _wire_unlock_wallet =
      _wire_unlock_walletPtr.asFunction<void Function(int, ffi.Pointer<wire_uint_8_list>)>();

  void wire_lock_wallet(
    int port_,
  ) {
    return _wire_lock_wallet(
      port_,
    );
  }

  late final _wire_lock_walletPtr = _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_lock_wallet');
  late final _wire_lock_wallet = _wire_lock_walletPtr.asFunction<void Function(int)>();

  void wire_has_registration(
    int port_,
  ) {
    return _wire_has_registration(
      port_,
    );
  }

  late final _wire_has_registrationPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_has_registration');
  late final _wire_has_registration = _wire_has_registrationPtr.asFunction<void Function(int)>();

  void wire_register(
    int port_,
    ffi.Pointer<wire_uint_8_list> pin,
  ) {
    return _wire_register(
      port_,
      pin,
    );
  }

  late final _wire_registerPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64, ffi.Pointer<wire_uint_8_list>)>>('wire_register');
  late final _wire_register = _wire_registerPtr.asFunction<void Function(int, ffi.Pointer<wire_uint_8_list>)>();

  void wire_create_pid_issuance_redirect_uri(
    int port_,
  ) {
    return _wire_create_pid_issuance_redirect_uri(
      port_,
    );
  }

  late final _wire_create_pid_issuance_redirect_uriPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_create_pid_issuance_redirect_uri');
  late final _wire_create_pid_issuance_redirect_uri =
      _wire_create_pid_issuance_redirect_uriPtr.asFunction<void Function(int)>();

  void wire_cancel_pid_issuance(
    int port_,
  ) {
    return _wire_cancel_pid_issuance(
      port_,
    );
  }

  late final _wire_cancel_pid_issuancePtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_cancel_pid_issuance');
  late final _wire_cancel_pid_issuance = _wire_cancel_pid_issuancePtr.asFunction<void Function(int)>();

  void wire_reject_pid_issuance(
    int port_,
  ) {
    return _wire_reject_pid_issuance(
      port_,
    );
  }

  late final _wire_reject_pid_issuancePtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_reject_pid_issuance');
  late final _wire_reject_pid_issuance = _wire_reject_pid_issuancePtr.asFunction<void Function(int)>();

  void wire_accept_pid_issuance(
    int port_,
    ffi.Pointer<wire_uint_8_list> pin,
  ) {
    return _wire_accept_pid_issuance(
      port_,
      pin,
    );
  }

  late final _wire_accept_pid_issuancePtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64, ffi.Pointer<wire_uint_8_list>)>>(
          'wire_accept_pid_issuance');
  late final _wire_accept_pid_issuance =
      _wire_accept_pid_issuancePtr.asFunction<void Function(int, ffi.Pointer<wire_uint_8_list>)>();

  void wire_process_uri(
    int port_,
    ffi.Pointer<wire_uint_8_list> uri,
  ) {
    return _wire_process_uri(
      port_,
      uri,
    );
  }

  late final _wire_process_uriPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64, ffi.Pointer<wire_uint_8_list>)>>('wire_process_uri');
  late final _wire_process_uri = _wire_process_uriPtr.asFunction<void Function(int, ffi.Pointer<wire_uint_8_list>)>();

  ffi.Pointer<wire_uint_8_list> new_uint_8_list_0(
    int len,
  ) {
    return _new_uint_8_list_0(
      len,
    );
  }

  late final _new_uint_8_list_0Ptr =
      _lookup<ffi.NativeFunction<ffi.Pointer<wire_uint_8_list> Function(ffi.Int32)>>('new_uint_8_list_0');
  late final _new_uint_8_list_0 = _new_uint_8_list_0Ptr.asFunction<ffi.Pointer<wire_uint_8_list> Function(int)>();

  void free_WireSyncReturn(
    WireSyncReturn ptr,
  ) {
    return _free_WireSyncReturn(
      ptr,
    );
  }

  late final _free_WireSyncReturnPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(WireSyncReturn)>>('free_WireSyncReturn');
  late final _free_WireSyncReturn = _free_WireSyncReturnPtr.asFunction<void Function(WireSyncReturn)>();
}

final class _Dart_Handle extends ffi.Opaque {}

final class wire_uint_8_list extends ffi.Struct {
  external ffi.Pointer<ffi.Uint8> ptr;

  @ffi.Int32()
  external int len;
}

typedef DartPostCObjectFnType
    = ffi.Pointer<ffi.NativeFunction<ffi.Bool Function(DartPort port_id, ffi.Pointer<ffi.Void> message)>>;
typedef DartPort = ffi.Int64;
