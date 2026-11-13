// Public authentication state only. OAuth credentials and device polling stay in Rust.
import { invoke } from './invoke';
import type { MinecraftUser } from '$lib/types/types';
export type AuthMethod = 'auto' | 'embedded' | 'external' | 'device';
export interface MicrosoftIdentity {
	client_id: string;
	test_build: boolean;
	custom_identity: boolean;
}
export interface MicrosoftLogin {
	id: string;
	phase: string;
	method: string;
	domain: string | null;
	user_code: string | null;
	verification_uri: string | null;
	error: string | null;
	user: MinecraftUser | null;
	expires_in: number;
	test_build: boolean;
}
export const microsoftAuthConfig = () => invoke<MicrosoftIdentity>('microsoft_auth_config');
export const startMicrosoftAuth = (method: AuthMethod, expectedUuid?: string) =>
	invoke<MicrosoftLogin>('start_microsoft_auth', { method, expectedUuid: expectedUuid ?? null });
export const microsoftAuthStatus = (id: string) =>
	invoke<MicrosoftLogin>('microsoft_auth_status', { id });
export const cancelMicrosoftAuth = (id: string) =>
	invoke<MicrosoftLogin>('cancel_microsoft_auth', { id });
export const microsoftMemorySession = (id: string) =>
	invoke<MicrosoftLogin>('microsoft_auth_memory_session', { id });
export const microsoftRetryStorage = (id: string) =>
	invoke<MicrosoftLogin>('microsoft_auth_retry_storage', { id });
export const microsoftOpenDevice = (id: string) =>
	invoke<void>('microsoft_auth_open_device', { id });
export const accountSessionStatuses = () =>
	invoke<{ uuid: string; state: string }[]>('account_session_statuses');
export const addOfflineAccount = (username: string) =>
	invoke<MinecraftUser>('add_offline_account', { username });
export const getCurrentUser = () => invoke<MinecraftUser>('get_current_user');
export const accountAllowsMultiplayer = () => invoke<boolean>('account_allows_multiplayer');
export const logout = () => invoke<void>('logout');
export const getUserList = () => invoke<MinecraftUser[]>('get_user_list');
export const switchUser = (uuid: string) => invoke<MinecraftUser>('switch_user', { uuid });
export const removeUser = (uuid: string) => invoke<void>('remove_user', { uuid });
