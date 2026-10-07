// Autenticación y cuentas
import { invoke } from './invoke';
import type { MinecraftUser, DeviceCode } from '$lib/types/types';

export const getDeviceCode = () => invoke<DeviceCode>('get_device_code');
export const authenticateWithDeviceCode = (
	deviceCode: string,
	interval: number,
	expiresIn: number
) =>
	invoke<MinecraftUser>('authenticate_with_device_code', {
		deviceCode,
		interval,
		expiresIn
	});
export const addOfflineAccount = (username: string) =>
	invoke<MinecraftUser>('add_offline_account', { username });
export const getCurrentUser = () => invoke<MinecraftUser>('get_current_user');
export const accountAllowsMultiplayer = () => invoke<boolean>('account_allows_multiplayer');
export const logout = () => invoke<void>('logout');
export const getUserList = () => invoke<MinecraftUser[]>('get_user_list');
export const switchUser = (uuid: string) => invoke<MinecraftUser>('switch_user', { uuid });
export const removeUser = (uuid: string) => invoke<void>('remove_user', { uuid });
