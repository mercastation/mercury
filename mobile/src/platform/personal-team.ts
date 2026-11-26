// A build signed with a free Apple account (MOBILE_PERSONAL_TEAM in
// app.config.ts): it has no push and no Sign in with Apple, so the screens
// that need them are not offered at all.
import Constants from 'expo-constants';

export const PERSONAL_TEAM = Constants.expoConfig?.extra?.personalTeam === true;
