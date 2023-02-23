import 'package:shared_preferences/shared_preferences.dart';

import '../../store/language_store.dart';

const _kPreferredLanguageCodeKey = 'preferred_language_code';

class LanguageStoreImpl extends LanguageStore {
  final PreferenceProvider _preferences;

  LanguageStoreImpl(this._preferences);

  @override
  Future<String?> getPreferredLanguageCode() async {
    final prefs = await _preferences.call();
    return prefs.getString(_kPreferredLanguageCodeKey);
  }

  @override
  Future<void> setPreferredLanguageCode(String? languageCode) async {
    final prefs = await _preferences.call();
    if (languageCode == null) {
      prefs.remove(_kPreferredLanguageCodeKey);
    } else {
      prefs.setString(_kPreferredLanguageCodeKey, languageCode);
    }
  }
}

typedef PreferenceProvider = Future<SharedPreferences> Function();
