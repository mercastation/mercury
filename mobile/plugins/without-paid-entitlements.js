// THE BUILD FOR A FREE APPLE ACCOUNT TAKES BACK WHAT APPLE WILL NOT SIGN WITH
// ONE (MOBILE_PERSONAL_TEAM in app.config.ts): push and Sign in with Apple.
// Expo applies an installed expo-apple-authentication's and expo-notifications'
// plugins by itself, listed or not (@expo/prebuild-config's
// withVersionedExpoSDKPlugins), so leaving them out of `plugins` is not enough.
// This one is listed first, so it runs after every other and has the last
// word. Their modules stay built in; asking them simply fails, and the app
// does not ask (src/platform/personal-team.ts).
const { withEntitlementsPlist } = require('expo/config-plugins');

const PAID_ONLY = ['aps-environment', 'com.apple.developer.applesignin'];

module.exports = function withoutPaidEntitlements(config) {
  return withEntitlementsPlist(config, (mod) => {
    for (const key of PAID_ONLY) delete mod.modResults[key];
    return mod;
  });
};
